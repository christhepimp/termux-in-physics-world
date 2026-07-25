//! WorldControl — sole authority over what the inhabitant environment experiences.

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::android_runtime::AndroidRuntime;

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

#[derive(Resource)]
pub struct WorldControl {
    pub environment_powered: bool,
    pub input_enabled: bool,
    pub display_enabled: bool,
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
        println!("[WorldControl] {cmd:?}");
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
        // Seed virtual display/input device records as world-owned.
        app.world_mut()
            .resource_mut::<WorldControl>()
            .virtual_devices
            .push(VirtualDeviceSpec {
                id: "world-display-0".into(),
                class: "virtual_display".into(),
            });
        app.world_mut()
            .resource_mut::<WorldControl>()
            .virtual_devices
            .push(VirtualDeviceSpec {
                id: "world-input-0".into(),
                class: "virtual_input".into(),
            });
        println!("[WorldControl] Authority online");
    }
}

pub fn process_world_commands(
    mut control: ResMut<WorldControl>,
    runtime: Res<AndroidRuntime>,
) {
    while let Some(cmd) = control.commands.pop_front() {
        match cmd {
            WorldCommand::LaunchTermux => {
                if control.environment_powered {
                    runtime.launch_termux();
                } else {
                    println!("[WorldControl] LaunchTermux denied (unpowered)");
                }
            }
            WorldCommand::SetInputEnabled(v) => control.input_enabled = v,
            WorldCommand::SetDisplayEnabled(v) => control.display_enabled = v,
            WorldCommand::SetEnvironmentPowered(v) => control.environment_powered = v,
            WorldCommand::SetTimeScale(s) => {
                control.time_scale = s.max(0.0);
                println!("[WorldControl] time_scale={} (guest bind later)", control.time_scale);
            }
            WorldCommand::SetResourceBudget { kind, limit } => {
                match kind {
                    ResourceKind::Cpu => control.cpu_budget = limit,
                    ResourceKind::Ram => control.ram_budget_mb = limit,
                    ResourceKind::Storage => control.storage_budget_mb = limit,
                    ResourceKind::Network => control.network_budget_kbps = limit,
                }
                println!("[WorldControl] budget {kind:?}={limit}");
            }
            WorldCommand::RegisterVirtualDevice { id, class } => {
                control.virtual_devices.push(VirtualDeviceSpec { id, class });
            }
            WorldCommand::ShutdownGuest => {
                runtime.shutdown_guest();
            }
            WorldCommand::ShutdownApp => {
                runtime.shutdown_guest();
                std::process::exit(0);
            }
        }
    }
}
