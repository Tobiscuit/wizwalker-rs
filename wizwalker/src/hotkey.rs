use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PeekMessageW, MSG, PM_REMOVE,
};
use crate::errors::{Error, Result};
use bitflags::bitflags;

const MAX_HOTKEY_ID: i32 = 0xBFFF;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModifierKeys: u32 {
        const ALT = MOD_ALT.0;
        const CTRL = MOD_CONTROL.0;
        const NOREPEAT = MOD_NOREPEAT.0;
        const SHIFT = MOD_SHIFT.0;
    }
}

pub type HotkeyCallback = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

struct GlobalHotkeyIdentifierManager {
    hotkey_id_list: Vec<bool>,
}

impl GlobalHotkeyIdentifierManager {
    fn new() -> Self {
        Self {
            hotkey_id_list: Vec::new(),
        }
    }

    fn get_id(&mut self) -> Result<i32> {
        let id_list_len = self.hotkey_id_list.len();
        if id_list_len == MAX_HOTKEY_ID as usize {
            return Err(Error::Runtime(format!("Max hotkey id of {} reached", MAX_HOTKEY_ID)));
        }

        let all_true = self.hotkey_id_list.iter().all(|&x| x);
        if all_true {
            self.hotkey_id_list.push(true);
            Ok((id_list_len + 1) as i32)
        } else {
            let index = self.hotkey_id_list.iter().position(|&x| !x).unwrap();
            self.hotkey_id_list[index] = true;
            Ok((index + 1) as i32)
        }
    }

    fn free_id(&mut self, hotkey_id: i32) {
        if hotkey_id > 0 && hotkey_id as usize <= self.hotkey_id_list.len() {
            self.hotkey_id_list[(hotkey_id - 1) as usize] = false;
        }

        let all_false = self.hotkey_id_list.iter().all(|&x| !x);
        if all_false {
            self.hotkey_id_list.clear();
        }
    }
}

enum MessageLoopCommand {
    Register {
        keycode: u32,
        modifiers: ModifierKeys,
        response_tx: tokio::sync::oneshot::Sender<Result<i32>>,
    },
    Unregister {
        hotkey_id: i32,
        response_tx: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Stop,
}

pub struct HotkeyListener {
    hotkeys: Arc<Mutex<HashMap<(u32, ModifierKeys), i32>>>,
    callbacks: Arc<Mutex<HashMap<(u32, ModifierKeys), HotkeyCallback>>>,
    command_tx: Option<mpsc::UnboundedSender<MessageLoopCommand>>,
    callback_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl HotkeyListener {
    pub fn new() -> Self {
        Self {
            hotkeys: Arc::new(Mutex::new(HashMap::new())),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
            command_tx: None,
            callback_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn is_running(&self) -> bool {
        self.command_tx.is_some()
    }

    pub fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Value("This listener has already been started".to_string()));
        }

        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<MessageLoopCommand>();
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<(u32, ModifierKeys)>();

        self.command_tx = Some(cmd_tx);

        let callbacks = self.callbacks.clone();
        let callback_tasks = self.callback_tasks.clone();

        // Spawn Tokio task to handle events
        tokio::spawn(async move {
            while let Some((keycode, modifiers)) = event_rx.recv().await {
                let callbacks_guard = callbacks.lock().await;
                if let Some(callback) = callbacks_guard.get(&(keycode, modifiers)) {
                    let fut = callback();
                    let mut tasks = callback_tasks.lock().await;
                    tasks.push(tokio::spawn(async move {
                        fut.await;
                    }));
                }
            }
        });

        // Spawn OS thread for the Win32 message loop
        std::thread::spawn(move || {
            let mut id_manager = GlobalHotkeyIdentifierManager::new();

            loop {
                // Check for commands from async context
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        MessageLoopCommand::Register { keycode, modifiers, response_tx } => {
                            let res = id_manager.get_id().and_then(|id| {
                                unsafe {
                                    if RegisterHotKey(None, id, HOT_KEY_MODIFIERS(modifiers.bits()), keycode).is_ok() {
                                        Ok(id)
                                    } else {
                                        id_manager.free_id(id);
                                        Err(Error::Runtime("RegisterHotKey failed".to_string()))
                                    }
                                }
                            });
                            let _ = response_tx.send(res);
                        }
                        MessageLoopCommand::Unregister { hotkey_id, response_tx } => {
                            unsafe {
                                let _ = UnregisterHotKey(None, hotkey_id);
                            }
                            id_manager.free_id(hotkey_id);
                            let _ = response_tx.send(Ok(()));
                        }
                        MessageLoopCommand::Stop => {
                            return;
                        }
                    }
                }

                // Process Win32 messages
                unsafe {
                    let mut msg = MSG::default();
                    if PeekMessageW(&mut msg, None, 0x0312, 0x0312, PM_REMOVE).as_bool() {
                        let lparam = msg.lParam.0 as u32;
                        let modifiers = lparam & 0xFFFF;
                        let keycode = (lparam >> 16) & 0xFFFF;
                        let _ = event_tx.send((keycode, ModifierKeys::from_bits_truncate(modifiers)));
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(MessageLoopCommand::Stop);
        }

        let mut tasks = self.callback_tasks.lock().await;
        for task in tasks.drain(..) {
            task.abort();
        }

        // Win32 UnregisterHotKey must be called by the thread that registered them,
        // but since we send Stop to the thread, the OS will clean up when the thread exits.
        // Or we could have explicitly sent Unregister commands before stopping.
        self.hotkeys.lock().await.clear();
    }

    pub async fn add_hotkey(
        &self,
        key: u32,
        callback: HotkeyCallback,
        modifiers: ModifierKeys,
    ) -> Result<()> {
        let mut hotkeys = self.hotkeys.lock().await;

        if hotkeys.contains_key(&(key, modifiers)) {
            return Err(Error::Value(format!("{} with modifiers {:?} already registered", key, modifiers)));
        }

        if let Some(tx) = &self.command_tx {
            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            tx.send(MessageLoopCommand::Register {
                keycode: key,
                modifiers,
                response_tx: resp_tx,
            }).map_err(|_| Error::Runtime("Failed to send register command".into()))?;

            let id = resp_rx.await.map_err(|_| Error::Runtime("Response channel closed".into()))??;
            hotkeys.insert((key, modifiers), id);
        } else {
            return Err(Error::Runtime("Listener is not running".into()));
        }

        let mut callbacks = self.callbacks.lock().await;
        let no_norepeat = modifiers & !ModifierKeys::NOREPEAT;
        callbacks.insert((key, no_norepeat), callback);

        Ok(())
    }

    pub async fn remove_hotkey(&self, key: u32, modifiers: ModifierKeys) -> Result<()> {
        let mut hotkeys = self.hotkeys.lock().await;

        let id = hotkeys.remove(&(key, modifiers)).ok_or_else(|| {
            Error::Value(format!("No hotkey registered for key {} with modifiers {:?}", key, modifiers))
        })?;

        if let Some(tx) = &self.command_tx {
            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            tx.send(MessageLoopCommand::Unregister {
                hotkey_id: id,
                response_tx: resp_tx,
            }).map_err(|_| Error::Runtime("Failed to send unregister command".into()))?;

            resp_rx.await.map_err(|_| Error::Runtime("Response channel closed".into()))??;
        }

        let mut callbacks = self.callbacks.lock().await;
        let no_norepeat = modifiers & !ModifierKeys::NOREPEAT;
        callbacks.remove(&(key, no_norepeat));

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let keys: Vec<_> = self.hotkeys.lock().await.keys().copied().collect();
        for (key, modifiers) in keys {
            let _ = self.remove_hotkey(key, modifiers).await;
        }
        Ok(())
    }
}
