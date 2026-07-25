use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::shared_buffer::FramePixels;
use crate::world::InhabitantScreen;

#[derive(Component)]
pub struct ScreenSurface {
    pub image: Handle<Image>,
}

pub fn setup_screen_material(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<InhabitantScreen>>,
) {
    let w = 720u32;
    let h = 1280u32;
    let handle = images.add(Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![16u8; (w * h * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
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
    }
}

pub fn blit_frame_into_image(image: &mut Image, frame: &FramePixels) {
    if image.width() != frame.width || image.height() != frame.height {
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
    let need = (frame.width * frame.height * 4) as usize;
    if let Some(dst) = image.data.as_mut() {
        if dst.len() >= need && frame.rgba.len() >= need {
            dst[..need].copy_from_slice(&frame.rgba[..need]);
        }
    }
}
