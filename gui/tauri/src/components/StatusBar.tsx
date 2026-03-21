import { useTelemetry } from "../hooks/useTelemetry";

export function StatusBar() {
  const telemetry = useTelemetry();

  // Hide the status bar entirely when no client is connected — the
  // Dashboard's "Connected Clients" section already shows connection status.
  if (!telemetry.activeClient) return null;

  return (
    <footer className="w-full z-50 flex justify-between items-center px-6 py-2 bg-bg-sunken border-t border-border-subtle font-[var(--font-body)] text-[10px] uppercase tracking-widest h-10 shrink-0">
      <div className="flex items-center gap-6">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-green-400" />
          <span className="text-accent-amber">
            {telemetry.activeClient} — {telemetry.zone}
          </span>
        </div>
        <span className="text-text-secondary/40">|</span>
        <span className="text-green-400/60">Connected</span>
        {telemetry.inCombat && (
          <>
            <span className="text-text-secondary/40">|</span>
            <span className="text-red-400 animate-pulse">⚔ In Combat</span>
          </>
        )}
      </div>
      <div className="flex items-center gap-6">
        <span className="text-accent-amber font-bold">v0.1.0</span>
      </div>
    </footer>
  );
}
