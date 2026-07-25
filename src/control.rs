//! WorldControl — physics world's authority over the guest environment.
//!
//! v0: policy state + command queue + gates for input/display/launch.
//! Later: resource enforcement, time, virtual devices.

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::android_bridge::AndroidBridge;

#[derive(Clone, Debug)]
pub enum ResourceKind {
    Cpu,
    Ram,
    Storage,
    Network,
}

#[derive(Clone, Debug)]
pub enum WorldCommand {
    LaunchTermux,
    SetInputEnabled(bool),
    SetDisplayEnabled(bool),
    SetEnvironmentPowered(bool),
    SetTimeScale(f32),
    SetResourceBudget { kind: ResourceKind, limit: f32 },
    RegisterVirtualDevice { id: String, class: String },
    ShutdownGuest,
    ShutdownApp,
}

#[derive(Clone, Debug)]
pub struct VirtualDeviceSpec {
    pub id: String,
    pub class: String,
}

/// Single control-plane resource owned by the physics world.
#[derive(Resource)]
pub struct WorldControl {
    pub environment_powered: bool,
    pub input_enabled: bool,
    pub display_enabled: bool,
    /// 1.0 = real-time. Future: dilate guest time.
    pub time_scale: f32,
    pub cpu_budget: f32,
    pub ram_budget_mb: f32,
    pub storage_budget_mb: f32,
    pub network_budget_kbps: f32,
    pub virtual_devices: Vec<VirtualDeviceSpec>,
    commands: VecDeque<WorldCommand>,
}

impl Default for WorldControl {
    fn default() -> Self {
        Self {
            environment_powered: true,
            input_enabled: true,
            display_enabled: true,
            time_scale: 1.0,
            cpu_budget: 1.0,
            ram_budget_mb: 4096.0,
            storage_budget_mb: 64_000.0,
            network_budget_kbps: 100_000.0,
            virtual_devices: Vec::new(),
            commands: VecDeque::new(),
        }
    }
}

impl WorldControl {
    pub fn enqueue(&mut self, cmd: WorldCommand) {
        println!("[WorldControl] queue {cmd:?}");
        self.commands.push_back(cmd);
    }

    pub fn allows_input(&self) -> bool {
        self.environment_powered && self.input_enabled
    }

    pub fn allows_display(&self) -> bool {
        self.environment_powered && self.display_enabled
    }
}

pub struct WorldControlPlugin;

impl Plugin for WorldControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldControl>();
        println!("[WorldControl] Control plane online (physics authority)");
    }
}

pub fn process_world_commands(
    mut control: ResMut<WorldControl>,
    bridge: Res<AndroidBridge>,
) {
    while let Some(cmd) = control.commands.pop_front() {
        match cmd {
            WorldCommand::LaunchTermux => {
                if control.environment_powered {
                    bridge.launch_termux();
                } else {
                    println!("[WorldControl] LaunchTermux denied — environment unpowered");
                }
            }
            WorldCommand::SetInputEnabled(v) => {
                control.input_enabled = v;
                println!("[WorldControl] input_enabled={v}");
            }
            WorldCommand::SetDisplayEnabled(v) => {
                control.display_enabled = v;
                println!("[WorldControl] display_enabled={v}");
            }
            WorldCommand::SetEnvironmentPowered(v) => {
                control.environment_powered = v;
                println!("[WorldControl] environment_powered={v}");
            }
            WorldCommand::SetTimeScale(s) => {
                control.time_scale = s.max(0.0);
                println!(
                    "[WorldControl] time_scale={} (guest enforcement later)",
                    control.time_scale
                );
            }
            WorldCommand::SetResourceBudget { kind, limit } => {
                match kind {
                    ResourceKind::Cpu => control.cpu_budget = limit,
                    ResourceKind::Ram => control.ram_budget_mb = limit,
                    ResourceKind::Storage => control.storage_budget_mb = limit,
                    ResourceKind::Network => control.network_budget_kbps = limit,
                }
                println!("[WorldControl] budget {kind:?}={limit} (enforcement later)");
            }
            WorldCommand::RegisterVirtualDevice { id, class } => {
                println!("[WorldControl] register virtual device id={id} class={class}");
                control.virtual_devices.push(VirtualDeviceSpec { id, class });
            }
            WorldCommand::ShutdownGuest => {
                println!("[WorldControl] ShutdownGuest requested (adb reboot/poweroff later)");
                // Future: adb reboot -p / emulator kill under world policy
            }
            WorldCommand::ShutdownApp => {
                println!("[WorldControl] Shutting down physics app");
                std::process::exit(0);
            }
        }
    }
}
