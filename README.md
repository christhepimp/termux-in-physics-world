# Termux Inside a Physics World

Prototype: the **real open-source Termux environment** (Android userspace + Termux app, packages, filesystem) runs as the operating environment **inside** a physics-engine world.

The physics engine is the reality layer. Termux is not a fake terminal widget and not merely a host-shell PTY — it is the actual Termux stack hosted by a real Android runtime whose display surface is an entity in the physics simulation.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│              Physics Engine (Bevy + Rapier3D)            │
│                   = world / reality layer                │
│                                                          │
│   ┌────────────────────────────────────────────────────┐ │
│   │  Display entity (mesh + collider in the world)     │ │
│   │  texture ← Android framebuffer frames              │ │
│   └──────────────────────▲─────────────────────────────┘ │
│                          │ scrcpy / ADB screen stream    │
│   ┌──────────────────────┴─────────────────────────────┐ │
│   │  Android runtime (emulator or device)              │ │
│   │    └── Termux app (real processes, $PREFIX, pkgs)  │ │
│   └────────────────────────────────────────────────────┘ │
│                          ▲                               │
│                          │ ADB input injection           │
│                     keyboard / touch                     │
└──────────────────────────────────────────────────────────┘
```

| Layer | Role |
|-------|------|
| **Physics engine** | Reality: space, bodies, the screen object |
| **Android runtime** | Hosts the complete Android environment |
| **Termux** | Real OE: shell, packages, filesystem under `$PREFIX` |

## First version scope

- [x] Physics world owns the display surface entity
- [x] Bridge to a **real Android** instance (emulator or device) via ADB
- [x] Stream Android framebuffer into the in-world display
- [x] Forward input into Android (so Termux can be used)
- [x] Detect / launch Termux package on the Android side
- [ ] Custom physics interactions (grab screen, etc.) — **not yet**
- [ ] Virtual hardware simulation — **not yet**

## Requirements

1. **Android SDK Platform Tools** (`adb` on `PATH`)
2. One of:
   - Android Emulator (AVD) with Google APIs / Play, **or**
   - Physical device with USB debugging
3. **Termux** installed on that Android environment  
   ([F-Droid Termux](https://f-droid.org/packages/com.termux/) recommended)
4. Optional but recommended: **scrcpy** for efficient screen streaming  
   (falls back to `adb exec-out screencap` if scrcpy is missing)

## Quick start

```bash
# 1. Start an emulator or plug in a device
adb devices

# 2. Install Termux if needed (example with local APK)
# adb install termux.apk

# 3. Run the physics world
cargo run
```

On startup the app:

1. Builds the physics world (floor, chassis, screen body)
2. Connects to the first `adb` device
3. Starts framebuffer capture into the screen texture
4. Tries to launch `com.termux`
5. Forwards keyboard input into the Android environment

## Controls

| Input | Action |
|-------|--------|
| Typing | Injected into Android (ADB input / text) |
| Enter / Backspace | Injected as keys |
| **Ctrl+Q** | Quit the physics app |
| **Ctrl+T** | Re-issue Termux launch intent |

## What “real Termux” means here

- Real Android system processes and Zygote-hosted apps
- Real Termux app (`com.termux`) with its own Linux userspace under `/data/data/com.termux/files`
- Real `pkg` / `apt`, real `$PREFIX`, real Termux-executed binaries
- UI of Termux (and the rest of Android) appears on the **in-world** screen mesh

The physics engine does not reimplement Termux. It **contains** the environment that runs Termux by treating the Android runtime as a subsystem whose only user-facing surface is a physics entity.

## Project layout

```
src/
  main.rs           # app wiring
  world.rs          # Rapier entities (reality layer)
  android_bridge.rs # ADB device, Termux launch, input
  framebuffer.rs    # screen capture → Bevy Image
  display.rs        # in-world textured screen
scripts/
  setup_emulator_hint.sh
ARCHITECTURE.md
```

## Non-goals (v0)

- Reimplementing Termux in Rust
- Host-shell PTY as a substitute for Termux (removed as primary path)
- Physics-based grabbing or cables
- Full system-image build from source (uses stock emulator/device + Termux APK)
