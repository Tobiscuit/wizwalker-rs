import { NavLink } from "react-router-dom";

const navItems = [
  { path: "/", icon: "home", label: "Dashboard" },
  { path: "/combat", icon: "swords", label: "Combat" },
  { path: "/navigation", icon: "explore", label: "Navigation" },
  { path: "/camera", icon: "photo_camera", label: "Camera" },
  { path: "/scripting", icon: "code", label: "Scripting" },
];

export function Sidebar() {
  return (
    <aside className="fixed left-0 top-0 h-full z-50 flex flex-col w-64 bg-bg-surface/80 backdrop-blur-xl border-r border-border-subtle shadow-arcane">
      {/* Logo */}
      <div className="px-8 py-10 flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-accent-violet flex items-center justify-center shadow-[0_0_20px_rgba(139,92,246,0.4)]">
          <span className="material-symbols-outlined filled text-bg-base">
            auto_fix_high
          </span>
        </div>
        <div>
          <h1 className="text-xl font-bold text-accent-violet font-[var(--font-headline)]">
            Arcane
          </h1>
          <p className="text-[10px] uppercase tracking-widest text-text-muted">
            For Wizard101
          </p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 flex flex-col gap-2">
        {navItems.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) =>
              `flex items-center gap-4 px-4 py-3 rounded-xl transition-all duration-200 font-[var(--font-headline)] ${
                isActive
                  ? "text-accent-violet bg-bg-card-top glow-violet"
                  : "text-text-secondary/60 hover:text-accent-violet hover:bg-bg-card-top/50"
              }`
            }
          >
            {({ isActive }) => (
              <>
                <span
                  className={`material-symbols-outlined ${isActive ? "filled" : ""}`}
                >
                  {item.icon}
                </span>
                <span className="font-medium">{item.label}</span>
              </>
            )}
          </NavLink>
        ))}
      </nav>

      {/* Settings at bottom */}
      <div className="px-4 pb-6">
        <NavLink
          to="/settings"
          className={({ isActive }) =>
            `flex items-center gap-4 px-4 py-3 rounded-xl transition-all duration-200 font-[var(--font-headline)] ${
              isActive
                ? "text-accent-violet bg-bg-card-top glow-violet"
                : "text-text-secondary/60 hover:text-accent-violet hover:bg-bg-card-top/50"
            }`
          }
        >
          {({ isActive }) => (
            <>
              <span
                className={`material-symbols-outlined ${isActive ? "filled" : ""}`}
              >
                settings
              </span>
              <span className="font-medium">Settings</span>
            </>
          )}
        </NavLink>
      </div>
    </aside>
  );
}
