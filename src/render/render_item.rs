use std::ops::Range;

#[derive(Clone, Debug)]
pub enum RenderItem {
    Sprite {
        range: Range<u32>,
        texture_id: u32,
        sort_key: f32,
    },
    BlendModeSprite {
        range: Range<u32>,
        texture_id: u32,
        sort_key: f32,
    },
    BlurSprite {
        range: Range<u32>,
        texture_id: u32,
        sort_key: f32,
    },
    SpriteMaskStart {
        range: Range<u32>,
        texture_id: u32,
        sort_key: f32,
    },
    SpriteMaskEnd {
        range: Range<u32>,
        texture_id: u32,
        sort_key: f32,
    },
}

impl RenderItem {
    #[inline]
    pub fn texture_id(&self) -> u32 {
        match self {
            RenderItem::Sprite { texture_id, .. } => *texture_id,
            RenderItem::BlendModeSprite { texture_id, .. } => *texture_id,
            RenderItem::BlurSprite { texture_id, .. } => *texture_id,
            RenderItem::SpriteMaskStart { texture_id, .. } => *texture_id,
            RenderItem::SpriteMaskEnd { texture_id, .. } => *texture_id,
        }
    }
    #[inline]
    pub fn range(&self) -> &Range<u32> {
        match self {
            RenderItem::Sprite { range, .. } => range,
            RenderItem::BlendModeSprite { range, .. } => range,
            RenderItem::BlurSprite { range, .. } => range,
            RenderItem::SpriteMaskStart { range, .. } => range,
            RenderItem::SpriteMaskEnd { range, .. } => range,
        }
    }
    #[inline]
    pub fn type_key(&self) -> u8 {
        match self {
            RenderItem::Sprite { .. } => 2,
            RenderItem::BlendModeSprite { .. } => 2,
            RenderItem::BlurSprite { .. } => 2,
            RenderItem::SpriteMaskStart { .. } => 1,
            RenderItem::SpriteMaskEnd { .. } => 3,
        }
    }

    #[inline]
    pub fn sort_key(&self) -> f32 {
        match self {
            RenderItem::Sprite { sort_key, .. } => *sort_key,
            RenderItem::BlendModeSprite { sort_key, .. } => *sort_key,
            RenderItem::BlurSprite { sort_key, .. } => *sort_key,
            RenderItem::SpriteMaskStart { sort_key, .. } => *sort_key,
            RenderItem::SpriteMaskEnd { sort_key, .. } => *sort_key,
        }
    }
}
