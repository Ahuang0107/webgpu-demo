#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    pub color: [f32; 4],
    tex_coords: [f32; 2],
    pub blend_mode: u32,
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4,2 => Float32x2, 3 => Uint32];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
    pub fn new(x: f32, y: f32, tx: f32, ty: f32) -> Self {
        Self {
            position: [x, y],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_coords: [tx, ty],
            blend_mode: 0,
        }
    }
}
