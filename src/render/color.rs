use glam::{U8Vec4, Vec4};

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Rgba(RgbaColor),
    Hsb(HsbColor),
}

impl Color {
    #[inline]
    pub fn new(rgba: [u8; 4]) -> Self {
        Self::Rgba(rgba.into())
    }

    #[inline]
    pub fn as_vec4(&self) -> Vec4 {
        self.as_rgba().as_vec4() / 255.0
    }

    #[inline]
    pub fn as_rgba(&self) -> RgbaColor {
        match self {
            Self::Rgba(rgba) => *rgba,
            Self::Hsb(hsb) => hsb.as_rgba(),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Rgba(RgbaColor::from([255; 4]))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RgbaColor(U8Vec4);

impl RgbaColor {
    #[inline]
    pub fn as_vec4(&self) -> Vec4 {
        self.0.as_vec4()
    }
}

impl From<[u8; 4]> for RgbaColor {
    fn from(value: [u8; 4]) -> Self {
        Self(U8Vec4::from(value))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct HsbColor {
    /// 色相
    /// 0 - 359
    pub hue: u16,
    /// 饱和度
    /// 0 - 100
    pub saturation: u8,
    /// 亮度
    /// 0 - 100
    ///
    /// = max(R,G,B)/255
    pub brightness: u8,
}

impl HsbColor {
    #[inline]
    pub fn new(mut hue: u16, mut saturation: u8, mut brightness: u8) -> Self {
        if hue >= 360 {
            hue = 0;
        }
        if saturation > 100 {
            saturation = 100;
        }
        if brightness > 100 {
            brightness = 100;
        }
        Self {
            hue,
            saturation,
            brightness,
        }
    }

    /// 注意 Photoshop 的 hsb 和 rgb 的转换中，比如 rgb=84,255,0 和 rgb=86,255,0 都对应 hsb=100,100,100
    /// 写 UT 的时候需要注意
    #[inline]
    pub fn as_rgba(&self) -> RgbaColor {
        let hub = self.hue as f32;
        let saturation = self.saturation as f32 / 100.0;
        let brightness = self.brightness as f32 / 100.0;

        let c = brightness * saturation;
        let x = c * (1.0 - ((hub / 60.0) % 2.0 - 1.0).abs());
        let m = brightness - c;

        let interval = (self.hue as f32 / 60.0).floor();

        let (r, g, b) = match interval as u8 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => {
                panic!()
            }
        };

        RgbaColor::from([
            ((r + m) * 255.0).ceil() as u8,
            ((g + m) * 255.0).ceil() as u8,
            ((b + m) * 255.0).ceil() as u8,
            255,
        ])
    }
}

#[cfg(test)]
pub fn test() {
    assert_eq!(
        HsbColor::new(0, 50, 60).as_rgba(),
        RgbaColor::from([153, 77, 77, 255])
    );
    assert_eq!(
        HsbColor::new(65, 50, 60).as_rgba(),
        RgbaColor::from([147, 153, 77, 255])
    );
    assert_eq!(
        HsbColor::new(148, 50, 60).as_rgba(),
        RgbaColor::from([77, 153, 113, 255])
    );
    assert_eq!(
        HsbColor::new(0, 100, 0).as_rgba(),
        RgbaColor::from([0, 0, 0, 255])
    );
    assert_eq!(
        HsbColor::new(0, 0, 0).as_rgba(),
        RgbaColor::from([0, 0, 0, 255])
    );
    assert_eq!(
        HsbColor::new(0, 0, 100).as_rgba(),
        RgbaColor::from([255, 255, 255, 255])
    );
    assert_eq!(
        HsbColor::new(0, 100, 100).as_rgba(),
        RgbaColor::from([255, 0, 0, 255])
    );
    assert_eq!(
        HsbColor::new(100, 100, 100).as_rgba(),
        RgbaColor::from([86, 255, 0, 255])
    );
}
