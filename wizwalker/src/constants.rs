//! Constants and enumerations ported from Python `wizwalker/constants.py`.

/// Number of game units covered in 1 second of movement.
pub const WIZARD_SPEED: f32 = 580.0;

/// Virtual key codes for sending keyboard input to the game.
///
/// These map to Windows Virtual-Key Codes (VK_*).
/// Python equivalent: `wizwalker.constants.Keycode`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Keycode {
    LeftMouse = 1,
    RightMouse = 2,
    ControlBreak = 3,
    MiddleMouse = 4,
    X1Mouse = 5,
    X2Mouse = 6,
    Backspace = 8,
    Tab = 9,
    Clear = 12,
    Enter = 13,
    Shift = 16,
    Ctrl = 17,
    Alt = 18,
    Pause = 19,
    CapsLock = 20,
    Esc = 27,
    Spacebar = 32,
    PageUp = 33,
    PageDown = 34,
    End = 35,
    Home = 36,
    LeftArrow = 37,
    UpArrow = 38,
    RightArrow = 39,
    DownArrow = 40,
    Select = 41,
    Print = 42,
    Execute = 43,
    PrintScreen = 44,
    Ins = 45,
    Del = 46,
    Help = 47,
    Zero = 48,
    One = 49,
    Two = 50,
    Three = 51,
    Four = 52,
    Five = 53,
    Six = 54,
    Seven = 55,
    Eight = 56,
    Nine = 57,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    LeftWindows = 91,
    RightWindows = 92,
    Applications = 93,
    ComputerSleep = 95,
    NumPad0 = 96,
    NumPad1 = 97,
    NumPad2 = 98,
    NumPad3 = 99,
    NumPad4 = 100,
    NumPad5 = 101,
    NumPad6 = 102,
    NumPad7 = 103,
    NumPad8 = 104,
    NumPad9 = 105,
    Multiply = 106,
    Add = 107,
    Separator = 108,
    Subtract = 109,
    Decimal = 110,
    Divide = 111,
    F1 = 112,
    F2 = 113,
    F3 = 114,
    F4 = 115,
    F5 = 116,
    F6 = 117,
    F7 = 118,
    F8 = 119,
    F9 = 120,
    F10 = 121,
    F11 = 122,
    F12 = 123,
    NumLock = 144,
    ScrollLock = 145,
    LeftShift = 160,
    RightShift = 161,
    LeftControl = 162,
    RightControl = 163,
    LeftMenu = 164,
    RightMenu = 165,
}

impl Keycode {
    /// Get the raw virtual key code value.
    pub fn value(self) -> u32 {
        self as u32
    }
}

// ── Win32 Message Constants ─────────────────────────────────────────────

/// WM_KEYDOWN — posted when a key is pressed.
pub const WM_KEYDOWN: u32 = 0x0100;
/// WM_KEYUP — posted when a key is released.
pub const WM_KEYUP: u32 = 0x0101;
/// WM_MOUSEMOVE — posted when the cursor moves.
pub const WM_MOUSEMOVE: u32 = 0x0200;
/// WM_LBUTTONDOWN — posted when the left mouse button is pressed.
pub const WM_LBUTTONDOWN: u32 = 0x0201;
/// WM_LBUTTONUP — posted when the left mouse button is released.
pub const WM_LBUTTONUP: u32 = 0x0202;
/// WM_RBUTTONDOWN — posted when the right mouse button is pressed.
pub const WM_RBUTTONDOWN: u32 = 0x0204;
/// WM_RBUTTONUP — posted when the right mouse button is released.
pub const WM_RBUTTONUP: u32 = 0x0205;
