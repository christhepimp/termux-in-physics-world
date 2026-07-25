# Control Plane Foundation

`WorldControl` is the physics world's authority object. v0 implements **structure and gates**, not full enforcement backends.

## Fields (conceptual)

| Field | v0 behavior | Future |
|-------|-------------|--------|
| `environment_powered` | Must be true for input/launch | Tie to in-world power entities |
| `input_enabled` | Gate keyboard injection | Disable when world rules say so |
| `display_enabled` | Gate applying framebuffer | Blank screen under world policy |
| `time_scale` | Stored (1.0) | Drive guest time dilation |
| `cpu_budget` / `ram_budget` / … | Placeholders | Enforce via emulator hooks / cgroups |
| `virtual_devices` | Empty list | Register devices Android will see |
| `commands` | Queue of `WorldCommand` | Scripted world→OE control |

## WorldCommand (extensible)

```text
LaunchTermux
SetInputEnabled(bool)
SetDisplayEnabled(bool)
SetTimeScale(f32)
SetResourceBudget { kind, limit }
RegisterVirtualDevice { id, class }
ShutdownGuest
```

v0 executes a subset (launch, input/display flags). Remaining variants log and reserve behavior.

## Path to “physics defines what Termux can do”

1. **v0** — Mediate I/O; hold policy state  
2. **v1** — Enforce budgets on emulator process; pause capture when unpowered  
3. **v2** — Virtual devices (block, net) implemented in-world, exposed to Android  
4. **v3** — World events (damage, power loss) automatically issue `WorldCommand`s  
