//! World-owned virtual display and virtual input.
//!
//! These are the only sanctioned I/O devices between the physics reality
//! and the Android inhabitant. No parallel desktop control path.

use bevy::prelude::*;

use crate::android_runtime::AndroidRuntime;

/// World-owned virtual display device (pixels land on InhabitantScreen).
#[derive(Resource)]
pub struct VirtualDisplay {
    pub enabled: bool,
    pub id: &'static str,
}

impl Default for VirtualDisplay {
    fn default() -> Self {
        Self {
            enabled: true,
            id: "world-display-0",
        }
    }
}

/// World-owned virtual input device.
#[derive(Resource)]
pub struct VirtualInput {
    pub enabled: bool,
    pub id: &'static str,
}

impl Default for VirtualInput {
    fn default() -> Self {
        Self {
            enabled: true,
            id: "world-input-0",
        }
    }
}

impl VirtualInput {
    pub fn text(&mut self, s: &str, runtime: &AndroidRuntime) {
        if self.enabled {
            runtime.inject_text(s);
        }
    }

    pub fn key(&mut self, code: i32, runtime: &AndroidRuntime) {
        if self.enabled {
            runtime.inject_keyevent(code);
        }
    }
}

// Registered alongside control plugin from main via init in AndroidRuntimePlugin-less path:
impl Plugin for VirtualIoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VirtualDisplay>()
            .init_resource::<VirtualInput>();
        println!("[VirtualIO] World-owned display + input devices registered");
    }
}

pub struct VirtualIoPlugin;
