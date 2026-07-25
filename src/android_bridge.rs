//! World-owned handle to the Android inhabitant.
//!
//! Not a parallel UI — only callable from world systems / WorldControl.

use std::process::{Command, Stdio};

use bevy::prelude::*;

#[derive(Clone, Debug, Resource)]
pub struct AndroidBridge {
    pub serial: Option<String>,
    pub connected: bool,
    pub termux_launched: bool,
}

impl Default for AndroidBridge {
    fn default() -> Self {
        Self {
            serial: None,
            connected: false,
            termux_launched: false,
        }
    }
}

impl AndroidBridge {
    fn adb_base(&self) -> Command {
        let mut cmd = Command::new("adb");
        if let Some(serial) = &self.serial {
            cmd.arg("-s").arg(serial);
        }
        cmd
    }

    pub fn refresh_device(&mut self) -> bool {
        let output = Command::new("adb").arg("devices").output();
        let Ok(output) = output else {
            self.connected = false;
            self.serial = None;
            return false;
        };
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
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

    pub fn launch_termux(&self) {
        if !self.connected {
            println!("[Android] No device — cannot launch Termux");
            return;
        }
        let status = self
            .adb_base()
            .args(["shell", "am", "start", "-n", "com.termux/.HomeActivity"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("[Android] Termux HomeActivity start requested");
            }
            _ => {
                let _ = self
                    .adb_base()
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
                println!("[Android] Termux launch via monkey fallback");
            }
        }
    }

    pub fn inject_text(&self, text: &str) {
        if !self.connected || text.is_empty() {
            return;
        }
        let escaped = text.replace(' ', "%s");
        let _ = self
            .adb_base()
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
            .adb_base()
            .args(["shell", "input", "keyevent", &code.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    pub fn termux_installed(&self) -> bool {
        if !self.connected {
            return false;
        }
        let output = self
            .adb_base()
            .args(["shell", "pm", "path", "com.termux"])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains("package:"),
            Err(_) => false,
        }
    }
}

pub struct AndroidPlugin;

impl Plugin for AndroidPlugin {
    fn build(&self, app: &mut App) {
        let mut bridge = AndroidBridge::default();
        if bridge.refresh_device() {
            println!(
                "[Android] Inhabitant runtime connected: {:?}",
                bridge.serial.as_deref().unwrap_or("?")
            );
            if bridge.termux_installed() {
                println!("[Android] Termux package present — requesting launch via world rules");
                bridge.launch_termux();
                bridge.termux_launched = true;
            } else {
                println!("[Android] Install Termux on the device, then Ctrl+T");
            }
        } else {
            println!("[Android] No adb device — start emulator or connect hardware");
        }

        app.insert_resource(bridge)
            .add_systems(Update, reconnect_if_needed);
    }
}

fn reconnect_if_needed(mut bridge: ResMut<AndroidBridge>) {
    if !bridge.connected && bridge.refresh_device() {
        println!(
            "[Android] Device online: {:?}",
            bridge.serial.as_deref().unwrap_or("?")
        );
        if bridge.termux_installed() && !bridge.termux_launched {
            bridge.launch_termux();
            bridge.termux_launched = true;
        }
    }
}
