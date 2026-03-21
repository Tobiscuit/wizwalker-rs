import { useEffect, useState, useCallback } from "react";
import { ToggleSwitch } from "../components/ToggleSwitch";
import { useWizWalker, type CameraState } from "../hooks/useWizWalker";

export function Camera() {
  const wiz = useWizWalker();

  const [freecam, setFreecam] = useState(false);
  const [lockCamera, setLockCamera] = useState(true);
  const [rollEnabled, setRollEnabled] = useState(false);

  const [position, setPosition] = useState({ x: "0.00", y: "150.00", z: "-400.00" });
  const [rotation, setRotation] = useState({ yaw: "0.00", pitch: "25.00", roll: "0.00" });
  const [fov, setFov] = useState("60");
  const [distance, setDistance] = useState("350");

  const [flythroughScript, setFlythroughScript] = useState(`# Camera Flythrough Script
keyframe 0s:
  position: [0, 150, -400]
  rotation: [0, 25, 0]
  fov: 60

keyframe 3s:
  position: [200, 200, -300]
  rotation: [45, 15, 0]
  fov: 50

keyframe 6s:
  position: [0, 300, -100]
  rotation: [0, 45, 0]
  fov: 40`);

  // Load current camera state from backend
  useEffect(() => {
    wiz.getCamera()
      .then((cam: CameraState) => {
        setPosition({
          x: cam.position.x.toFixed(2),
          y: cam.position.y.toFixed(2),
          z: cam.position.z.toFixed(2),
        });
        setRotation({
          yaw: cam.yaw.toFixed(2),
          pitch: cam.pitch.toFixed(2),
          roll: cam.roll.toFixed(2),
        });
        setFov(cam.fov.toFixed(0));
        setDistance(cam.distance.toFixed(0));
      })
      .catch(() => {});
  }, []);

  // Apply camera position via IPC
  const handleApplyPosition = useCallback(async () => {
    const x = parseFloat(position.x);
    const y = parseFloat(position.y);
    const z = parseFloat(position.z);
    if (isNaN(x) || isNaN(y) || isNaN(z)) return;
    try {
      await wiz.setCameraPosition(x, y, z);
    } catch (err) {
      console.error("Set camera position failed:", err);
    }
  }, [wiz, position]);

  // Apply camera rotation via IPC
  const handleApplyRotation = useCallback(async () => {
    const yaw = parseFloat(rotation.yaw);
    const pitch = parseFloat(rotation.pitch);
    const roll = parseFloat(rotation.roll);
    if (isNaN(yaw) || isNaN(pitch) || isNaN(roll)) return;
    try {
      await wiz.setCameraRotation(yaw, pitch, roll);
    } catch (err) {
      console.error("Set camera rotation failed:", err);
    }
  }, [wiz, rotation]);

  // Apply FOV via IPC
  const handleApplyFov = useCallback(async () => {
    const f = parseFloat(fov);
    if (isNaN(f)) return;
    try {
      await wiz.setCameraFov(f);
    } catch (err) {
      console.error("Set camera FOV failed:", err);
    }
  }, [wiz, fov]);

  return (
    <div className="flex-1 overflow-y-auto px-12 pb-24 pt-4 space-y-12">
      <section>
        <h3 className="font-[var(--font-headline)] text-3xl font-light tracking-tight mb-2">
          Camera <span className="font-bold text-accent-violet">Controls</span>
        </h3>
        <p className="text-text-muted text-sm">
          Control camera position, rotation, field of view, and create flythrough sequences.
        </p>
      </section>

      <div className="grid grid-cols-12 gap-10">
        {/* Camera Settings */}
        <section className="col-span-12 lg:col-span-5 space-y-6">
          {/* Mode Toggles */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-amber">videocam</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Camera Mode
            </h4>
          </div>
          <div className="bg-bg-surface rounded-3xl p-6 space-y-4 shadow-2xl">
            {[
              { label: "Freecam", icon: "3d_rotation", state: freecam, set: setFreecam },
              { label: "Lock to Player", icon: "person_pin", state: lockCamera, set: setLockCamera },
              { label: "Roll Enabled", icon: "rotate_90_degrees_cw", state: rollEnabled, set: setRollEnabled },
            ].map((t) => (
              <div key={t.label} className="flex items-center justify-between p-4 bg-bg-card-top/40 rounded-2xl border border-text-dim/10">
                <div className="flex items-center gap-4">
                  <span className="material-symbols-outlined text-accent-amber/60">{t.icon}</span>
                  <span className="font-medium">{t.label}</span>
                </div>
                <ToggleSwitch enabled={t.state} onChange={t.set} />
              </div>
            ))}
          </div>

          {/* Position Inputs */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-cyan">open_with</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Position
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl p-6 border border-text-dim/10 space-y-3">
            {Object.entries(position).map(([axis, val]) => (
              <div key={axis} className="flex items-center gap-4">
                <span className="text-xs font-bold text-text-muted w-6 uppercase">{axis}</span>
                <input
                  type="text"
                  value={val}
                  onChange={(e) => setPosition((p) => ({ ...p, [axis]: e.target.value }))}
                  className="flex-1 bg-bg-card-top/40 border border-text-dim/10 rounded-xl px-4 py-3 font-mono text-sm text-accent-cyan outline-none focus:border-accent-violet/30 transition-colors"
                />
              </div>
            ))}
            <button
              onClick={handleApplyPosition}
              className="w-full py-2 mt-2 bg-accent-cyan/10 text-accent-cyan rounded-xl text-xs font-bold uppercase tracking-wider hover:bg-accent-cyan/20 transition-all"
            >
              Apply Position
            </button>
          </div>

          {/* Rotation Inputs */}
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-violet">360</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Rotation
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl p-6 border border-text-dim/10 space-y-3">
            {Object.entries(rotation).map(([axis, val]) => (
              <div key={axis} className="flex items-center gap-4">
                <span className="text-xs font-bold text-text-muted w-12 capitalize">{axis}</span>
                <input
                  type="text"
                  value={val}
                  onChange={(e) => setRotation((p) => ({ ...p, [axis]: e.target.value }))}
                  className="flex-1 bg-bg-card-top/40 border border-text-dim/10 rounded-xl px-4 py-3 font-mono text-sm text-accent-cyan outline-none focus:border-accent-violet/30 transition-colors"
                />
              </div>
            ))}
            <button
              onClick={handleApplyRotation}
              className="w-full py-2 mt-2 bg-accent-violet/10 text-accent-violet rounded-xl text-xs font-bold uppercase tracking-wider hover:bg-accent-violet/20 transition-all"
            >
              Apply Rotation
            </button>
          </div>

          {/* FOV & Distance */}
          <div className="bg-bg-surface rounded-3xl p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between">
              <span className="font-medium">Field of View</span>
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={fov}
                  onChange={(e) => setFov(e.target.value)}
                  onBlur={handleApplyFov}
                  className="w-20 bg-bg-card-top/40 border border-text-dim/10 rounded-lg px-3 py-2 font-mono text-sm text-accent-amber text-center outline-none"
                />
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="font-medium">Zoom Distance</span>
              <input
                type="text"
                value={distance}
                onChange={(e) => setDistance(e.target.value)}
                className="w-20 bg-bg-card-top/40 border border-text-dim/10 rounded-lg px-3 py-2 font-mono text-sm text-accent-amber text-center outline-none"
              />
            </div>
          </div>
        </section>

        {/* Flythrough Creator */}
        <section className="col-span-12 lg:col-span-7 space-y-6">
          <div className="flex items-center gap-4">
            <span className="material-symbols-outlined text-accent-violet">movie</span>
            <h4 className="font-[var(--font-headline)] text-xl font-bold tracking-tight uppercase">
              Flythrough Creator
            </h4>
          </div>
          <div className="bg-bg-sunken rounded-3xl border border-text-dim/10 overflow-hidden flex flex-col h-[500px]">
            <div className="flex items-center justify-between px-6 py-3 bg-bg-surface border-b border-border-subtle">
              <span className="text-xs text-text-muted uppercase tracking-widest font-bold">flythrough.yaml</span>
              <div className="flex gap-3">
                <button className="text-xs px-3 py-1 bg-accent-violet/10 text-accent-violet rounded-lg hover:bg-accent-violet/20 transition-all">
                  Load
                </button>
                <button className="text-xs px-3 py-1 bg-accent-violet/10 text-accent-violet rounded-lg hover:bg-accent-violet/20 transition-all">
                  Save
                </button>
              </div>
            </div>
            <textarea
              value={flythroughScript}
              onChange={(e) => setFlythroughScript(e.target.value)}
              className="flex-1 w-full bg-transparent p-6 font-mono text-sm text-accent-cyan resize-none outline-none leading-relaxed"
              spellCheck={false}
            />
          </div>

          <div className="flex gap-4">
            <button className="flex-1 py-4 bg-accent-violet/10 border border-accent-violet/20 rounded-2xl text-accent-violet font-bold uppercase tracking-wider text-sm hover:bg-accent-violet/20 transition-all flex items-center justify-center gap-2">
              <span className="material-symbols-outlined filled">play_arrow</span>
              Preview
            </button>
            <button className="flex-1 py-4 bg-accent-amber/10 border border-accent-amber/20 rounded-2xl text-accent-amber font-bold uppercase tracking-wider text-sm hover:bg-accent-amber/20 transition-all flex items-center justify-center gap-2">
              <span className="material-symbols-outlined">save</span>
              Export Video
            </button>
            <button className="py-4 px-6 bg-bg-card border border-text-dim/10 rounded-2xl text-text-secondary/60 hover:text-text-primary transition-all flex items-center justify-center gap-2">
              <span className="material-symbols-outlined">restart_alt</span>
              Reset
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
