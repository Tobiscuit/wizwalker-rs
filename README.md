# 🧙 WizWalker-RS

> **WizWalker rewritten in Rust** — A Wizard101 memory manipulation library.

Rust port of [LaurenzLikeThat/wizwalker](https://github.com/LaurenzLikeThat/wizwalker) (Python), used as the foundation for [Deimos-Wizard101](https://github.com/Deimos-Wizard101/Deimos-Wizard101).

## Architecture

```
┌──────────────────────────────────────────────┐
│  deimos (binary crate)                       │
│  GUI (egui) + Bot Logic + Task Orchestration │
├──────────────────────────────────────────────┤
│  wizwalker (library crate)                   │
│  Memory Hooks · Game Structs · Input Sim     │
├──────────────────────────────────────────────┤
│  windows-rs → Win32 API → Game Process       │
└──────────────────────────────────────────────┘
```

### Workspace Structure

| Crate | Type | Purpose |
|-------|------|---------|
| `wizwalker/` | Library | Process memory R/W, inline hooking, game struct wrappers, input simulation |
| `deimos/` | Binary | GUI, combat AI, questing, pet training, scripting VM |

## Building

**Requirements:** Rust 1.85+, Windows 10/11 (Win32 APIs required)

```bash
cargo build --workspace
```

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `windows` | 0.59 | Win32 API bindings (ReadProcessMemory, etc.) |
| `tokio` | 1.x | Async runtime |
| `egui`/`eframe` | 0.31 | GUI framework |
| `thiserror` | 2.x | Error type derivation |
| `bitflags` | 2.x | Game flag enums |

## Origins

This project was mechanically ported from the Python WizWalker library by a fleet of AI agents (Google Jules), then manually verified and integrated.

- **Python source**: [LaurenzLikeThat/wizwalker](https://github.com/LaurenzLikeThat/wizwalker/tree/development) (correct offsets)
- **Deimos source**: [Tobiscuit/Deimos-Wizard101](https://github.com/Tobiscuit/Deimos-Wizard101)

## License

GPL-3.0 (matching upstream WizWalker)
