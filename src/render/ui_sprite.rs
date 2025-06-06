use crate::{Camera2D, Rect, Sprite};
use glam::Vec2;
use std::time::Duration;

/// 直接在 Sprite 的基础上添加 UI Sprite 的支持
#[derive(Copy, Clone, Debug, Default)]
pub struct UiSprite {
    pub sprite: Sprite,
    pub size: Val<Vec2>,
    pub left: Option<Val<f32>>,
    pub top: Option<Val<f32>>,
    pub loop_anima: Option<(&'static [Rect], usize, Duration)>,
}

impl UiSprite {
    pub fn update(&mut self, camera: &Camera2D, delta: Duration) {
        let size = self.size.cal(camera);

        self.sprite.custom_size = Some(size * camera.get_scale());

        if let Some(left) = self.left {
            let left = left.cal(camera) + size.x / 2.0;
            self.sprite.transform.translation.x = camera
                .viewport_to_world(Vec2::new(camera.viewport_size.x - left, 0.0))
                .x;
        }
        if let Some(top) = self.top {
            let top = top.cal(camera) + size.y / 2.0;
            self.sprite.transform.translation.y = camera.viewport_to_world(Vec2::new(0.0, top)).y;
        }
        if let Some((frames, index, elapsed)) = &mut self.loop_anima {
            *elapsed += delta;
            if *elapsed >= Duration::from_millis(100) {
                *elapsed = elapsed.saturating_sub(Duration::from_millis(100));
                *index = (*index + 1) % frames.len();
            }
            self.sprite.rect = Some(frames[*index]);
        }
    }
    pub fn contains(&self, cursor: Vec2) -> bool {
        if let Some(custom_size) = self.sprite.custom_size {
            let rect =
                Rect::from_center_size(self.sprite.transform.translation.truncate(), custom_size);
            if rect.contains(cursor) {
                return true;
            }
        }
        false
    }
    pub fn start_anima(&mut self, frames: &'static [Rect]) {
        if self.loop_anima.is_none() {
            self.loop_anima = Some((frames, 0, Duration::ZERO));
        }
    }

    pub fn stop_anima(&mut self) {
        if let Some((frames, _, _)) = self.loop_anima {
            self.sprite.rect = Some(frames[0]);
        }
        self.loop_anima = None;
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Val<T> {
    base: T,
}

impl From<f32> for Val<f32> {
    fn from(value: f32) -> Self {
        Self { base: value }
    }
}

impl Val<f32> {
    pub fn cal(&self, camera: &Camera2D) -> f32 {
        if camera.viewport_size.x >= 3840.0 && camera.viewport_size.y >= 2160.0 {
            self.base * 4.0
        } else if camera.viewport_size.x >= 2880.0 && camera.viewport_size.y >= 1620.0 {
            self.base * 2.0
        } else if camera.viewport_size.x >= 1920.0 && camera.viewport_size.y >= 1080.0 {
            self.base * 2.0
        } else {
            self.base
        }
    }
}

impl From<Vec2> for Val<Vec2> {
    fn from(value: Vec2) -> Self {
        Self { base: value }
    }
}

impl Val<Vec2> {
    pub fn cal(&self, camera: &Camera2D) -> Vec2 {
        if camera.viewport_size.x >= 3840.0 && camera.viewport_size.y >= 2160.0 {
            self.base * 4.0
        } else if camera.viewport_size.x >= 1920.0 && camera.viewport_size.y >= 1080.0 {
            self.base * 2.0
        } else {
            self.base
        }
    }
}
