# In-process architecture

## Goal

Compile and run so the **physics engine process** is the container for the OE integration. Display and input are in-process. On Android, the engine itself is the Android app process.

## Process diagram

```
termux-world (single process)
├── Main thread: Bevy + Rapier
├── Runtime service thread: maintain guest connection / pump
├── Capture thread: fill SharedFrameBuffer
├── SharedFrameBuffer (Arc mutex bytes)  ← virtual display
├── InputCommand queue (Arc mutex)       ← virtual input
└── WorldControl (policy)
```

## Targets

### `cfg(not(target_os = "android"))` — desktop

- `EmbeddedRuntime` uses host tools only as a **backend** to a world-owned guest.
- All buffers and queues are in-process.
- Prefer headless emulator under this process (`VCE_AVD_NAME`).

### `cfg(target_os = "android")` — device

- Physics app is a native Android activity (cargo-apk).
- `android_native` module: JNI entry points for surface/input (stubs ready for binding).
- Termux launched via Android intents from **this** process — no host adb.

## Modules

| Module | Role |
|--------|------|
| `embedded_runtime` | In-process runtime service |
| `shared_buffer` | SharedFrameBuffer + InputQueue |
| `android_native` | Android-target JNI / intent hooks |
| `control` | WorldControl |
| `world` / `display` | Physics entities + texture from shared buffer |
