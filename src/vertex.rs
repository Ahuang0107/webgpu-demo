#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    }, // 0
    Vertex {
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    }, // 1
    Vertex {
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    }, // 2
    Vertex {
        position: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    }, // 3
];

pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
