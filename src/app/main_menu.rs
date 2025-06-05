use crate::assets::{START_HOVER, START_NORMAL};
use crate::input::Input;
use crate::{AppState, Audio, Camera2D, Rect, Render, Sprite, TextureStore, Transform};
use glam::{Vec2, Vec3, Vec3Swizzles};
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::MouseButton;

#[derive(Debug, Default)]
pub struct MainMenu {
    camera: Camera2D,
    start: Sprite,
}

impl MainMenu {
    pub fn new(
        render: &Render,
        texture_store: &mut TextureStore,
        window_size: PhysicalSize<u32>,
    ) -> Self {
        texture_store.load_texture_raw(render, START_NORMAL);
        texture_store.load_texture_raw(render, START_HOVER);
        let start = Sprite {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 400.0)),
            texture_id: START_NORMAL.0,
            ..Default::default()
        };

        let mut camera = Camera2D::new(Vec2::new(
            window_size.width as f32,
            window_size.height as f32,
        ));
        camera.transform.translation.x = 0.0;
        camera.transform.translation.y = 0.0;
        camera.transform.scale.x = 1.0;
        camera.transform.scale.y = 1.0;

        MainMenu { camera, start }
    }

    pub fn resize(&mut self, window_size: PhysicalSize<u32>) {
        self.camera.viewport_size = (window_size.width as f32, window_size.height as f32).into();
    }

    pub fn update(
        &mut self,
        _delta: Duration,
        input: &Input,
        _audio: &mut Audio,
        texture_store: &TextureStore,
        next_app_state: &mut Option<AppState>,
    ) {
        let cursor_world_pos = self
            .camera
            .viewport_to_world(Vec2::new(
                input.cursor_pos().x as f32,
                input.cursor_pos().y as f32,
            ))
            .truncate();
        if let Some((size, _)) = texture_store.get(&START_NORMAL.0) {
            let rect = Rect::from_center_size(self.start.transform.translation.xy(), *size);
            if rect.contains(cursor_world_pos) {
                self.start.texture_id = START_HOVER.0;
                if input.if_mouse_just_pressed(&MouseButton::Left) {
                    *next_app_state = Some(AppState::InGame);
                }
            } else {
                self.start.texture_id = START_NORMAL.0;
            }
        }
    }

    pub fn render(&self, render: &Render, texture_store: &TextureStore) {
        render.render(texture_store, &self.camera, &vec![&self.start], None);
    }
}
