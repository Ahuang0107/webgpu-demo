use crate::Rect;

pub static AUDIO_PICKUP: &'static [u8] = include_bytes!("audio/pickup_demo.ogg");
pub static AUDIO_PLACE: &'static [u8] = include_bytes!("audio/place_demo_2.ogg");
pub static AUDIO_BGM: &'static [u8] =
    include_bytes!("audio/bgm/Carousel Dreams - The Soundlings.mp3");
pub static AUDIO_BGM_2: &'static [u8] = include_bytes!("audio/bgm/Unrest - ELPHNT.mp3");
pub static AUDIO_AMBIENT: &'static [u8] = include_bytes!("audio/ambient_sound_demo.ogg");
pub static AUDIO_RECORD_PRESS: &'static [u8] = include_bytes!("audio/record_press.ogg");

pub static UI_CURSOR: (AssetsId, &'static [u8]) = (
    AssetsId::new(b"bbe14c53-ab76-40b4-8383-1c3da2ca0a2b"),
    include_bytes!("ui/ui-cursor.png"),
);
pub static BG_CHECKER: (AssetsId, &'static [u8]) = (
    AssetsId::new(b"903c0509-5e63-49ad-aa14-7c9efb603fbd"),
    include_bytes!("checker.png"),
);
pub static UI_ZOOM_IN: (AssetsId, &'static [u8]) = (
    AssetsId::new(b"36bd36f0-6626-43d2-a53b-9318cd33ea66"),
    include_bytes!("ui/ui-zoom-in.png"),
);
pub static UI_ZOOM_IN_SLICE: [Rect; 6] = [
    Rect::new_u32(0, 0, 35, 35),
    Rect::new_u32(36, 0, 71, 35),
    Rect::new_u32(72, 0, 107, 35),
    Rect::new_u32(108, 0, 143, 35),
    Rect::new_u32(144, 0, 179, 35),
    Rect::new_u32(180, 0, 215, 35),
    // Rect::new_u32(216, 0, 251, 35),
    // Rect::new_u32(252, 0, 287, 35),
];
pub static UI_ZOOM_OUT: (AssetsId, &'static [u8]) = (
    AssetsId::new(b"c618c22b-27b5-4827-ba2a-bc5602ea21bb"),
    include_bytes!("ui/ui-zoom-out.png"),
);
pub static UI_ZOOM_OUT_SLICE: [Rect; 6] = [
    Rect::new_u32(0, 0, 35, 35),
    Rect::new_u32(36, 0, 71, 35),
    Rect::new_u32(72, 0, 107, 35),
    Rect::new_u32(108, 0, 143, 35),
    Rect::new_u32(144, 0, 179, 35),
    Rect::new_u32(180, 0, 215, 35),
    // Rect::new_u32(216, 0, 251, 35),
    // Rect::new_u32(252, 0, 287, 35),
];

pub static SCENE_SIDEBOARD: &'static [u8] = include_bytes!("scenes/SideBoardScene.json");
pub static PACKAGE_SIDEBOARD: &'static [u8] = include_bytes!("package/SideBoardSceneTotal.pkg");

pub static START_NORMAL: (AssetsId, &'static [u8]) = (
    AssetsId::new(b"8688cb57-0b86-48c7-b9ae-d6cfdba6a562"),
    include_bytes!("ui/start-normal.png"),
);
pub static START_HOVER: (AssetsId, &'static [u8]) = (
    AssetsId::new(b"03e0e1a3-d6c3-4a68-82a0-a29edac1e645"),
    include_bytes!("ui/start-hover.png"),
);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AssetsId([u8; 36]);

impl AssetsId {
    pub const INVALID: AssetsId = AssetsId([0; 36]);
    #[inline]
    const fn new(input: &[u8; 36]) -> AssetsId {
        AssetsId(*input)
    }
    pub fn from_u32(num: u32) -> AssetsId {
        let mut result = [0u8; 36];
        let bytes = num.to_be_bytes();
        result[32..].copy_from_slice(&bytes);
        AssetsId(result)
    }
}

impl Default for AssetsId {
    fn default() -> Self {
        AssetsId::INVALID
    }
}

impl std::fmt::Display for AssetsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssetsId({})",
            std::str::from_utf8(&self.0).expect("Invalid UTF-8 UUID")
        )
    }
}
