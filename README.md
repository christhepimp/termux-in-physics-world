# Android + Termux as Inhabitants of a Physics World

This prototype embeds a **real Android runtime** (with **real Termux**) so that it exists **inside the physics engine’s world boundary**.

The physics engine is not a viewer of an external phone. It is the **reality layer and authority**. Android is a world-managed operating environment. Termux is the user environment inside that Android. There is **no supported external desktop window** and **no separate control path** around the physics world.

```
Physics Engine  =  Reality / World Control Layer
        |
        +-- World-owned virtual display (physics entity + texture)
        +-- World-owned virtual input
        +-- WorldControl (resources, power, time, devices)
        |
        +-- Android Runtime  (headless, spawned/attached by the world)
                |
                +-- Real Termux Environment
```

## What “inside the world” means here

| Requirement | Implementation |
|-------------|----------------|
| Physics is authority | `WorldControl` gates power, input, display, resources |
| Android not a separate UI | Emulator runs **headless** (`-no-window`); pixels only on the world screen |
| Display owned by world | `VirtualDisplay` → texture on `InhabitantScreen` body |
| Input owned by world | `VirtualInput` → only via world systems → guest |
| Termux is real | `com.termux` on that Android, launched under world policy |
| No side path | App code must not talk to the guest except through world modules |

## First version

- [x] Physics world (Bevy + Rapier) as the only user-facing window
- [x] Headless Android runtime management (`AndroidRuntime`)
- [x] Virtual display owned by the world (framebuffer → screen entity)
- [x] Virtual input owned by the world
- [x] `WorldControl` for power / I/O / resource / device policy foundation
- [x] Real Termux launch when package present
- [ ] Full virtual CPU/RAM/block hardware backends (hooks only)
- [ ] Gameplay physics interactions (not yet)

## Requirements

- Android SDK: `adb`, and optionally `emulator`
- An AVD name (or an already-running **headless** emulator/device)
- Termux installed in that Android environment

```bash
# Option A — let the world start a headless AVD
export VCE_AVD_NAME=YourAvdName
cargo run

# Option B — you start headless yourself, world attaches
emulator -avd YourAvdName -no-window -no-audio -gpu swiftshader_indirect &
adb wait-for-device
cargo run
```

**Controls (only through the physics app window)**  
Typing → world virtual input → Android/Termux  
**Ctrl+T** → world command: launch Termux  
**Ctrl+Q** → world command: shutdown app  

## Stack

Rust · Bevy · Rapier3D · Android emulator/ADB (headless guest)

See [ARCHITECTURE.md](ARCHITECTURE.md) and [CONTROL_PLANE.md](CONTROL_PLANE.md).
