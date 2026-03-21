import { useState } from "react";
import { ToggleSwitch } from "../components/ToggleSwitch";

interface ClientCard {
  id: string;
  zone: string;
  level: number;
  school: string;
  connected: boolean;
  progress: number;
}

const mockClients: ClientCard[] = [
  { id: "P1", zone: "Wizard City", level: 130, school: "Fire Wizard", connected: true, progress: 75 },
  { id: "P2", zone: "Krokotopia", level: 24, school: "Storm Wizard", connected: true, progress: 25 },
  { id: "P3", zone: "Marleybone", level: 42, school: "Life Wizard", connected: true, progress: 50 },
  { id: "P4", zone: "", level: 0, school: "", connected: false, progress: 0 },
];

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
  { icon: "location_searching", label: "Quest TP" },
  { icon: "sync_alt", label: "XYZ Sync" },
  { icon: "door_open", label: "Exit Zone" },
  { icon: "group_add", label: "Form Team" },
  { icon: "auto_mode", label: "Loop Script" },
  { icon: "play_arrow", label: "Resume All", primary: true },
];

export function Dashboard() {
  const [toggles, setToggles] = useState<Record<string, boolean>>({
    speedhack: true,
    auto_combat: true,
    auto_dialogue: true,
    auto_sigil: false,
    auto_questing: false,
    pet_trainer: true,
    anti_afk: true,
  });

  const [activeClient, setActiveClient] = useState("P1");

  const handleToggle = (key: string, enabled: boolean) => {
    setToggles((prev) => ({ ...prev, [key]: enabled }));
  };

  return (
    <div className="flex-1 overflow-y-auto px-12 pb-24 pt-4 space-y-12">
      {/* Connected Clients */}
      <section>
        <div className="flex items-end justify-between mb-6">
          <h3 className="font-[var(--font-headline)] text-3xl font-light tracking-tight">
            Connected <span className="font-bold text-accent-violet">Clients</span>
          </h3>
          <p className="text-text-secondary/40 text-xs tracking-[0.2em] uppercase mb-1">
            Status: Operational
          </p>
        </div>
        <div className="grid grid-cols-4 gap-6">
          {mockClients.map((client) =>
            client.connected ? (
              <button
                key={client.id}
                onClick={() => setActiveClient(client.id)}
                className={`glass-card p-6 rounded-2xl flex flex-col gap-4 cursor-pointer transition-all duration-200 text-left ${
                  activeClient === client.id
                    ? "border border-accent-violet/30 glow-violet"
                    : "border border-text-dim/10 opacity-70 hover:opacity-100 hover:border-accent-violet/20"
                }`}
              >
                <div className="flex justify-between items-start">
                  <span
                    className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold ${
                      activeClient === client.id
                        ? "bg-accent-violet/20 text-accent-violet"
                        : "bg-bg-card-top text-text-secondary/60"
                    }`}
                  >
                    {client.id}
                  </span>
                  <div className="w-2 h-2 rounded-full bg-accent-amber shadow-[0_0_8px_#ffb95f]" />
                </div>
                <div>
                  <p className="font-[var(--font-headline)] text-lg font-bold text-text-primary">
                    {client.zone}
                  </p>
                  <p className="text-text-secondary/60 text-xs font-medium">
                    Lvl {client.level} {client.school}
                  </p>
                </div>
                <div className="mt-2 h-1 bg-bg-card-top rounded-full overflow-hidden">
                  <div
                    className="h-full bg-accent-violet transition-all"
                    style={{ width: `${client.progress}%` }}
                  />
                </div>
              </button>
            ) : (
              <div
                key={client.id}
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
                  enabled={toggles[control.key]}
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
            {/* XYZ Coordinates */}
            <div className="bg-bg-sunken rounded-3xl p-8 border border-text-dim/10">
              <div className="flex justify-between items-center mb-8">
                <div className="flex items-center gap-3">
                  <div className="w-3 h-3 rounded-full bg-accent-cyan animate-pulse-glow" />
                  <span className="text-xs uppercase tracking-tighter text-text-secondary/40 font-bold">
                    Spatial Coordinates
                  </span>
                </div>
                <span className="px-3 py-1 rounded-full bg-accent-cyan/10 text-accent-cyan text-[10px] font-black tracking-widest uppercase">
                  Hooked
                </span>
              </div>
              <div className="grid grid-cols-3 gap-8">
                {[
                  { axis: "X-Axis", value: "142.882" },
                  { axis: "Y-Axis", value: "-94.103" },
                  { axis: "Z-Axis", value: "0.004" },
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
              {/* Decorative Waveform */}
              <div className="mt-8 h-12 flex items-end gap-1 opacity-20">
                {[2, 4, 6, 3, 8, 5, 2, 4, 7, 3, 5, 8, 4, 6, 3].map((h, i) => (
                  <div
                    key={i}
                    className="w-full bg-accent-cyan rounded-t"
                    style={{ height: `${h * 5}px` }}
                  />
                ))}
              </div>
            </div>

            {/* Quick Action Grid */}
            <div className="grid grid-cols-3 gap-4">
              {quickActions.map((action) => (
                <button
                  key={action.label}
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
