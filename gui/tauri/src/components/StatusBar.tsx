export function StatusBar() {
  return (
    <footer className="fixed bottom-0 w-full z-50 flex justify-between items-center px-6 py-2 bg-bg-sunken border-t border-border-subtle font-[var(--font-body)] text-[10px] uppercase tracking-widest h-10">
      <div className="flex items-center gap-6">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-green-400" />
          <span className="text-accent-amber">
            P1 — Wizard City — WizardGraphicalClient.exe
          </span>
        </div>
        <span className="text-text-secondary/40">|</span>
        <span className="text-text-secondary/40">Connected</span>
      </div>
      <div className="flex items-center gap-6">
        <span className="text-text-secondary/40">Memory: 420MB</span>
        <span className="text-text-secondary/40">CPU: 4.2%</span>
        <span className="text-accent-amber font-bold">v0.1.0</span>
      </div>
    </footer>
  );
}
