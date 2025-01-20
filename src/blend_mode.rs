#[allow(dead_code)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(C)]
pub enum BlendMode {
    #[default]
    Normal = 0,
    // Darken = 10,
    Multiply = 11,
    // ColorBurn = 12,
    // Lighten = 20,
    // Screen = 21,
    // ColorDodge = 22,
    // Addition = 23,
    Overlay = 30,
    SoftLight = 31,
    HardLight = 32,
    // Difference = 40,
    // Exclusion = 41,
    // Subtract = 42,
    // Divide = 43,
    // Hue = 50,
    // Saturation = 51,
    // Color = 52,
    // Luminosity = 53,
    Blur = 60,
}
