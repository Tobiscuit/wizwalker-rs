import { useEffect, useState, useCallback } from "react";
import { ToggleSwitch } from "../components/ToggleSwitch";
import { useWizWalker, type PlayerStats } from "../hooks/useWizWalker";
import { useTelemetry } from "../hooks/useTelemetry";

const combatToggles = [
  { key: "auto_combat", label: "Auto Combat", icon: "swords" },
  { key: "auto_dialogue", label: "Auto Dialogue", icon: "chat_bubble" },
  { key: "kill_minions", label: "Kill Minions First", icon: "target" },
  { key: "auto_sigil", label: "Auto Sigil", icon: "flag" },
  { key: "auto_potions", label: "Auto Potions", icon: "local_pharmacy" },
];

function formatStat(value: number, suffix = ""): string {
  if (value >= 1000) return value.toLocaleString() + suffix;
  return value.toString() + suffix;
}

export function Combat() {
  const wiz = useWizWalker();
  const telemetry = useTelemetry();

  const [toggles, setToggles] = useState<Record<string, boolean>>({});
  const [stats, setStats] = useState<PlayerStats | null>(null);

  const [playstyle, setPlaystyle] = useState(`# Fire PvE Strategy
round 1:
  - enchant "Epic" on "Meteor Strike"
  - cast "Fire Blade" on self

round 2:
  - cast "Meteor Strike" on all enemies

default:
  - cast "Fire Cat" on lowest HP enemy`);

  // Load initial toggle states
  useEffect(() => {
    wiz.getToggleStates().then(setToggles).catch(() => {});
  }, []);

  // Poll stats periodically (only when we have a connected client)
  useEffect(() => {
    if (!telemetry.activeClient) return;
    const interval = setInterval(() => {
      wiz.getStats().then(setStats).catch(() => {});
    }, 2000);
    return () => clearInterval(interval);
  }, [telemetry.activeClient]);

  const handleToggle = useCallback(async (key: string, enabled: boolean) => {
    try {
      await wiz.toggleHook(key, enabled);
      setToggles((prev) => ({ ...prev, [key]: enabled }));
    } catch (err) {
      console.error("Toggle failed:", err);
    }
  }, [wiz]);

  const statRows = stats
    ? [
        { label: "Max Health", value: formatStat(stats.maxHealth), color: "text-red-400" },
        { label: "Max Mana", value: formatStat(stats.maxMana), color: "text-blue-400" },
        { label: "Power Pip %", value: stats.powerPipChance.toFixed(0) + "%", color: "text-accent-amber" },
        { label: "Accuracy", value: stats.accuracy.toFixed(0) + "%", color: "text-accent-cyan" },
        { label: "Resist (all)", value: stats.resist.toFixed(0) + "%", color: "text-accent-violet" },
        { label: "Damage", value: stats.damage.toFixed(0) + "%", color: "text-orange-400" },
        { label: "Critical", value: formatStat(stats.critical), color: "text-yellow-300" },
        { label: "Pierce", value: stats.pierce.toFixed(0) + "%", color: "text-emerald-400" },
      ]
    : [
        { label: "Max Health", value: "—", color: "text-text-dim" },
        { label: "Max Mana", value: "—", color: "text-text-dim" },
        { label: "Power Pip %", value: "—", color: "text-text-dim" },
        { label: "Accuracy", value: "—", color: "text-text-dim" },
        { label: "Resist (all)", value: "—", color: "text-text-dim" },
        { label: "Damage", value: "—", color: "text-text-dim" },
        { label: "Critical", value: "—", color: "text-text-dim" },
        { label: "Pierce", value: "—", color: "text-text-dim" },
      ];

  return (
    <div className="flex-1 overflow-y-auto px-12 pb-24 pt-4 space-y-12">
      <section>
        <h3 className="font-[var(--font-headline)] text-3xl font-light tracking-tight mb-2">
          Combat <span className="font-bold text-accent-violet">&amp; Strategy</span>
        </h3>
        <p className="text-text-muted text-sm">
          Configure automatic combat behavior and view real-time battle stats.
          {telemetry.inCombat && (
            <span className="ml-2 text-red-400 font-bold animate-pulse">⚔ In Combat</span>
          )}
        </p>
      </section>

      <div className="grid grid-cols-12 gap-10">
        {/* Combat Toggles */}
        <section className="col-span-12 lg:col-span-5 space-y-6">
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-amber">swords</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Combat Controls
            </h4>
          </div>
          <div className="bg-bg-surface rounded-3xl p-8 space-y-4 shadow-2xl">
            {combatToggles.map((t) => (
              <div
                key={t.key}
                className="flex items-center justify-between p-4 bg-bg-card-top/40 rounded-2xl border border-text-dim/10 group hover:border-accent-amber/20 transition-all"
              >
                <div className="flex items-center gap-4">
                  <span className="material-symbols-outlined text-accent-amber/40 group-hover:text-accent-amber transition-colors">
                    {t.icon}
                  </span>
                  <span className="font-medium tracking-wide">{t.label}</span>
                </div>
                <ToggleSwitch
                  enabled={toggles[t.key] ?? false}
                  onChange={(v) => handleToggle(t.key, v)}
                />
              </div>
            ))}
          </div>

          {/* Stat Viewer — from IPC */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-cyan">bar_chart</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Stat Viewer
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl p-8 border border-text-dim/10">
            <div className="grid grid-cols-2 gap-4">
              {statRows.map((s) => (
                <div key={s.label} className="flex justify-between items-center p-3 bg-bg-card-top/30 rounded-xl">
                  <span className="text-text-secondary/60 text-sm">{s.label}</span>
                  <span className={`font-mono font-bold ${s.color}`}>{s.value}</span>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Playstyle Config */}
        <section className="col-span-12 lg:col-span-7 space-y-6">
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-violet">code</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Playstyle Configuration
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl border border-text-dim/10 overflow-hidden flex flex-col h-[600px]">
            <div className="flex items-center justify-between px-6 py-3 bg-bg-surface border-b border-border-subtle">
              <span className="text-xs text-text-muted uppercase tracking-widest font-bold">strategy.yaml</span>
              <div className="flex gap-3">
                <button className="text-xs px-3 py-1 bg-accent-violet/10 text-accent-violet rounded-lg hover:bg-accent-violet/20 transition-all">
                  Import
                </button>
                <button className="text-xs px-3 py-1 bg-accent-violet/10 text-accent-violet rounded-lg hover:bg-accent-violet/20 transition-all">
                  Export
                </button>
              </div>
            </div>
            <textarea
              value={playstyle}
              onChange={(e) => setPlaystyle(e.target.value)}
              className="flex-1 w-full bg-transparent p-6 font-mono text-sm text-accent-cyan resize-none outline-none leading-relaxed"
              spellCheck={false}
            />
          </div>

          <div className="flex gap-4">
            <button
              onClick={() => handleToggle("auto_combat", true)}
              className="flex-1 flex items-center justify-center gap-2 py-4 bg-accent-violet/10 border border-accent-violet/20 rounded-2xl text-accent-violet font-bold uppercase tracking-wider text-sm hover:bg-accent-violet/20 transition-all"
            >
              <span className="material-symbols-outlined filled">play_arrow</span>
              Start Combat
            </button>
            <button
              onClick={() => handleToggle("auto_combat", false)}
              className="flex-1 flex items-center justify-center gap-2 py-4 bg-accent-rose/10 border border-accent-rose/20 rounded-2xl text-accent-rose font-bold uppercase tracking-wider text-sm hover:bg-accent-rose/20 transition-all"
            >
              <span className="material-symbols-outlined">stop</span>
              Stop All
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
