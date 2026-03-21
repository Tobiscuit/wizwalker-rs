interface ToggleSwitchProps {
  enabled: boolean;
  onChange: (enabled: boolean) => void;
}

export function ToggleSwitch({ enabled, onChange }: ToggleSwitchProps) {
  return (
    <button
      onClick={() => onChange(!enabled)}
      className={`w-12 h-6 rounded-full relative p-1 cursor-pointer transition-all duration-200 ${
        enabled
          ? "bg-accent-amber glow-amber"
          : "bg-bg-sunken"
      }`}
    >
      <div
        className={`w-4 h-4 rounded-full transition-all duration-200 ${
          enabled
            ? "bg-accent-amber-dark ml-auto"
            : "bg-border-outline ml-0"
        }`}
      />
    </button>
  );
}
