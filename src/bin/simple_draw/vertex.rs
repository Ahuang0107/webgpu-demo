#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexInput {
    pub position: [f32; 2],
    /// uv 的坐标与 png 的坐标方向相同
    ///
    /// `[0,0]` `[1,0]`
    ///
    /// `[0,1]` `[1,1]`
    pub uv: [f32; 2],
}

impl VertexInput {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2,1 => Float32x2];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
