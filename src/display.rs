//! In-world screen: Bevy Image attached to the physics TerminalScreen entity.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::framebuffer::FramePixels;
use crate::world::TerminalScreen;

#[derive(Component)]
pub struct ScreenSurface {
    pub image: Handle<Image>,
}

pub fn setup_screen_material(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<TerminalScreen>>,
) {
    // Placeholder resolution; replaced dynamically as frames arrive.
    let w = 720u32;
    let h = 1280u32;
    let data = vec![20u8; (w * h * 4) as usize];
    let image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let handle = images.add(image);

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(handle.clone()),
        unlit: true,
        ..default()
    });

    if let Ok(entity) = query.get_single() {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Rectangle::new(1.6, 0.95))),
            MeshMaterial3d(material),
            ScreenSurface { image: handle },
        ));
        println!("[Display] Screen texture bound to physics TerminalScreen entity");
    }
}

/// Copy captured Android pixels into the GPU image (nearest fit / crop center).
pub fn blit_frame_into_image(image: &mut Image, frame: &FramePixels) {
    let dst_w = image.width() as usize;
    let dst_h = image.height() as usize;
    let Some(dst) = image.data.as_mut() else {
        return;
    };

    // If size differs a lot, resize image asset once
    if dst_w != frame.width as usize || dst_h != frame.height as usize {
        // Recreate dimensions in-place when possible
        *image = Image::new(
            Extent3d {
                width: frame.width.max(1),
                height: frame.height.max(1),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            frame.rgba.clone(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        return;
    }

    let need = dst_w * dst_h * 4;
    if dst.len() < need || frame.rgba.len() < need {
        return;
    }
    dst[..need].copy_from_slice(&frame.rgba[..need]);
}
