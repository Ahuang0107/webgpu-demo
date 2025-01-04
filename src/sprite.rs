use crate::blend_mode::BlendMode;
use crate::vertex::Vertex;

pub struct Sprite {
    pub vertices: [Vertex; 4],
    pub indices: [u16; 6],
    pub texture_id: u32,
    pub blend_mode: BlendMode,
    pub if_mask: bool,
}

pub struct RawSprite {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub texture_id: u32,
    pub blend_mode: BlendMode,
    pub if_mask: bool,
}
