use crate::assets::{AssetsId, START_HOVER, START_NORMAL};
use crate::input::Input;
use crate::{Audio, Camera2D, Render, Sprite, TextureStore, Transform};
use glam::Vec3;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct MainMenu {
    start: Sprite,
}

impl MainMenu {
    pub fn new(render: &Render, texture_store: &mut TextureStore, camera: &mut Camera2D) -> Self {
        texture_store.load_texture_raw(render, START_NORMAL);
        texture_store.load_texture_raw(render, START_HOVER);
        let start = Sprite {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 400.0)),
            texture_id: START_NORMAL.0,
            ..Default::default()
        };

        camera.transform.translation.x = 0.0;
        camera.transform.translation.y = 0.0;
        camera.transform.scale.x = 1.0;
        camera.transform.scale.y = 1.0;

        MainMenu { start }
    }

    pub fn update(
        &mut self,
        delta: Duration,
        input: &Input,
        audio: &mut Audio,
        camera: &mut Camera2D,
    ) {
    }

    pub fn render(&self, render: &Render, texture_store: &TextureStore, camera: &Camera2D) {
        render.render(texture_store, camera, &vec![], None);
    }
}
