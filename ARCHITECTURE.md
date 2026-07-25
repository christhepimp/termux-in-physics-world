# Architecture

## Principle

The **physics engine is the reality layer**. The operating environment (Termux when available, otherwise a real host shell) runs as a process whose **only connection** to the user-visible world is through entities that exist inside that physics simulation.

There is no parallel “desktop terminal window” that is the real UI. The UI is a textured mesh on a `TerminalScreen` body in the Rapier world.

## Components

| Module | Role |
|--------|------|
| `world` | Floor, chassis, screen — all Rapier entities |
| `session` | PTY + child process (Termux or host shell) |
| `display` | Byte stream → character grid → GPU texture |
| `main` | Input → PTY; PTY → display; physics step |

## Data flow

```
Keyboard (Bevy input)
    → SessionBridge.write (PTY master)
        → child process (Termux / bash)
            → PTY slave stdout
                → SessionOutput buffer
                    → TerminalDisplay.feed_bytes
                        → Image texture on TerminalScreen mesh
                            → rendered inside physics world
```

## Termux detection

1. `TERMUX_VERSION` + `$PREFIX/bin/bash|login`
2. `/data/data/com.termux/files/usr/bin/bash`
3. Else host `$SHELL` / `bash` with the **same** bridge

## Next experiments (not in v0)

- Grab/move the chassis with physics
- Multiple sessions on multiple screens
- Bridge to Termux over ADB/SSH when developing on desktop
- Proper bitmap font / VT100 emulator (e.g. alacritty_terminal)
