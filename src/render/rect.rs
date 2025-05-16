use glam::Vec2;

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    #[inline]
    pub fn from_corners(p0: Vec2, p1: Vec2) -> Self {
        Self {
            min: p0.min(p1),
            max: p0.max(p1),
        }
    }
    #[inline]
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::from_corners(Vec2::new(x0, y0), Vec2::new(x1, y1))
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
}
