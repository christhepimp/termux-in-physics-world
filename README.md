# Termux in a Physics World

Prototype that places a **real operating environment** inside a **physics-engine-controlled world**.

## Core idea

The physics engine is the reality layer. Termux (or a Termux-class Linux userspace session) exists *inside* that world — not as a separate desktop window glued on top.

```
┌─────────────────────────────────────────────┐
│           Physics World (Rapier3D)          │
│                                             │
│   ┌─────────────────────────────────────┐   │
│   │  In-world display surface (mesh)    │   │
│   │  ← terminal raster / text buffer    │   │
│   └─────────────────────────────────────┘   │
│                      ▲                      │
│                      │ stdin / stdout       │
│   ┌──────────────────┴──────────────────┐   │
│   │  Real OS session (PTY)              │   │
│   │  • Termux when available            │   │
│   │  • otherwise host Linux shell       │   │
│   └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## First version goals

- [x] Physics world is the environment (Bevy + Rapier3D)
- [x] Real process/PTY session (not a fake terminal simulator)
- [x] Display session output on a surface *inside* the 3D world
- [x] Forward keyboard input from the app into the session
- [ ] Custom physics interactions with the terminal object (later)

## Termux vs host shell

**Termux** is an Android userspace environment. This prototype:

1. Detects a Termux-like environment (`TERMUX_VERSION`, `$PREFIX` under `/data/data/com.termux`, or `termux-exec` on `PATH`).
2. If present, starts the session with Termux’s shell (`$PREFIX/bin/bash` or `login`).
3. Otherwise starts a **real host PTY** (`bash`/`sh`) so the architecture can be proven on desktop Linux/macOS/Windows without Android.

The integration path is the same either way: physics world owns the display object; the session is a child process behind a PTY bridge.

## Run

```bash
cargo run
```

Controls:

- **Type** — sent to the OS session
- **Enter / Backspace** — as usual
- **Esc** — quit
- Mouse — look around (optional orbit later)

## Stack

- Rust
- Bevy (window, rendering, input)
- Rapier3D (physics reality layer)
- Portable PTY (real terminal I/O)

## Non-goals (v0)

- Full Termux app embedding on non-Android
- Physics-driven cable plugs, smashable monitors, etc.
- GUI apps beyond a text session
