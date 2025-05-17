#[derive(Copy, Clone, Debug)]
pub struct Color([u8; 4]);

impl Color {
    #[inline]
    pub fn new(rgba: [u8; 4]) -> Self {
        Self(rgba)
    }

    #[inline]
    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.0[0] as f32 / 255.0,
            self.0[1] as f32 / 255.0,
            self.0[2] as f32 / 255.0,
            self.0[3] as f32 / 255.0,
        ]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self([255; 4])
    }
}
