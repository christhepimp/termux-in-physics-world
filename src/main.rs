//! Physics engine process hosts the embedded OE runtime in-process.

mod android_native;
mod control;
mod display;
mod embedded_runtime;
mod shared_buffer;
mod world;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use control::{WorldCommand, WorldControl, WorldControlPlugin};
use display::setup_screen_material;
use embedded_runtime::{EmbeddedRuntime, EmbeddedRuntimePlugin};
use shared_buffer::{InputQueue, SharedFrameBuffer};
use world::{spawn_physics_world, InhabitantScreen};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Physics Process — Embedded Android/Termux".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(WorldControlPlugin)
        .add_plugins(EmbeddedRuntimePlugin)
        .add_systems(
            Startup,
            (setup_camera_light, spawn_physics_world, setup_screen_material).chain(),
        )
        .add_systems(
            Update,
            (
                in_process_input_system,
                in_process_display_system,
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
    println!("[Process] Physics engine process online — OE runtime is in-process");
}

fn in_process_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventReader<KeyboardInput>,
    mut control: ResMut<WorldControl>,
    input_q: Res<InputQueue>,
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
            Key::Character(c) => input_q.push_text(c),
            Key::Space => input_q.push_text(" "),
            Key::Enter => input_q.push_key(66),
            Key::Backspace => input_q.push_key(67),
            Key::Tab => input_q.push_key(61),
            Key::Escape => input_q.push_key(111),
            Key::ArrowUp => input_q.push_key(19),
            Key::ArrowDown => input_q.push_key(20),
            Key::ArrowLeft => input_q.push_key(21),
            Key::ArrowRight => input_q.push_key(22),
            _ => {}
        }
    }
}

fn in_process_display_system(
    control: Res<WorldControl>,
    frames: Res<SharedFrameBuffer>,
    mut images: ResMut<Assets<Image>>,
    query: Query<&display::ScreenSurface, With<InhabitantScreen>>,
) {
    if !control.allows_display() {
        return;
    }
    let Some(frame) = frames.latest() else {
        return;
    };
    for surface in query.iter() {
        if let Some(image) = images.get_mut(&surface.image) {
            display::blit_frame_into_image(image, &frame);
        }
    }
}
