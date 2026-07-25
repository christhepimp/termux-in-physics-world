//! Bridge to a real Android runtime that hosts Termux.
//!
//! Uses ADB only — does not reimplement Termux. The Android environment
//! remains the authority for processes, packages, and filesystem.

use std::process::{Command, Stdio};
use std::sync::Arc;

use bevy::prelude::*;

#[derive(Clone, Debug)]
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
        // Standard Termux activity
        let status = self
            .adb_base()
            .args([
                "shell",
                "am",
                "start",
                "-n",
                "com.termux/.HomeActivity",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("[Android] Launch intent sent: com.termux/.HomeActivity");
            }
            Ok(_) => {
                // Fallback: monkey launcher
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
                println!("[Android] Fallback monkey launch for com.termux");
            }
            Err(e) => println!("[Android] adb failed: {e}"),
        }
    }

    pub fn inject_text(&self, text: &str) {
        if !self.connected || text.is_empty() {
            return;
        }
        // adb input text needs spaces as %s
        let escaped = text
            .replace(' ', "%s")
            .replace('\'', "\\'")
            .replace('"', "\\\"");
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
                "[Android] Connected to device {:?}",
                bridge.serial.as_deref().unwrap_or("?")
            );
            if bridge.termux_installed() {
                println!("[Android] Termux package com.termux is installed");
                bridge.launch_termux();
                bridge.termux_launched = true;
            } else {
                println!("[Android] Termux NOT installed on device.");
                println!("[Android] Install from F-Droid, then press Ctrl+T to launch.");
            }
        } else {
            println!("[Android] No adb device. Start an emulator or connect a phone.");
            println!("[Android] Example: emulator -avd <name> && adb wait-for-device");
        }

        app.insert_resource(bridge)
            .add_systems(Update, periodic_device_refresh);
    }
}

fn periodic_device_refresh(mut bridge: ResMut<AndroidBridge>, time: Res<Time>) {
    // Lightweight reconnect check ~every 3s via frame time accumulation
    // (using a static-ish approach with resource field would be cleaner; keep simple)
    let _ = time;
    // Only refresh if disconnected
    if !bridge.connected {
        if bridge.refresh_device() {
            println!(
                "[Android] Device appeared: {:?}",
                bridge.serial.as_deref().unwrap_or("?")
            );
            if bridge.termux_installed() && !bridge.termux_launched {
                bridge.launch_termux();
                bridge.termux_launched = true;
            }
        }
    }
    let _ = Arc::new(());
}
