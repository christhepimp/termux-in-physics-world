//! World-managed Android runtime (headless inhabitant process).
//!
//! The runtime is not an external product UI. Prefer headless emulator
//! started by the world, or attach to an existing adb serial that is
//! already headless. All user-facing I/O still goes through Virtual*.

use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use bevy::prelude::*;

#[derive(Resource)]
pub struct AndroidRuntime {
    pub serial: Option<String>,
    pub connected: bool,
    pub termux_launched: bool,
    /// Child emulator process if the world spawned it.
    child: Mutex<Option<Child>>,
    pub owned_by_world: bool,
}

impl Default for AndroidRuntime {
    fn default() -> Self {
        Self {
            serial: None,
            connected: false,
            termux_launched: false,
            child: Mutex::new(None),
            owned_by_world: false,
        }
    }
}

impl AndroidRuntime {
    fn adb(&self) -> Command {
        let mut cmd = Command::new("adb");
        if let Some(serial) = &self.serial {
            cmd.arg("-s").arg(serial);
        }
        cmd
    }

    pub fn refresh(&mut self) -> bool {
        let output = Command::new("adb").arg("devices").output();
        let Ok(output) = output else {
            self.connected = false;
            self.serial = None;
            return false;
        };
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines().skip(1) {
            let mut parts = line.split_whitespace();
            if let (Some(serial), Some(state)) = (parts.next(), parts.next()) {
                if state == "device" {
                    self.serial = Some(serial.to_string());
                    self.connected = true;
                    return true;
                }
            }
        }
        self.connected = false;
        self.serial = None;
        false
    }

    /// Spawn headless emulator under world ownership when VCE_AVD_NAME is set.
    pub fn try_spawn_headless_avd(&mut self) {
        if self.connected {
            return;
        }
        let Ok(avd) = std::env::var("VCE_AVD_NAME") else {
            println!("[AndroidRuntime] Set VCE_AVD_NAME to let the world spawn a headless AVD");
            return;
        };

        println!("[AndroidRuntime] Spawning headless AVD '{avd}' as world inhabitant...");
        let spawn = Command::new("emulator")
            .args([
                "-avd",
                &avd,
                "-no-window",
                "-no-audio",
                "-no-boot-anim",
                "-gpu",
                "swiftshader_indirect",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match spawn {
            Ok(child) => {
                self.owned_by_world = true;
                *self.child.lock().unwrap() = Some(child);
                println!("[AndroidRuntime] Headless emulator started (world-owned process)");
                // Best-effort wait
                let _ = Command::new("adb")
                    .args(["wait-for-device"])
                    .status();
                let _ = self.refresh();
            }
            Err(e) => {
                println!("[AndroidRuntime] Failed to spawn emulator: {e}");
                println!("[AndroidRuntime] Start headless yourself: emulator -avd {avd} -no-window");
            }
        }
    }

    pub fn launch_termux(&self) {
        if !self.connected {
            println!("[AndroidRuntime] Not connected");
            return;
        }
        let ok = self
            .adb()
            .args(["shell", "am", "start", "-n", "com.termux/.HomeActivity"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            let _ = self
                .adb()
                .args([
                    "shell",
                    "monkey",
                    "-p",
                    "com.termux",
                    "-c",
                    "android.intent.category.LAUNCHER",
                    "1",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        println!("[AndroidRuntime] Termux launch requested inside inhabitant Android");
    }

    pub fn termux_installed(&self) -> bool {
        if !self.connected {
            return false;
        }
        self.adb()
            .args(["shell", "pm", "path", "com.termux"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("package:"))
            .unwrap_or(false)
    }

    pub fn inject_text(&self, text: &str) {
        if !self.connected || text.is_empty() {
            return;
        }
        let escaped = text.replace(' ', "%s");
        let _ = self
            .adb()
            .args(["shell", "input", "text", &escaped])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    pub fn inject_keyevent(&self, code: i32) {
        if !self.connected {
            return;
        }
        let _ = self
            .adb()
            .args(["shell", "input", "keyevent", &code.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    pub fn shutdown_guest(&self) {
        if self.owned_by_world {
            if let Ok(mut guard) = self.child.lock() {
                if let Some(mut child) = guard.take() {
                    let _ = child.kill();
                    println!("[AndroidRuntime] World-owned emulator process killed");
                }
            }
        } else if self.connected {
            // Do not force-reboot user devices by default; log only.
            println!("[AndroidRuntime] Guest not world-owned — leave process running");
        }
    }
}

pub struct AndroidRuntimePlugin;

impl Plugin for AndroidRuntimePlugin {
    fn build(&self, app: &mut App) {
        let mut runtime = AndroidRuntime::default();

        if !runtime.refresh() {
            runtime.try_spawn_headless_avd();
            let _ = runtime.refresh();
        } else {
            println!(
                "[AndroidRuntime] Attached to existing adb device {:?}",
                runtime.serial
            );
            println!(
                "[AndroidRuntime] Prefer headless (-no-window) so the physics app is the only UI"
            );
        }

        if runtime.connected {
            if runtime.termux_installed() {
                runtime.launch_termux();
                runtime.termux_launched = true;
            } else {
                println!("[AndroidRuntime] Install Termux in the guest, then Ctrl+T");
            }
        }

        app.insert_resource(runtime)
            .add_systems(Update, runtime_maintain);
    }
}

fn runtime_maintain(mut runtime: ResMut<AndroidRuntime>) {
    if !runtime.connected {
        let _ = runtime.refresh();
        if runtime.connected && runtime.termux_installed() && !runtime.termux_launched {
            runtime.launch_termux();
            runtime.termux_launched = true;
        }
    }
}
