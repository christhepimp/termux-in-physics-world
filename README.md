# Android + Termux Inside the Physics Engine Process

The physics engine process **owns** the operating environment. Android/Termux are not a separate app you remote-control from outside — their runtime service, display buffer, and input path live **inside the same process boundary** as the physics world (Bevy).

```
┌─ physics engine process (termux-world) ─────────────────────┐
│  Rapier / Bevy world                                        │
│  WorldControl · VirtualDisplay · VirtualInput               │
│  EmbeddedRuntime (in-process threads + shared frame buffer) │
│       │                                                     │
│       ├── [Android APK target] ART + Termux in same app     │
│       └── [Desktop] world-owned headless guest + in-proc I/O│
└─────────────────────────────────────────────────────────────┘
```

## Compile / run modes

### 1. Desktop (development)

```bash
cargo run
# optional: world-spawned headless AVD
export VCE_AVD_NAME=YourAvdName
cargo run
```

The physics **process** hosts:
- physics simulation
- in-process capture thread writing into a **shared frame buffer**
- in-process input injection path
- optional world-owned headless emulator child (still under this process tree)

There is no separate control UI. The Bevy window is the only window.

### 2. Android (true same-process inhabitant)

Build the physics engine **as an Android application** so it runs under ART next to Termux in the same userspace/device process model:

```bash
# requires cargo-apk / android NDK toolchain
cargo apk run --target aarch64-linux-android
```

On device, `EmbeddedRuntime` uses the in-process Android path (JNI hooks) instead of host `adb`.

## Architecture principle

| Piece | Process ownership |
|-------|-------------------|
| Physics world | This process |
| WorldControl | This process |
| Virtual display buffer | This process (`SharedFrameBuffer`) |
| Virtual input queue | This process |
| Runtime service threads | This process |
| Termux | Real Termux; on Android target, same device/app environment |

## Controls

- Type in the physics window → in-process virtual input → guest
- **Ctrl+T** launch Termux (world command)
- **Ctrl+Q** shutdown

See [ARCHITECTURE.md](ARCHITECTURE.md).
