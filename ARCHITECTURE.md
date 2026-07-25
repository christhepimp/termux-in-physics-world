# Architecture

## Principle

Android and Termux are **inhabitants** of a physics-engine-controlled reality. They are not an external system that the engine “watches.”

- The **only** interactive window is the physics application.
- The Android runtime is started or adopted as a **world-managed process** (headless).
- Perception (display) and action (input) are **virtual devices owned by the world**.
- `WorldControl` is the authority for what the inhabitant may experience.

## Hierarchy

```
Physics Engine (Reality / World Control)
├── Spatial world (Rapier bodies: floor, chassis, screen)
├── VirtualDisplay (world-owned)
├── VirtualInput (world-owned)
├── WorldControl (power, time, resources, virtual devices, commands)
└── AndroidRuntime (headless guest process / adb serial)
      └── Termux (real app inside Android)
```

## Boundary rule

```
User → Physics window → World systems → VirtualDisplay / VirtualInput / WorldControl
                                              ↓
                                    AndroidRuntime (headless)
                                              ↓
                                           Termux
```

There is no first-class path: User → Emulator window → Termux.

## Headless embedding

When `VCE_AVD_NAME` is set, the world attempts:

```text
emulator -avd <name> -no-window -no-audio -no-boot-anim -gpu swiftshader_indirect
```

so the guest does not present a parallel desktop UI. Frames enter the world only through the capture path onto the `InhabitantScreen` entity.

## Future

- Virtual CPU/RAM/storage/sensors registered in `WorldControl.virtual_devices` and backed by world systems
- Resource budgets enforced on the runtime process
- Time scale affecting guest clock
- World events (power loss) issuing `WorldCommand`s automatically
