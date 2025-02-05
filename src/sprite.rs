use crate::blend_mode::BlendMode;
use crate::vertex::Vertex;

pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [u8; 4],
    pub texture_id: u32,
    pub blend_mode: BlendMode,
    pub mask_in: bool,
    pub mask_out: bool,
    pub window_size: [u32; 2],
}

impl Sprite {
    pub fn new(pos: [f32; 2], size: [f32; 2], texture_id: u32) -> Self {
        Sprite {
            x: pos[0],
            y: pos[1],
            width: size[0],
            height: size[1],
            color: [255; 4],
            texture_id,
            blend_mode: BlendMode::Normal,
            mask_in: false,
            mask_out: false,
            window_size: [0, 0],
        }
    }
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
    pub fn with_opacity(mut self, opacity: u8) -> Self {
        self.color[3] = opacity;
        self
    }
    pub fn with_window_size(mut self, window_size: [u32; 2]) -> Self {
        self.window_size = window_size;
        self
    }
    pub fn with_mask_in(mut self) -> Self {
        self.mask_in = true;
        self
    }
    pub fn with_mask_out(mut self) -> Self {
        self.mask_out = true;
        self
    }
    pub fn set_window_size(&mut self, window_size: [u32; 2]) {
        self.window_size = window_size;
    }
    fn points(&self) -> [(f32, f32); 4] {
        [
            (self.x, self.y),
            (self.x, self.y + self.height),
            (self.x + self.width, self.y + self.height),
            (self.x + self.width, self.y),
        ]
    }
    pub fn vertices(&self) -> [Vertex; 4] {
        let mut vertices = cal_vertices(self.points(), self.window_size);
        for vertex in vertices.iter_mut() {
            vertex.color = [
                self.color[0] as f32 / 255.0,
                self.color[1] as f32 / 255.0,
                self.color[2] as f32 / 255.0,
                self.color[3] as f32 / 255.0,
            ];
            vertex.blend_mode = self.blend_mode as u32;
        }
        vertices
    }
    pub const fn indices(&self) -> [u16; 6] {
        [0, 1, 2, 0, 2, 3]
    }
}

pub struct RawSprite {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub texture_id: u32,
    pub blend_mode: BlendMode,
    pub mask_in: bool,
    pub mask_out: bool,
}

fn cal_vertices<'a>(p: [(f32, f32); 4], size: [u32; 2]) -> [Vertex; 4] {
    let width = size[0];
    let height = size[1];
    let p0 = p[0];
    let p1 = p[1];
    let p2 = p[2];
    let p3 = p[3];
    let w_2 = (width / 2) as f32;
    let h_2 = (height / 2) as f32;
    let center = (w_2, h_2);
    let p0_v = Vertex::new((p0.0 - center.0) / w_2, -(p0.1 - center.1) / h_2, 0.0, 0.0);
    let p1_v = Vertex::new((p1.0 - center.0) / w_2, -(p1.1 - center.1) / h_2, 0.0, 1.0);
    let p2_v = Vertex::new((p2.0 - center.0) / w_2, -(p2.1 - center.1) / h_2, 1.0, 1.0);
    let p3_v = Vertex::new((p3.0 - center.0) / w_2, -(p3.1 - center.1) / h_2, 1.0, 0.0);

    let vertices: [Vertex; 4] = [p0_v, p1_v, p2_v, p3_v];

    vertices
}
