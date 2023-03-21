#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}

impl Vertex {
    #[inline(always)]
    pub const fn from_i8(pos: [i8; 3], color: [i8; 3]) -> Self {
        Self {
            position: [pos[0] as f32, pos[0] as f32, pos[0] as f32, 1.0],
            color: [color[0] as f32, color[0] as f32, color[0] as f32, 1.0],
        }
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    // top (0, 0, 1)
    // Vertex::from_i8([-1, -1, 1], [0, 0, 0]),
    // Vertex::from_i8([1, -1, 1], [0, 0, 0]),
    // Vertex::from_i8([1, 1, 1], [0, 0, 0]),
    // Vertex::from_i8([-1, 1, 1], [0, 0, 0]),
    // // top bottom (0, 0, -1)
    // Vertex::from_i8([-1, 1, -1], [0, 0, 0]),
    // Vertex::from_i8([1, 1, -1], [0, 0, 0]),
    // Vertex::from_i8([1, -1, -1], [0, 0, 0]),
    // Vertex::from_i8([-1, -1, -1], [0, 0, 0]),
    // // right (1, 0, 0)
    // Vertex::from_i8([1, -1, -1], [0, 0, 0]),
    // Vertex::from_i8([1, 1, -1], [0, 0, 0]),
    // Vertex::from_i8([1, 1, 1], [0, 0, 0]),
    // Vertex::from_i8([1, -1, 1], [0, 0, 0]),
    // // left (-1, 0, 0)
    // Vertex::from_i8([-1, -1, 1], [0, 0, 0]),
    // Vertex::from_i8([-1, 1, 1], [0, 0, 0]),
    // Vertex::from_i8([-1, 1, -1], [0, 0, 0]),
    // Vertex::from_i8([-1, -1, -1], [0, 0, 0]),
    // // front (0, 1, 0)
    // Vertex::from_i8([1, 1, -1], [0, 0, 0]),
    // Vertex::from_i8([-1, 1, -1], [0, 0, 0]),
    // Vertex::from_i8([-1, 1, 1], [0, 0, 0]),
    // Vertex::from_i8([1, 1, 1], [0, 0, 0]),
    // // back (0, -1, 0)
    // Vertex::from_i8([1, -1, 1], [0, 0, 0]),
    // Vertex::from_i8([-1, -1, 1], [0, 0, 0]),
    // Vertex::from_i8([-1, -1, -1], [0, 0, 0]),
    // Vertex::from_i8([1, -1, -1], [0, 0, 0]),
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0, 1.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 0
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0, 1.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 1
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0, 1.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 2
    Vertex {
        position: [0.35966998, -0.3473291, 0.0, 1.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 3
    Vertex {
        position: [0.44147372, 0.2347359, 0.0, 1.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 4
];

pub const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, 0, 3,
    4,
    // 0, 1, 2, 2, 3, 0, // top
    // 4, 5, 6, 6, 7, 4, // bottom
    // 8, 9, 10, 10, 11, 8, // right
    // 12, 13, 14, 14, 15, 12, // left
    // 16, 17, 18, 18, 19, 16, // front
    // 20, 21, 22, 22, 23, 20, // back
];
