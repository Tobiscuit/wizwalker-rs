import { useState } from "react";

export function Scripting() {
  const [script, setScript] = useState(`import wizwalker as ww

async def main():
    """Auto-farm Empyrea side quests."""
    client = await ww.connect()
    
    # Navigate to quest location
    await client.teleport_to_quest()
    await client.wait_for_zone_change()
    
    # Combat loop
    while client.in_combat:
        # Check for healing first
        if client.health_percent < 50:
            await client.cast("Rebirth", target="self")
        
        # AoE if 3+ enemies
        if client.enemy_count >= 3:
            await client.cast("Meteor Strike", target="all")
        else:
            await client.cast("Fire Cat", target="lowest_hp")
        
        await client.wait_for_round()
    
    # Collect rewards and continue
    await client.collect_rewards()
    await client.next_quest()

# Run the bot
ww.run(main)
`);

  const [output, setOutput] = useState(`[21:05:23] Bot initialized successfully
[21:05:24] Connected to WizardGraphicalClient.exe (P1)
[21:05:24] Zone: Empyrea - Aeriel Jungle
[21:05:25] Quest: "The Bat" — Step 3/5
[21:05:26] Teleporting to quest location...
[21:05:28] Zone change detected: Aeriel Jungle → Sky Caves
[21:05:30] Combat entered! Enemies: 3
[21:05:31] Casting: Meteor Strike → all enemies
[21:05:35] Round complete. HP: 8420/8420 | Mana: 612/650
[21:05:36] Waiting for round...`);

  const [isRunning, setIsRunning] = useState(false);

  return (
    <div className="flex-1 overflow-y-auto px-12 pb-24 pt-4 space-y-6">
      <section className="flex items-end justify-between">
        <div>
          <h3 className="font-[var(--font-headline)] text-3xl font-light tracking-tight mb-2">
            Bot <span className="font-bold text-accent-violet">Scripting</span>
          </h3>
          <p className="text-text-muted text-sm">
            Write, edit, and run automation scripts with the Arcane API.
          </p>
        </div>
        <div className="flex gap-3">
          <button className="text-xs px-4 py-2 bg-bg-card border border-text-dim/10 text-text-secondary/60 rounded-lg hover:text-text-primary transition-all">
            <span className="material-symbols-outlined text-sm align-middle mr-1">folder_open</span>
            Open
          </button>
          <button className="text-xs px-4 py-2 bg-bg-card border border-text-dim/10 text-text-secondary/60 rounded-lg hover:text-text-primary transition-all">
            <span className="material-symbols-outlined text-sm align-middle mr-1">save</span>
            Save
          </button>
          <button
            onClick={() => setIsRunning(!isRunning)}
            className={`text-xs px-6 py-2 rounded-lg font-bold uppercase tracking-wider transition-all flex items-center gap-2 ${
              isRunning
                ? "bg-accent-rose/10 border border-accent-rose/20 text-accent-rose hover:bg-accent-rose/20"
                : "bg-accent-violet/10 border border-accent-violet/20 text-accent-violet hover:bg-accent-violet/20"
            }`}
          >
            <span className="material-symbols-outlined text-sm filled">
              {isRunning ? "stop" : "play_arrow"}
            </span>
            {isRunning ? "Kill Bot" : "Run Bot"}
          </button>
        </div>
      </section>

      {/* IDE Layout */}
      <div className="flex flex-col gap-4 h-[calc(100vh-240px)]">
        {/* Code Editor */}
        <div className="flex-[3] bg-bg-sunken rounded-3xl border border-text-dim/10 overflow-hidden flex flex-col">
          <div className="flex items-center justify-between px-6 py-3 bg-bg-surface border-b border-border-subtle">
            <div className="flex items-center gap-4">
              <span className="text-xs text-text-muted uppercase tracking-widest font-bold">bot_script.py</span>
              {isRunning && (
                <span className="flex items-center gap-2 text-[10px] text-accent-amber uppercase tracking-widest font-bold">
                  <div className="w-2 h-2 rounded-full bg-accent-amber animate-pulse-glow" />
                  Running
                </span>
              )}
            </div>
            <span className="text-[10px] text-text-muted uppercase tracking-widest">Python • UTF-8 • LF</span>
          </div>
          <div className="flex-1 flex">
            {/* Line Numbers */}
            <div className="py-6 px-4 text-right select-none">
              {script.split("\n").map((_, i) => (
                <div key={i} className="text-[11px] leading-relaxed text-text-dim font-mono">
                  {i + 1}
                </div>
              ))}
            </div>
            {/* Code Area */}
            <textarea
              value={script}
              onChange={(e) => setScript(e.target.value)}
              className="flex-1 bg-transparent py-6 pr-6 font-mono text-sm text-accent-cyan resize-none outline-none leading-relaxed"
              spellCheck={false}
            />
          </div>
        </div>

        {/* Console Output */}
        <div className="flex-1 bg-bg-sunken rounded-3xl border border-text-dim/10 overflow-hidden flex flex-col min-h-[180px]">
          <div className="flex items-center justify-between px-6 py-3 bg-bg-surface border-b border-border-subtle">
            <span className="text-xs text-text-muted uppercase tracking-widest font-bold">Console Output</span>
            <button
              onClick={() => setOutput("")}
              className="text-[10px] text-text-muted hover:text-text-primary transition-colors uppercase tracking-widest"
            >
              Clear
            </button>
          </div>
          <pre className="flex-1 p-6 font-mono text-xs text-green-400/80 overflow-auto leading-relaxed">
            {output}
          </pre>
        </div>
      </div>
    </div>
  );
}
