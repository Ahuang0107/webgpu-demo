use glam::Vec2;

#[derive(Copy, Clone, Debug, Default)]
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
    pub const fn new_u32(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        Self {
            min: Vec2::new(x0 as f32, y0 as f32),
            max: Vec2::new(x1 as f32, y1 as f32),
        }
    }

    #[inline]
    pub fn from_center_size(origin: Vec2, size: Vec2) -> Self {
        assert!(size.cmpge(Vec2::ZERO).all(), "Rect size must be positive");
        let half_size = size / 2.0;
        Self {
            min: origin - half_size,
            max: origin + half_size,
        }
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        (point.cmpge(self.min) & point.cmple(self.max)).all()
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.min.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.max.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.max.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.min.y
    }
}
