use crate::{BlendMode, Color, Sprite, Transform};
use glam::{Vec2, Vec3};
use isometric_engine::{MetaModel, Scene};
use std::collections::HashMap;

pub fn collect_sprites(scene: &Scene, image_map: &HashMap<MetaModel, u32>) -> Vec<Sprite> {
    let collect_sprites = scene.collect_sprites();
    let collect_sprite_masks = scene.collect_sprite_masks();
    let mut sprites: Vec<Sprite> =
        Vec::with_capacity(collect_sprites.len() + collect_sprite_masks.len());
    for sprite_pin in collect_sprites {
        if let Some(image_handle) = image_map.get(&sprite_pin.meta) {
            let [x, y, z] = sprite_pin.get_xyz();
            let blend_mode = sprite_pin.blend_mode();
            sprites.push(Sprite {
                transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                texture_id: *image_handle,
                anchor: Vec2::new(-0.5, -0.5),
                color: Color::new([255, 255, 255, sprite_pin.get_opacity()]),
                blend_mode: match blend_mode {
                    isometric_engine::BlendMode::Normal => BlendMode::Normal,
                    isometric_engine::BlendMode::Multiply => BlendMode::Multiply,
                    isometric_engine::BlendMode::Overlay => BlendMode::Overlay,
                    isometric_engine::BlendMode::SoftLight => BlendMode::SoftLight,
                    isometric_engine::BlendMode::HardLight => BlendMode::HardLight,
                },
                ..Default::default()
            });
        }
    }

    for sprite_mask in collect_sprite_masks {
        if let Some(image_handle) = image_map.get(&sprite_mask.meta) {
            let [x, y] = sprite_mask.get_acl_offset();
            let [mask_start, mask_end] = sprite_mask.get_range();
            sprites.push(Sprite {
                transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0)),
                texture_id: *image_handle,
                anchor: Vec2::new(-0.5, -0.5),
                mask: Some([mask_start as f32, mask_end as f32]),
                ..Default::default()
            });
        }
    }

    sprites
}
