import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { ToggleSwitch } from "../components/ToggleSwitch";
import { useWizWalker, type ClientInfo } from "../hooks/useWizWalker";
import { useTelemetry } from "../hooks/useTelemetry";

interface ControlItem {
  icon: string;
  label: string;
  key: string;
}

const controls: ControlItem[] = [
  { icon: "speed", label: "Speedhack", key: "speedhack" },
  { icon: "swords", label: "Auto Combat", key: "auto_combat" },
  { icon: "chat_bubble", label: "Auto Dialogue", key: "auto_dialogue" },
  { icon: "flag", label: "Auto Sigil", key: "auto_sigil" },
  { icon: "explore", label: "Auto Questing", key: "auto_questing" },
  { icon: "pets", label: "Pet Trainer", key: "pet_trainer" },
  { icon: "shield", label: "Anti-AFK Hooks", key: "anti_afk" },
];

const quickActions = [
  { icon: "location_searching", label: "Quest TP", action: "quest_tp" },
  { icon: "sync_alt", label: "XYZ Sync", action: "xyz_sync" },
  { icon: "door_open", label: "Exit Zone", action: "exit_zone" },
  { icon: "group_add", label: "Form Team", action: "form_team" },
  { icon: "auto_mode", label: "Loop Script", action: "loop_script" },
  { icon: "play_arrow", label: "Resume All", action: "resume_all", primary: true },
];

export function Dashboard() {
  const wiz = useWizWalker();
  const telemetry = useTelemetry();

  const [clients, setClients] = useState<ClientInfo[]>([]);
  const [toggles, setToggles] = useState<Record<string, boolean>>({});
  const [activeClient, setActiveClient] = useState("P1");
  const [scanStatus, setScanStatus] = useState<"idle" | "scanning" | "none_found">("idle");

  // Load initial state + listen for auto-detected clients from backend
  useEffect(() => {
    wiz.getClients().then(setClients).catch(() => {});
    wiz.getToggleStates().then(setToggles).catch(() => {});

    // When the background loop auto-detects new clients, refresh the list.
    const unlisten = listen("clients-changed", () => {
      wiz.getClients().then(setClients).catch(() => {});
    });

    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Toggle a hook via IPC → update local state on success
  const handleToggle = useCallback(async (key: string, enabled: boolean) => {
    try {
      await wiz.toggleHook(key, enabled);
      setToggles((prev) => ({ ...prev, [key]: enabled }));
    } catch (err) {
      console.error("Toggle failed:", err);
    }
  }, [wiz]);

  // Scan for clients via IPC — with feedback
  const handleScan = useCallback(async () => {
    setScanStatus("scanning");
    try {
      const found = await wiz.scanClients();
      setClients(found);
      if (found.length === 0) {
        setScanStatus("none_found");
        setTimeout(() => setScanStatus("idle"), 3000);
      } else {
        setScanStatus("idle");
      }
    } catch (err) {
      console.error("Scan failed:", err);
      setScanStatus("none_found");
      setTimeout(() => setScanStatus("idle"), 3000);
    }
  }, [wiz]);

  // Quick actions dispatch
  const handleAction = useCallback(async (action: string) => {
    try {
      switch (action) {
        case "xyz_sync":
          await wiz.xyzSync();
          break;
        default:
          console.log(`Action '${action}' not yet wired`);
      }
    } catch (err) {
      console.error("Action failed:", err);
    }
  }, [wiz]);

  // Fill empty client slots (up to 4)
  const clientSlots = [...clients];
  while (clientSlots.length < 4) {
    clientSlots.push({
      label: `P${clientSlots.length + 1}`,
      pid: 0,
      title: "",
      hooked: false,
      zone: "",
      isForeground: false,
      isRunning: false,
    });
  }

  return (
    <div className="flex-1 overflow-y-auto px-12 pb-24 pt-4 space-y-12">
      {/* Connected Clients */}
      <section>
        <div className="flex items-end justify-between mb-6">
          <h3 className="font-[var(--font-headline)] text-3xl font-light tracking-tight">
            Connected <span className="font-bold text-accent-violet">Clients</span>
          </h3>
          <div className="flex items-center gap-4">
            {scanStatus === "none_found" && (
              <span className="text-xs text-accent-amber/80 animate-pulse">
                No Wizard101 clients found
              </span>
            )}
            <button
              onClick={handleScan}
              disabled={scanStatus === "scanning"}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-widest transition-all ${
                scanStatus === "scanning"
                  ? "bg-accent-violet/5 text-accent-violet/40 cursor-wait"
                  : "bg-accent-violet/10 hover:bg-accent-violet/20 text-accent-violet"
              }`}
            >
              <span className={`material-symbols-outlined text-sm ${scanStatus === "scanning" ? "animate-spin" : ""}`}>
                {scanStatus === "scanning" ? "progress_activity" : "radar"}
              </span>
              {scanStatus === "scanning" ? "Scanning…" : "Scan"}
            </button>
          </div>
        </div>
        <div className="grid grid-cols-4 gap-6">
          {clientSlots.map((client) =>
            client.isRunning ? (
              <button
                key={client.label}
                onClick={() => setActiveClient(client.label)}
                className={`glass-card p-6 rounded-2xl flex flex-col gap-4 cursor-pointer transition-all duration-200 text-left ${
                  activeClient === client.label
                    ? "border border-accent-violet/30 glow-violet"
                    : "border border-text-dim/10 opacity-70 hover:opacity-100 hover:border-accent-violet/20"
                }`}
              >
                <div className="flex justify-between items-start">
                  <span
                    className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold ${
                      activeClient === client.label
                        ? "bg-accent-violet/20 text-accent-violet"
                        : "bg-bg-card-top text-text-secondary/60"
                    }`}
                  >
                    {client.label}
                  </span>
                  <div className={`w-2 h-2 rounded-full ${
                    client.hooked
                      ? "bg-accent-amber shadow-[0_0_8px_#ffb95f]"
                      : "bg-accent-cyan shadow-[0_0_8px_#39f0f5]"
                  }`} />
                </div>
                <div>
                  <p className="font-[var(--font-headline)] text-lg font-bold text-text-primary">
                    {client.zone || client.title || "Connected"}
                  </p>
                  <p className="text-text-secondary/60 text-xs font-medium">
                    PID {client.pid} {client.hooked ? "• Hooked" : ""}
                  </p>
                </div>
              </button>
            ) : (
              <div
                key={client.label}
                className="bg-bg-sunken/50 p-6 rounded-2xl border border-text-dim/5 flex flex-col items-center justify-center gap-2 opacity-30 grayscale"
              >
                <span className="material-symbols-outlined text-4xl">cloud_off</span>
                <p className="text-xs uppercase tracking-widest font-bold">Disconnected</p>
              </div>
            )
          )}
        </div>
      </section>

      {/* Controls + Telemetry Grid */}
      <div className="grid grid-cols-12 gap-10">
        {/* Quick Controls */}
        <section className="col-span-12 lg:col-span-5 flex flex-col">
          <div className="flex items-center gap-4 mb-6">
            <span className="material-symbols-outlined text-accent-amber">bolt</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Quick Controls
            </h4>
          </div>
          <div className="bg-bg-surface rounded-3xl p-8 space-y-4 shadow-2xl relative overflow-hidden">
            <div className="absolute top-0 right-0 w-32 h-32 bg-accent-amber/5 blur-3xl rounded-full" />
            {controls.map((control) => (
              <div
                key={control.key}
                className="flex items-center justify-between p-4 bg-bg-card-top/40 rounded-2xl border border-text-dim/10 group hover:border-accent-amber/20 transition-all"
              >
                <div className="flex items-center gap-4">
                  <span
                    className={`material-symbols-outlined transition-colors ${
                      toggles[control.key]
                        ? "text-accent-amber/40 group-hover:text-accent-amber"
                        : "text-text-secondary/40"
                    }`}
                  >
                    {control.icon}
                  </span>
                  <span className="font-medium tracking-wide">{control.label}</span>
                </div>
                <ToggleSwitch
                  enabled={toggles[control.key] ?? false}
                  onChange={(v) => handleToggle(control.key, v)}
                />
              </div>
            ))}
          </div>
        </section>

        {/* Live Telemetry */}
        <section className="col-span-12 lg:col-span-7 flex flex-col">
          <div className="flex items-center gap-4 mb-6">
            <span className="material-symbols-outlined text-accent-cyan">radar</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Live Telemetry
            </h4>
          </div>
          <div className="flex-1 flex flex-col gap-8">
            {/* XYZ Coordinates — now live from telemetry events */}
            <div className="bg-bg-sunken rounded-3xl p-8 border border-text-dim/10">
              <div className="flex justify-between items-center mb-8">
                <div className="flex items-center gap-3">
                  <div className={`w-3 h-3 rounded-full ${
                    telemetry.activeClient
                      ? "bg-accent-cyan animate-pulse-glow"
                      : "bg-text-dim/20"
                  }`} />
                  <span className="text-xs uppercase tracking-tighter text-text-secondary/40 font-bold">
                    Spatial Coordinates
                  </span>
                </div>
                <span className={`px-3 py-1 rounded-full text-[10px] font-black tracking-widest uppercase ${
                  telemetry.activeClient
                    ? "bg-accent-cyan/10 text-accent-cyan"
                    : "bg-text-dim/10 text-text-dim"
                }`}>
                  {telemetry.activeClient ? "Live" : "Waiting"}
                </span>
              </div>
              <div className="grid grid-cols-3 gap-8">
                {[
                  { axis: "X-Axis", value: telemetry.position.x.toFixed(3) },
                  { axis: "Y-Axis", value: telemetry.position.y.toFixed(3) },
                  { axis: "Z-Axis", value: telemetry.position.z.toFixed(3) },
                ].map((coord) => (
                  <div key={coord.axis} className="space-y-1">
                    <p className="text-[10px] text-text-secondary/30 uppercase font-black">
                      {coord.axis}
                    </p>
                    <p className="font-mono text-3xl font-light text-accent-cyan">
                      {coord.value}
                    </p>
                  </div>
                ))}
              </div>
              {/* Zone + Combat status */}
              <div className="mt-6 flex items-center gap-4 text-xs">
                <span className="px-3 py-1 rounded-full bg-bg-card-top text-text-secondary/60 font-bold">
                  {telemetry.zone}
                </span>
                {telemetry.inCombat && (
                  <span className="px-3 py-1 rounded-full bg-red-500/10 text-red-400 font-bold animate-pulse">
                    ⚔ In Combat
                  </span>
                )}
              </div>
            </div>

            {/* Quick Action Grid */}
            <div className="grid grid-cols-3 gap-4">
              {quickActions.map((action) => (
                <button
                  key={action.label}
                  onClick={() => handleAction(action.action)}
                  className={`rounded-2xl flex flex-col items-center justify-center gap-3 p-6 border group transition-all ${
                    action.primary
                      ? "bg-accent-violet/10 hover:bg-accent-violet/20 border-accent-violet/20"
                      : "bg-bg-card hover:bg-bg-card-top border-text-dim/10"
                  }`}
                >
                  <span
                    className={`material-symbols-outlined group-hover:scale-110 transition-transform ${
                      action.primary ? "filled text-accent-violet" : "text-accent-violet"
                    }`}
                  >
                    {action.icon}
                  </span>
                  <span
                    className={`text-[10px] uppercase font-bold tracking-widest ${
                      action.primary ? "text-accent-violet" : "text-text-secondary/80"
                    }`}
                  >
                    {action.label}
                  </span>
                </button>
              ))}
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
