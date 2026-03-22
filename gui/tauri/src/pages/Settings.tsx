import { useEffect, useState, useCallback, useRef } from "react";
import { ToggleSwitch } from "../components/ToggleSwitch";
import { useWizWalker } from "../hooks/useWizWalker";
import { LazyStore } from "@tauri-apps/plugin-store";

// Persistent settings store — matches Python Deimos config.ini behavior
const settingsStore = new LazyStore("settings.json");

const hotkeys = [
  { action: "Speed Toggle", key: "F5" },
  { action: "Quest Teleport", key: "F7" },
  { action: "Mass Teleport", key: "F6" },
  { action: "XYZ Sync", key: "F8" },
  { action: "Auto Combat", key: "9" },
  { action: "Auto Dialogue", key: "F4" },
  { action: "Auto Sigil", key: "F2" },
  { action: "Auto Questing", key: "F3" },
  { action: "Freecam", key: "F1" },
  { action: "Kill Tool", key: "F9" },
];

export function Settings() {
  const wiz = useWizWalker();
  const storeLoaded = useRef(false);

  const [settings, setSettings] = useState({
    auto_potions: false,
    discord_rpc: false,
    drop_logging: false,
    anti_afk: false,
    auto_update: false,
    always_on_top: false,
  });

  const [speedMultiplier, setSpeedMultiplier] = useState(1.0);
  const [theme, setTheme] = useState<"dark" | "light">("dark");

  // Load initial state — from persistent store first, then IPC as fallback
  useEffect(() => {
    // Load persisted settings from disk
    (async () => {
      try {
        const savedSpeed = await settingsStore.get<number>("speed_multiplier");
        if (savedSpeed != null) {
          setSpeedMultiplier(savedSpeed);
          await wiz.setSpeedMultiplier(savedSpeed); // Sync to backend
        }
        const savedAntiAfk = await settingsStore.get<boolean>("anti_afk");
        if (savedAntiAfk != null) {
          setSettings((prev) => ({ ...prev, anti_afk: savedAntiAfk }));
        }
      } catch {
        // Store not available yet, will use defaults
      }
      storeLoaded.current = true;
    })();

    // Also sync with backend IPC state
    wiz.getToggleStates().then((toggles) => {
      setSettings((prev) => ({
        ...prev,
        auto_potions: toggles.auto_potions ?? false,
        anti_afk: toggles.anti_afk ?? false,
      }));
    }).catch(() => {});

    wiz.getSpeedMultiplier().then(setSpeedMultiplier).catch(() => {});
  }, []);

  // Toggle a setting via IPC + persist to disk
  const handleToggle = useCallback(async (key: string, enabled: boolean) => {
    setSettings((prev) => ({ ...prev, [key]: enabled }));
    try {
      await wiz.toggleHook(key, enabled);
      // Persist toggleable settings
      if (["anti_afk", "auto_potions", "drop_logging", "discord_rpc"].includes(key)) {
        await settingsStore.set(key, enabled);
      }
    } catch (err) {
      console.error("Toggle failed:", err);
      // Revert on failure
      setSettings((prev) => ({ ...prev, [key]: !enabled }));
    }
  }, [wiz]);

  // Speed multiplier — update via IPC + persist to disk
  const handleSpeedChange = useCallback(async (delta: number) => {
    const newValue = Math.min(20, Math.max(1, speedMultiplier + delta));
    setSpeedMultiplier(newValue);
    try {
      await wiz.setSpeedMultiplier(newValue);
      await settingsStore.set("speed_multiplier", newValue);
    } catch (err) {
      console.error("Set speed failed:", err);
    }
  }, [wiz, speedMultiplier]);

  return (
    <div className="flex-1 overflow-y-auto px-12 pb-24 pt-4 space-y-12">
      <section>
        <h3 className="font-[var(--font-headline)] text-3xl font-light tracking-tight mb-2">
          <span className="font-bold text-accent-violet">Settings</span>
        </h3>
        <p className="text-text-muted text-sm">
          Configure the core parameters of your arcane automation environment.
        </p>
      </section>

      <div className="grid grid-cols-12 gap-10">
        {/* Hotkey Bindings */}
        <section className="col-span-12 lg:col-span-6 space-y-6">
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-amber">keyboard</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Hotkey Bindings
            </h4>
          </div>
          <div className="bg-bg-surface rounded-3xl p-6 shadow-2xl">
            <div className="grid grid-cols-2 gap-3">
              {hotkeys.map((hk) => (
                <div
                  key={hk.action}
                  className="flex items-center justify-between p-4 bg-bg-card-top/40 rounded-2xl border border-text-dim/10 hover:border-accent-violet/20 transition-all group"
                >
                  <span className="text-sm font-medium">{hk.action}</span>
                  <span className="px-3 py-1 rounded-lg bg-accent-cyan/10 text-accent-cyan text-xs font-bold">
                    {hk.key}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* General Settings */}
        <section className="col-span-12 lg:col-span-6 space-y-6">
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-violet">tune</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              General Settings
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl p-6 border border-text-dim/10 space-y-6">
            {/* Speed Multiplier — wired to IPC */}
            <div>
              <p className="text-[10px] text-text-secondary/30 uppercase font-black mb-3">
                Speed Multiplier
              </p>
              <div className="flex items-center gap-4">
                <button
                  onClick={() => handleSpeedChange(-0.5)}
                  className="w-10 h-10 rounded-xl bg-bg-card-top flex items-center justify-center text-text-primary hover:bg-bg-card transition-colors"
                >
                  <span className="material-symbols-outlined">remove</span>
                </button>
                <span className="flex-1 text-center font-mono text-3xl font-light text-accent-amber">
                  {speedMultiplier.toFixed(1)}
                </span>
                <button
                  onClick={() => handleSpeedChange(0.5)}
                  className="w-10 h-10 rounded-xl bg-bg-card-top flex items-center justify-center text-text-primary hover:bg-bg-card transition-colors"
                >
                  <span className="material-symbols-outlined">add</span>
                </button>
              </div>
            </div>

            {/* Toggles — wired to IPC */}
            <div className="space-y-3">
              {[
                { key: "auto_potions", label: "Auto Potions" },
                { key: "discord_rpc", label: "Discord Rich Presence" },
                { key: "drop_logging", label: "Drop Logging" },
                { key: "anti_afk", label: "Anti-AFK" },
                { key: "auto_update", label: "Auto-Updating" },
              ].map((s) => (
                <div key={s.key} className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium">{s.label}</span>
                  <ToggleSwitch
                    enabled={settings[s.key as keyof typeof settings] as boolean}
                    onChange={(v) => handleToggle(s.key, v)}
                  />
                </div>
              ))}
            </div>
          </div>

          {/* Appearance */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-violet">palette</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Appearance
            </h4>
          </div>
          <div className="bg-bg-surface rounded-3xl p-6 shadow-2xl space-y-4">
            <div className="flex gap-3">
              <button
                onClick={() => setTheme("dark")}
                className={`flex-1 py-3 rounded-xl text-sm transition-all ${
                  theme === "dark"
                    ? "bg-accent-violet/20 text-accent-violet border border-accent-violet/30"
                    : "bg-bg-card-top/40 text-text-muted border border-text-dim/10"
                }`}
              >
                Dark
              </button>
              <button
                onClick={() => setTheme("light")}
                className={`flex-1 py-3 rounded-xl text-sm transition-all flex items-center justify-center gap-2 ${
                  theme === "light"
                    ? "bg-accent-violet/20 text-accent-violet border border-accent-violet/30"
                    : "bg-bg-card-top/40 text-text-muted border border-text-dim/10"
                }`}
              >
                Light
                <span className="material-symbols-outlined text-sm">lock</span>
              </button>
            </div>
            <div className="flex items-center justify-between py-2">
              <span className="text-sm font-medium">GUI Always On Top</span>
              <ToggleSwitch
                enabled={settings.always_on_top}
                onChange={(v) => setSettings((p) => ({ ...p, always_on_top: v }))}
              />
            </div>
          </div>

          {/* About */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-cyan">info</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              About
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl p-8 border border-text-dim/10">
            <div className="flex items-center gap-6 mb-6">
              <h2 className="font-[var(--font-headline)] text-3xl font-bold text-accent-violet">
                Arcane
              </h2>
              <span className="px-3 py-1 rounded-full bg-accent-rose/10 text-accent-rose text-[10px] font-black uppercase tracking-widest">
                Built with Rust
              </span>
              <a href="https://github.com/Tobiscuit/wizwalker-rs" target="_blank" className="text-accent-cyan text-sm hover:underline">
                Source Code
              </a>
            </div>
            <div className="flex items-center gap-4 mb-6">
              <span className="text-xs uppercase tracking-widest text-text-muted">Version</span>
              <span className="px-2 py-1 rounded bg-bg-card-top text-xs font-mono text-accent-amber">v0.1.0</span>
              <span className="text-xs text-text-dim">License: GPL v3</span>
            </div>
            <div className="flex gap-6">
              <button className="flex items-center gap-2 text-sm text-text-secondary/60 hover:text-text-primary transition-colors">
                <span className="material-symbols-outlined text-sm">description</span>
                Documentation
              </button>
              <button className="flex items-center gap-2 text-sm text-text-secondary/60 hover:text-text-primary transition-colors">
                <span className="material-symbols-outlined text-sm">support_agent</span>
                Support
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
