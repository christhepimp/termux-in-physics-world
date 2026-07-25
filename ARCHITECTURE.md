# Architecture

## Principle

**Physics engine = reality layer.**  
**Android + Termux = operating environment that exists inside that reality.**

Termux is not drawn by a synthetic VT100 alone. The pixels on the world screen come from a **real Android framebuffer** while Termux (and any other Android UI) runs in that Android environment.

## Layers

1. **Physics (Bevy + Rapier3D)**  
   Owns space, colliders, and the display body. Nothing “outside” the world is the primary UI.

2. **Android runtime**  
   Emulator (QEMU-based AVD) or physical device, addressed through ADB.

3. **Termux**  
   Standard open-source Termux installed in that Android environment. Processes, packages, and filesystem are Termux’s own.

4. **Bridge**  
   - Framebuffer: scrcpy video or `adb screencap` → RGBA → `Image` on `TerminalScreen`  
   - Input: Bevy key events → `adb input` / `adb shell input text`  
   - Control: `am start` for `com.termux`

## Data flow

```
Android (Termux UI drawn by Android)
    → screencap / scrcpy
        → FramebufferBridge
            → Bevy Image asset
                → StandardMaterial on TerminalScreen mesh
                    → rendered in physics world

Keyboard
    → AndroidBridge.inject_*
        → adb shell input
            → Android InputManager
                → Termux / focused window
```

## Why not “Termux as a crate”?

Termux is an Android application and bootstrap, not a portable library. Embedding the *real* environment means embedding or attaching a **real Android** that runs the Termux APK. This project does that attachment explicitly and keeps the physics world as the place the user looks at and types into.

## Future (out of v0 scope)

- Physics interactions with the screen chassis
- Multiple Android displays / multi-seat
- Building a minimal Android system image with Termux preinstalled
- Running under native Android (Bevy as a native activity) with Termux as a sibling userspace — different packaging, same principle
