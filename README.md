# Operating Environment Inside a Physics Reality

Prototype of a **real Android + Termux environment** that exists **inside** a physics-engine world and is reachable only through that world.

The physics engine is not a viewer. It is the **reality and control layer**. Android and Termux are **inhabitants** of that reality.

```
┌─────────────────────────────────────────────────────────────┐
│                 PHYSICS ENGINE (reality layer)              │
│                                                             │
│  World rules · space · display bodies · control plane       │
│       │                                                     │
│       │ mediates every interaction                          │
│       ▼                                                     │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Android (operating environment)                      │  │
│  │     └── Termux (user environment)                     │  │
│  │           processes · packages · filesystem · UI      │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Goals

| Now (v0) | Later |
|----------|--------|
| Real Android + Termux attached to the world | Virtual hardware Android treats as real |
| Display inside the physics world | CPU / RAM / storage / network budgets |
| Input/output only via world-mediated bridges | World-controlled time |
| Control-plane foundation (`WorldControl`) | Physics rules that constrain the OE |

## Architecture

| Layer | Responsibility |
|-------|----------------|
| **Physics engine** | Reality: entities, display surface, **WorldControl** authority |
| **Android** | Guest operating environment (emulator or device) |
| **Termux** | User environment inside Android |
| **Bridges** | Framebuffer + input — *only* used through world systems |

Nothing outside the physics world is the primary UI. Typing and seeing Android/Termux both go through world systems.

## First version

- [x] Real Android via ADB (emulator or device)
- [x] Real Termux launch (`com.termux`) when installed
- [x] Framebuffer → texture on a **physics entity**
- [x] Keyboard → world system → Android input injection
- [x] `WorldControl` resource: policy hooks for power, time, resources, devices
- [ ] Gameplay physics (grabbing, throwing) — **not yet**
- [ ] Full virtual device backend — **foundation only**

## Requirements

- `adb` on `PATH`
- Android emulator **or** device with debugging
- Termux installed on that Android ([F-Droid](https://f-droid.org/packages/com.termux/))

```bash
adb devices
cargo run
```

**Ctrl+T** — launch Termux · **Ctrl+Q** — quit

## Control foundation

`WorldControl` is the single place future rules attach:

- Environment power / allowed interaction
- Time scale (world clock vs guest)
- Resource budgets (CPU, RAM, storage, network) — policy stubs
- Virtual device registry — empty, ready for devices the world will present to Android

See [ARCHITECTURE.md](ARCHITECTURE.md) and [CONTROL_PLANE.md](CONTROL_PLANE.md).

## Stack

Rust · Bevy · Rapier3D · ADB (Android / Termux host)
