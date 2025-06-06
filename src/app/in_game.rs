use crate::assets::{AssetsId, BG_CHECKER, PACKAGE_SIDEBOARD, SCENE_SIDEBOARD};
use crate::input::Input;
use crate::utils::collect_sprites;
use crate::{Audio, Camera2D, Color, Render, ScreenRepeat, Sprite, TextureStore};
use glam::Vec2;
use isometric_engine::{MetaModel, Package, Scene, SerdeFrom};
use std::collections::HashMap;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Debug, Default)]
pub struct InGame {
    camera: Camera2D,
    sprites: Vec<Sprite>,
    screen_repeat: ScreenRepeat,
    package: Option<Package>,
    scene: Option<Scene>,
    image_map: HashMap<MetaModel, AssetsId>,
}

impl InGame {
    pub fn new(
        render: &Render,
        texture_store: &mut TextureStore,
        window_size: PhysicalSize<u32>,
    ) -> InGame {
        let mut camera = Camera2D::new(Vec2::new(
            window_size.width as f32,
            window_size.height as f32,
        ));
        camera.set_zoom(2);

        let screen_repeat = ScreenRepeat {
            texture_id: texture_store.load_texture_raw(render, BG_CHECKER),
            offset: Vec2::ZERO,
            scale: 1.0 / camera.get_scale(),
            color: Color::from((107, 13, 56)),
        };

        let scene = Scene::from_bytes(SCENE_SIDEBOARD);
        camera.set_translation(Vec2::new(
            scene.size()[0] as f32 / 2.0,
            scene.size()[1] as f32 / 2.0,
        ));
        let package = Package::unpack_from_bytes(PACKAGE_SIDEBOARD).unwrap();
        let mut image_map: HashMap<MetaModel, AssetsId> =
            HashMap::with_capacity(package.sprite_image_map.len());
        for (key, image) in package.sprite_image_map.iter() {
            image_map.insert(*key, texture_store.load_texture(render, image));
        }

        InGame {
            camera,
            sprites: Vec::new(),
            screen_repeat,
            package: Some(package),
            scene: Some(scene),
            image_map,
        }
    }

    pub fn resize(&mut self, window_size: PhysicalSize<u32>) {
        self.camera.viewport_size = (window_size.width as f32, window_size.height as f32).into();
    }

    pub fn update(&mut self, delta: Duration, input: &Input, audio: &mut Audio) {
        let cursor_world_pos = self
            .camera
            .viewport_to_world(Vec2::new(
                input.cursor_pos().x as f32,
                input.cursor_pos().y as f32,
            ))
            .truncate();

        let camera = &mut self.camera;
        if input.if_keyboard_just_pressed(&KeyCode::KeyZ) {
            camera.zoom_in();
        }
        if input.if_keyboard_just_pressed(&KeyCode::KeyX) {
            camera.zoom_out();
        }
        let camera_move_step = 2.0;
        if input.if_keyboard_pressed(&KeyCode::ArrowLeft) {
            camera.add_translation(Vec2::new(-camera_move_step, 0.0));
        }
        if input.if_keyboard_pressed(&KeyCode::ArrowRight) {
            camera.add_translation(Vec2::new(camera_move_step, 0.0));
        }
        if input.if_keyboard_pressed(&KeyCode::ArrowUp) {
            camera.add_translation(Vec2::new(0.0, camera_move_step));
        }
        if input.if_keyboard_pressed(&KeyCode::ArrowDown) {
            camera.add_translation(Vec2::new(0.0, -camera_move_step));
        }

        camera.update_anima(delta);
        self.screen_repeat.scale = 1.0 / camera.get_scale();

        let Some(scene) = &self.scene else {
            return;
        };
        let scene_center = Vec2::new(scene.size()[0] as f32, scene.size()[1] as f32);
        self.screen_repeat.offset = (camera.get_translation() - scene_center) * 0.4;

        if let (Some(scene), Some(package)) = (&mut self.scene, &mut self.package) {
            if input.if_keyboard_just_pressed(&KeyCode::KeyS) {
                // TODO 这里不能简单的直接序列化，存档需要有一些额外的操作，有些物品是不需要持久化状态的，所以需要保存时将状态重置到初始状态。
                std::fs::write("src/assets/scenes/SideBoardScene.json", scene.to_bytes()).unwrap();
                log::info!("Saved scene");
            }
            if input.if_keyboard_just_pressed(&KeyCode::KeyT) {
                scene
                    .take_out_new_item()
                    .expect("Failed to take out-new-item");
            }

            let click_type = if input.if_mouse_just_pressed(&MouseButton::Left) {
                1
            } else if input.if_mouse_just_pressed(&MouseButton::Right) {
                2
            } else {
                0
            };
            let _sync_result = scene
                .sync(
                    delta.as_micros() as u64,
                    [cursor_world_pos.x as i32, cursor_world_pos.y as i32],
                    click_type,
                    package.items.as_slice(),
                    audio,
                )
                .expect("failed to sync scene");

            self.sprites = collect_sprites(&scene, &self.image_map);
        }
    }

    pub fn render(&self, render: &Render, texture_store: &TextureStore) {
        render.render(
            texture_store,
            &self.camera,
            &self.sprites.iter().collect(),
            Some(&self.screen_repeat),
        );
    }
}
