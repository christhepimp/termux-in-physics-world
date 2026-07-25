//! Physics reality layer — the world Termux is displayed inside.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct TerminalScreen;

#[derive(Component)]
pub struct TerminalChassis;

pub fn spawn_physics_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor
    commands.spawn((
        Name::new("Floor"),
        Mesh3d(meshes.add(Cuboid::new(24.0, 0.2, 24.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.12, 0.13, 0.15))),
        Transform::from_xyz(0.0, -0.1, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(12.0, 0.1, 12.0),
    ));

    // Chassis — physical object that frames the Android/Termux display
    commands.spawn((
        Name::new("TermuxChassis"),
        TerminalChassis,
        Mesh3d(meshes.add(Cuboid::new(1.75, 1.2, 0.1))),
        MeshMaterial3d(materials.add(Color::srgb(0.06, 0.07, 0.08))),
        Transform::from_xyz(0.0, 1.15, -0.05),
        RigidBody::Fixed,
        Collider::cuboid(0.875, 0.6, 0.05),
    ));

    // Screen entity — framebuffer texture attached at startup
    commands.spawn((
        Name::new("TermuxScreen"),
        TerminalScreen,
        Transform::from_xyz(0.0, 1.18, 0.02),
        RigidBody::Fixed,
        Collider::cuboid(0.8, 0.5, 0.01),
    ));

    println!("[Physics] Reality layer online");
    println!("[Physics] Termux display surface is a world entity (not a host window UI)");
}
