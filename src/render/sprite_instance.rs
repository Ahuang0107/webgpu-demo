use crate::{BlendMode, Color};
use glam::{Affine3A, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    // Affine 4x3 transposed to 3x4
    pub i_model_transpose: [Vec4; 3],
    pub i_uv_offset_scale: Vec4,
    pub color: Vec4,
    pub blend_mode: u32,
    pub _padding: [u32; 3],
}

impl SpriteInstance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        0 => Float32x4,
        1 => Float32x4,
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
        5 => Uint32,
        6 => Uint32x3,
    ];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
    #[inline]
    pub fn from(
        transform: &Affine3A,
        uv_offset_scale: &Vec4,
        color: Color,
        blend_mode: BlendMode,
    ) -> Self {
        let transpose_model_3x3 = transform.matrix3.transpose();
        Self {
            i_model_transpose: [
                transpose_model_3x3.x_axis.extend(transform.translation.x),
                transpose_model_3x3.y_axis.extend(transform.translation.y),
                transpose_model_3x3.z_axis.extend(transform.translation.z),
            ],
            i_uv_offset_scale: *uv_offset_scale,
            color: color.as_vec4(),
            blend_mode: blend_mode as u32,
            _padding: [0, 0, 0],
        }
    }
}
