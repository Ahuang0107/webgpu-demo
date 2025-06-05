use crate::assets::{AssetsId, BG_CHECKER, PACKAGE_SIDEBOARD, SCENE_SIDEBOARD};
use crate::input::Input;
use crate::utils::collect_sprites;
use crate::{Audio, Camera2D, Color, Render, ScreenRepeat, Sprite, TextureStore};
use glam::Vec2;
use isometric_engine::{MetaModel, Package, Scene, SerdeFrom};
use std::collections::HashMap;
use std::time::Duration;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Debug, Default)]
pub struct InGame {
    sprites: Vec<Sprite>,
    screen_repeat: ScreenRepeat,
    package: Option<Package>,
    scene: Option<Scene>,
    image_map: HashMap<MetaModel, AssetsId>,
}

impl InGame {
    pub fn new(render: &Render, texture_store: &mut TextureStore, camera: &mut Camera2D) -> InGame {
        camera.set_zoom(2);

        let screen_repeat = ScreenRepeat {
            texture_id: texture_store.load_texture_raw(render, BG_CHECKER),
            offset: Vec2::ZERO,
            scale: 1.0 / camera.transform.scale.truncate(),
            color: Color::from((107, 13, 56)),
        };

        InGame {
            sprites: Vec::new(),
            screen_repeat,
            package: None,
            scene: None,
            image_map: HashMap::default(),
        }
    }

    pub fn load_scene(&mut self, render: &Render, texture_store: &mut TextureStore) {
        let scene = Scene::from_bytes(SCENE_SIDEBOARD);
        let package = Package::unpack_from_bytes(PACKAGE_SIDEBOARD).unwrap();
        let mut image_map: HashMap<MetaModel, AssetsId> =
            HashMap::with_capacity(package.sprite_image_map.len());
        for (key, image) in package.sprite_image_map.iter() {
            image_map.insert(*key, texture_store.load_texture(render, image));
        }

        unimplemented!()
    }

    pub fn update(
        &mut self,
        delta: Duration,
        input: &Input,
        audio: &mut Audio,
        camera: &mut Camera2D,
    ) {
        if input.if_keyboard_just_pressed(&KeyCode::KeyZ) {
            camera.zoom_in();
        }
        if input.if_keyboard_just_pressed(&KeyCode::KeyX) {
            camera.zoom_out();
        }
        if input.if_keyboard_just_pressed(&KeyCode::ArrowLeft) {
            camera.transform.translation.x -= 1.0;
        }
        if input.if_keyboard_just_pressed(&KeyCode::ArrowRight) {
            camera.transform.translation.x += 1.0;
        }
        if input.if_keyboard_just_pressed(&KeyCode::ArrowUp) {
            camera.transform.translation.y += 1.0;
        }
        if input.if_keyboard_just_pressed(&KeyCode::ArrowDown) {
            camera.transform.translation.y -= 1.0;
        }

        camera.update_anima(delta);
        self.screen_repeat.scale = 1.0 / camera.transform.scale.truncate();

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
                    [
                        input.cursor_world_pos().x as i32,
                        input.cursor_world_pos().y as i32,
                    ],
                    click_type,
                    package.items.as_slice(),
                    audio,
                )
                .expect("failed to sync scene");

            self.sprites = collect_sprites(&scene, &self.image_map);
        }
    }

    pub fn render(&self, render: &Render, texture_store: &TextureStore, camera: &Camera2D) {
        render.render(
            texture_store,
            camera,
            &self.sprites.iter().collect(),
            Some(&self.screen_repeat),
        );
    }
}
