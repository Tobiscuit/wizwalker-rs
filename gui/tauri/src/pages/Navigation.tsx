import { useState, useCallback } from "react";
import { ToggleSwitch } from "../components/ToggleSwitch";
import { useWizWalker } from "../hooks/useWizWalker";
import { useTelemetry } from "../hooks/useTelemetry";

const teleportPresets = [
  { label: "Quest TP", icon: "location_searching", action: "quest_tp" },
  { label: "Mass TP", icon: "groups", action: "mass_tp" },
  { label: "Entity TP", icon: "person_pin_circle", action: "entity_tp" },
  { label: "Zone TP", icon: "door_open", action: "zone_tp" },
];

export function Navigation() {
  const wiz = useWizWalker();
  const telemetry = useTelemetry();

  const [freecam, setFreecam] = useState(false);
  const [xyzSyncEnabled, setXyzSyncEnabled] = useState(false);
  const [customX, setCustomX] = useState("");
  const [customY, setCustomY] = useState("");
  const [customZ, setCustomZ] = useState("");
  const [selectedWorld, setSelectedWorld] = useState("Wizard City");

  const worlds = [
    "Wizard City", "Krokotopia", "Marleybone", "MooShu",
    "Dragonspyre", "Celestia", "Zafaria", "Avalon",
    "Azteca", "Khrysalis", "Polaris", "Mirage",
    "Empyrea", "Karamelle", "Lemuria", "Novus", "Wallaru",
  ];

  // Copy current live position into the custom fields
  const handleCopyXYZ = useCallback(() => {
    setCustomX(telemetry.position.x.toFixed(3));
    setCustomY(telemetry.position.y.toFixed(3));
    setCustomZ(telemetry.position.z.toFixed(3));
  }, [telemetry.position]);

  // Teleport to the custom coordinates
  const handleTeleport = useCallback(async () => {
    const x = parseFloat(customX);
    const y = parseFloat(customY);
    const z = parseFloat(customZ);
    if (isNaN(x) || isNaN(y) || isNaN(z)) return;
    try {
      await wiz.teleportTo(x, y, z);
    } catch (err) {
      console.error("Teleport failed:", err);
    }
  }, [wiz, customX, customY, customZ]);

  // XYZ sync all clients
  const handleXyzSync = useCallback(async () => {
    try {
      await wiz.xyzSync();
    } catch (err) {
      console.error("XYZ Sync failed:", err);
    }
  }, [wiz]);

  return (
    <div className="flex-1 overflow-y-auto px-12 pb-24 pt-4 space-y-12">
      <section>
        <h3 className="font-[var(--font-headline)] text-3xl font-light tracking-tight mb-2">
          Navigation <span className="font-bold text-accent-violet">&amp; Teleport</span>
        </h3>
        <p className="text-text-muted text-sm">
          Teleport to locations, navigate worlds, and control the freecam.
        </p>
      </section>

      <div className="grid grid-cols-12 gap-10">
        {/* Position & Custom TP */}
        <section className="col-span-12 lg:col-span-5 space-y-6">
          {/* Current Position — Live from telemetry */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-cyan">my_location</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Current Position
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl p-8 border border-text-dim/10">
            <div className="grid grid-cols-3 gap-6">
              {[
                { axis: "X", value: telemetry.position.x.toFixed(3) },
                { axis: "Y", value: telemetry.position.y.toFixed(3) },
                { axis: "Z", value: telemetry.position.z.toFixed(3) },
              ].map((c) => (
                <div key={c.axis} className="space-y-2">
                  <p className="text-[10px] text-text-secondary/30 uppercase font-black">{c.axis}-Axis</p>
                  <p className="font-mono text-2xl font-light text-accent-cyan">{c.value}</p>
                </div>
              ))}
            </div>
            <div className="flex gap-3 mt-6">
              <button
                onClick={handleCopyXYZ}
                className="flex-1 text-xs py-2 bg-accent-cyan/10 text-accent-cyan rounded-lg hover:bg-accent-cyan/20 transition-all uppercase tracking-wider font-bold"
              >
                Copy XYZ
              </button>
              <button
                onClick={handleTeleport}
                className="flex-1 text-xs py-2 bg-accent-violet/10 text-accent-violet rounded-lg hover:bg-accent-violet/20 transition-all uppercase tracking-wider font-bold"
              >
                Paste & TP
              </button>
            </div>
          </div>

          {/* Custom Teleport */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-amber">pin_drop</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Custom Teleport
            </h4>
          </div>
          <div className="bg-bg-surface rounded-3xl p-8 space-y-4 shadow-2xl">
            {[
              { label: "X", value: customX, set: setCustomX },
              { label: "Y", value: customY, set: setCustomY },
              { label: "Z", value: customZ, set: setCustomZ },
            ].map((input) => (
              <div key={input.label} className="flex items-center gap-4">
                <span className="text-xs font-bold text-text-muted w-6">{input.label}</span>
                <input
                  type="text"
                  value={input.value}
                  onChange={(e) => input.set(e.target.value)}
                  placeholder="0.000"
                  className="flex-1 bg-bg-card-top/40 border border-text-dim/10 rounded-xl px-4 py-3 font-mono text-sm text-accent-cyan outline-none focus:border-accent-violet/30 transition-colors"
                />
              </div>
            ))}
            <button
              onClick={handleTeleport}
              className="w-full py-3 mt-2 bg-accent-violet/10 border border-accent-violet/20 rounded-2xl text-accent-violet font-bold uppercase tracking-wider text-sm hover:bg-accent-violet/20 transition-all flex items-center justify-center gap-2"
            >
              <span className="material-symbols-outlined text-sm">near_me</span>
              Teleport to Coordinates
            </button>
          </div>

          {/* Toggles */}
          <div className="bg-bg-surface rounded-3xl p-6 space-y-4 shadow-2xl">
            <div className="flex items-center justify-between p-4 bg-bg-card-top/40 rounded-2xl border border-text-dim/10">
              <div className="flex items-center gap-4">
                <span className="material-symbols-outlined text-accent-amber">videocam</span>
                <span className="font-medium">Freecam</span>
              </div>
              <ToggleSwitch enabled={freecam} onChange={setFreecam} />
            </div>
            <div className="flex items-center justify-between p-4 bg-bg-card-top/40 rounded-2xl border border-text-dim/10">
              <div className="flex items-center gap-4">
                <span className="material-symbols-outlined text-accent-cyan">sync</span>
                <span className="font-medium">XYZ Sync</span>
              </div>
              <ToggleSwitch
                enabled={xyzSyncEnabled}
                onChange={(v) => {
                  setXyzSyncEnabled(v);
                  if (v) handleXyzSync();
                }}
              />
            </div>
          </div>
        </section>

        {/* Quick Actions & World Selector */}
        <section className="col-span-12 lg:col-span-7 space-y-6">
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-violet">bolt</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Quick Actions
            </h4>
          </div>
          <div className="grid grid-cols-2 gap-4">
            {teleportPresets.map((tp) => (
              <button
                key={tp.label}
                className="bg-bg-card hover:bg-bg-card-top transition-all rounded-2xl flex items-center gap-4 p-6 border border-text-dim/10 group"
              >
                <span className="material-symbols-outlined text-accent-violet group-hover:scale-110 transition-transform text-2xl">
                  {tp.icon}
                </span>
                <div className="text-left">
                  <p className="font-bold text-sm">{tp.label}</p>
                  <p className="text-text-muted text-xs">Teleport action</p>
                </div>
              </button>
            ))}
          </div>

          {/* World Navigation */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-amber">public</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              World Navigation
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl p-8 border border-text-dim/10">
            <div className="grid grid-cols-3 gap-3">
              {worlds.map((world) => (
                <button
                  key={world}
                  onClick={() => setSelectedWorld(world)}
                  className={`text-sm py-3 px-4 rounded-xl transition-all ${
                    selectedWorld === world
                      ? "bg-accent-violet/20 text-accent-violet border border-accent-violet/30"
                      : "bg-bg-card-top/30 text-text-secondary/60 border border-text-dim/10 hover:text-text-primary hover:border-accent-violet/20"
                  }`}
                >
                  {world}
                </button>
              ))}
            </div>
            <button className="w-full py-3 mt-6 bg-accent-amber/10 border border-accent-amber/20 rounded-2xl text-accent-amber font-bold uppercase tracking-wider text-sm hover:bg-accent-amber/20 transition-all flex items-center justify-center gap-2">
              <span className="material-symbols-outlined text-sm">flight_takeoff</span>
              Navigate to {selectedWorld}
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
