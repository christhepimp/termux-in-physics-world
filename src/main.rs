//! Physics engine = reality / world control.
//! Android + Termux = inhabitants. No external control path.

mod android_runtime;
mod control;
mod display;
mod framebuffer;
mod virtual_io;
mod world;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use android_runtime::{AndroidRuntime, AndroidRuntimePlugin};
use control::{WorldCommand, WorldControl, WorldControlPlugin};
use display::setup_screen_material;
use framebuffer::{FramebufferPlugin, FramebufferState};
use virtual_io::{VirtualDisplay, VirtualInput};
use world::{spawn_physics_world, InhabitantScreen};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Physics Reality (Android/Termux Inhabitant)".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(WorldControlPlugin)
        .add_plugins(AndroidRuntimePlugin)
        .add_plugins(FramebufferPlugin)
        .add_systems(
            Startup,
            (setup_camera_light, spawn_physics_world, setup_screen_material).chain(),
        )
        .add_systems(
            Update,
            (
                world_virtual_input_system,
                world_virtual_display_system,
                control::process_world_commands,
            ),
        )
        .run();
}

fn setup_camera_light(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.35, 2.8).looking_at(Vec3::new(0.0, 1.15, 0.0), Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.65, 0.7, 0.75),
        brightness: 0.4,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(2.5, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    println!("[World] Physics engine is the reality boundary");
    println!("[World] Android/Termux have no separate window or control path");
}

/// Virtual input path: user → physics window → world → guest only.
fn world_virtual_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventReader<KeyboardInput>,
    mut control: ResMut<WorldControl>,
    mut vinput: ResMut<VirtualInput>,
    runtime: Res<AndroidRuntime>,
) {
    if keys.just_pressed(KeyCode::KeyQ) && keys.pressed(KeyCode::ControlLeft) {
        control.enqueue(WorldCommand::ShutdownApp);
        return;
    }
    if keys.just_pressed(KeyCode::KeyT) && keys.pressed(KeyCode::ControlLeft) {
        control.enqueue(WorldCommand::LaunchTermux);
        return;
    }

    if !control.allows_input() {
        return;
    }

    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Character(c) => vinput.text(c, &runtime),
            Key::Space => vinput.text(" ", &runtime),
            Key::Enter => vinput.key(66, &runtime),
            Key::Backspace => vinput.key(67, &runtime),
            Key::Tab => vinput.key(61, &runtime),
            Key::Escape => vinput.key(111, &runtime),
            Key::ArrowUp => vinput.key(19, &runtime),
            Key::ArrowDown => vinput.key(20, &runtime),
            Key::ArrowLeft => vinput.key(21, &runtime),
            Key::ArrowRight => vinput.key(22, &runtime),
            _ => {}
        }
    }
}

/// Virtual display path: guest framebuffer → world-owned texture on physics entity.
fn world_virtual_display_system(
    control: Res<WorldControl>,
    fb: Res<FramebufferState>,
    vdisp: Res<VirtualDisplay>,
    mut images: ResMut<Assets<Image>>,
    query: Query<&display::ScreenSurface, With<InhabitantScreen>>,
) {
    if !control.allows_display() || !vdisp.enabled {
        return;
    }
    let Some(frame) = fb.latest_frame() else {
        return;
    };
    for surface in query.iter() {
        if let Some(image) = images.get_mut(&surface.image) {
            display::blit_frame_into_image(image, &frame);
        }
    }
}
