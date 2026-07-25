# World Control Plane

`WorldControl` is the physics world’s authority object over the Android inhabitant.

## Gates (enforced in v0)

| Gate | Effect |
|------|--------|
| `environment_powered` | If false: no input, no display update, no Termux launch |
| `input_enabled` | Virtual input ignored |
| `display_enabled` | Framebuffer not applied to screen |

## Commands

`LaunchTermux` · `SetInputEnabled` · `SetDisplayEnabled` · `SetEnvironmentPowered` · `SetTimeScale` · `SetResourceBudget` · `RegisterVirtualDevice` · `ShutdownGuest` · `ShutdownApp`

## Resources & devices (foundation)

Budgets and `virtual_devices` are stored now; enforcement backends come later when virtual hardware is implemented inside the world.
