use crate::assets::AssetsId;
use crate::{BlendMode, Color, Rect, Transform};
use glam::{Affine3A, Quat, Vec2, Vec4};

#[derive(Copy, Clone, Debug, Default)]
pub struct Sprite {
    pub transform: Transform,
    pub texture_id: AssetsId,
    /// Select an area of the texture
    pub rect: Option<Rect>,
    /// Change the on-screen size of the sprite
    pub custom_size: Option<Vec2>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor: Vec2,
    /// Mask range
    pub mask: Option<[f32; 2]>,
    /// 这里其实有点冲突，本来 color 还承担着 opacity 的作用
    /// 但是如果是特殊的 color blend mode 模式，则图层本身的 opacity 和 color 的 opacity 没办法兼容
    pub color: Color,
    pub color_blend_mode: BlendMode,
    pub blend_mode: BlendMode,
}

impl Sprite {
    #[inline(always)]
    pub fn calculate_transform(&self, image_size: Vec2) -> Affine3A {
        let quad_size = self
            .custom_size
            .unwrap_or_else(|| self.rect.map(|r| r.size()).unwrap_or(image_size));

        self.transform.compute_affine()
            * Affine3A::from_scale_rotation_translation(
                quad_size.extend(1.0),
                Quat::IDENTITY,
                (quad_size * (-self.anchor - Vec2::splat(0.5))).extend(0.0),
            )
    }
    #[inline(always)]
    pub fn calculate_uv_offset_scale(&self, image_size: Vec2) -> Vec4 {
        let mut uv_offset_scale: Vec4;

        // If a rect is specified, adjust UVs and the size of the quad
        if let Some(rect) = self.rect {
            let rect_size = rect.size();
            uv_offset_scale = Vec4::new(
                rect.min.x / image_size.x,
                rect.max.y / image_size.y,
                rect_size.x / image_size.x,
                -rect_size.y / image_size.y,
            );
        } else {
            uv_offset_scale = Vec4::new(0.0, 1.0, 1.0, -1.0);
        }

        if self.flip_x {
            uv_offset_scale.x += uv_offset_scale.z;
            uv_offset_scale.z *= -1.0;
        }
        if self.flip_y {
            uv_offset_scale.y += uv_offset_scale.w;
            uv_offset_scale.w *= -1.0;
        }

        uv_offset_scale
    }
}
