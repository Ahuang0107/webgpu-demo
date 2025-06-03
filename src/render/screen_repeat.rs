use crate::Color;
use glam::{Vec2, Vec4};

#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenRepeat {
    pub texture_id: u32,
    pub offset: Vec2,
    pub scale: Vec2,
    pub color: Color,
}

impl ScreenRepeat {
    #[inline(always)]
    pub fn get_uniform(&self) -> ScreenRepeatUniform {
        ScreenRepeatUniform {
            uv_offset_scale: self.offset.extend(self.scale.x).extend(self.scale.y),
            color: self.color.as_vec4(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct ScreenRepeatUniform {
    uv_offset_scale: Vec4,
    color: Vec4,
}
