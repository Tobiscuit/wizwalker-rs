import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { Dashboard } from "./pages/Dashboard";

function App() {
  return (
    <BrowserRouter>
      <div className="flex h-screen w-screen bg-bg-base text-text-primary overflow-hidden">
        <Sidebar />
        <main className="pl-64 flex-1 flex flex-col overflow-hidden">
          {/* Top Header */}
          <header className="flex justify-between items-center w-full px-12 py-6 bg-bg-base/50 backdrop-blur-md font-[var(--font-headline)] font-medium z-40">
            <div className="flex items-center gap-8">
              <h2 className="text-2xl font-black text-accent-violet tracking-widest uppercase">
                Overview
              </h2>
              <div className="h-4 w-px bg-text-dim/20" />
              <div className="flex gap-6 text-sm">
                <span className="text-accent-violet border-b-2 border-accent-violet-deep pb-1 cursor-default">
                  Instance Controller
                </span>
                <span className="text-text-secondary/50 hover:text-text-primary transition-colors cursor-pointer">
                  Global Hooks
                </span>
                <span className="text-text-secondary/50 hover:text-text-primary transition-colors cursor-pointer">
                  API Keys
                </span>
              </div>
            </div>
            <div className="flex items-center gap-6">
              <button className="flex items-center gap-2 px-4 py-2 bg-accent-violet-bright/10 border border-accent-violet/20 rounded-lg text-accent-violet text-xs font-bold uppercase tracking-wider hover:bg-accent-violet/20 transition-all">
                <span className="material-symbols-outlined text-sm">add</span>
                New Client
              </button>
              <div className="flex gap-4">
                <span className="material-symbols-outlined text-text-secondary/70 hover:text-text-primary cursor-pointer transition-colors">
                  notifications
                </span>
                <span className="material-symbols-outlined text-text-secondary/70 hover:text-text-primary cursor-pointer transition-colors">
                  account_circle
                </span>
              </div>
            </div>
          </header>

          {/* Page Content */}
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/combat" element={<Placeholder title="Combat & Strategy" />} />
            <Route path="/navigation" element={<Placeholder title="Navigation & Teleport" />} />
            <Route path="/camera" element={<Placeholder title="Camera Controls" />} />
            <Route path="/scripting" element={<Placeholder title="Bot Scripting" />} />
            <Route path="/settings" element={<Placeholder title="Settings" />} />
          </Routes>
        </main>
        <StatusBar />
      </div>
    </BrowserRouter>
  );
}

function Placeholder({ title }: { title: string }) {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center space-y-4">
        <h2 className="font-[var(--font-headline)] text-4xl font-bold text-accent-violet">
          {title}
        </h2>
        <p className="text-text-muted">Coming soon — select Dashboard to see the prototype</p>
      </div>
    </div>
  );
}

export default App;
