#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}
impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 0
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 1
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 2
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 3
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
    }, // 4
];

// 这里不管是直接调整 VERTICES 中顶点的顺序还是调整 INDICES 的顺序，本质还是调整了送往 GPU 的实际需要绘制的顶点的组合
// 所以结果是一样的
pub const INDICES: &[u16] = &[0, 1, 4, 2, 3];
