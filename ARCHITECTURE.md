# Architecture

## Principle

**The physics engine defines the rules of existence.**

Android and Termux do not sit beside the simulation. They inhabit it. All observation and interaction is mediated by world systems so that, over time, the physics layer can constrain or extend what the operating environment is allowed to be.

## Layers

```
Physics Engine (Bevy + Rapier3D)
├── Spatial reality (bodies, colliders, transforms)
├── Display inhabitant surface (TerminalScreen entity)
├── WorldControl (authority / policy)
├── AndroidBridge (world-owned handle to guest OE)
└── Framebuffer capture (world-owned sensory path)
        │
        ▼
Android runtime (emulator or device)
└── Termux (com.termux)
```

## Interaction rule

Application code must **not** talk to `adb` ad hoc. It goes through:

1. Bevy systems in the physics app
2. `WorldControl` gates (allowed / denied)
3. `AndroidBridge` / framebuffer modules

That keeps a single choke point for future resource, time, and device control.

## Data paths (v0)

**Perception (Android → world)**  
Android framebuffer → capture thread → `FramebufferState` → texture on `TerminalScreen` mesh.

**Action (world → Android)**  
Keyboard events → `world_input_system` → `WorldControl.allows_input` → `AndroidBridge.inject_*`.

**Lifecycle**  
Startup / Ctrl+T → `WorldControl` → launch Termux intent on the guest.

## Future control surface

| Domain | World owns | Guest sees |
|--------|------------|------------|
| Hardware | Virtual device models in the world | Drivers / sysfs / binder (later) |
| Resources | Budgets in `WorldControl` | cgroups / limits (later) |
| Time | `time_scale`, world tick | guest clock skew (later) |
| Network | Policy + virtual NIC (later) | packets only if world allows |
| Storage | Virtual block backed by world (later) | block device / file |

## Non-goals (v0)

- Reimplementing Android or Termux
- Gameplay interactions with the chassis
- Convincing Android that fictional PCI devices exist (hook points only)
