use crate::assets::{
    AssetsId, BG_CHECKER, PACKAGE_SIDEBOARD, SCENE_SIDEBOARD, UI_ZOOM_IN, UI_ZOOM_IN_SLICE,
    UI_ZOOM_OUT, UI_ZOOM_OUT_SLICE,
};
use crate::input::Input;
use crate::utils::collect_sprites;
use crate::{
    Audio, Camera2D, Color, Rect, Render, ScreenRepeat, Sprite, TextureStore, Transform, UiSprite,
};
use glam::{Vec2, Vec3};
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
    ui_zoom_in_sprite: UiSprite,
    ui_zoom_out_sprite: UiSprite,
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
            texture_id: BG_CHECKER.0,
            offset: Vec2::ZERO,
            scale: 1.0 / camera.get_scale(),
            color: Color::from((107, 13, 56)),
        };
        let ui_zoom_in_sprite = UiSprite {
            sprite: Sprite {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0)),
                texture_id: UI_ZOOM_IN.0,
                rect: Some(UI_ZOOM_IN_SLICE[0]),
                ..Default::default()
            },
            size: Vec2::new(36.0, 36.0).into(),
            left: Some(60.0.into()),
            top: Some(16.0.into()),
            ..Default::default()
        };
        let ui_zoom_out_sprite = UiSprite {
            sprite: Sprite {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0)),
                texture_id: UI_ZOOM_OUT.0,
                rect: Some(UI_ZOOM_OUT_SLICE[0]),
                ..Default::default()
            },
            size: Vec2::new(36.0, 36.0).into(),
            left: Some(16.0.into()),
            top: Some(16.0.into()),
            ..Default::default()
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
            ui_zoom_in_sprite,
            ui_zoom_out_sprite,
            package: Some(package),
            scene: Some(scene),
            image_map,
        }
    }

    pub fn resize(&mut self, window_size: PhysicalSize<u32>) {
        self.camera.viewport_size = (window_size.width as f32, window_size.height as f32).into();
    }

    pub fn update(&mut self, delta: Duration, input: &Input, audio: &mut Audio) {
        let cursor_world_pos = self.camera.viewport_to_world(input.cursor_pos()).truncate();

        let camera = &mut self.camera;
        if !camera.if_can_zoom_in() {
            self.ui_zoom_in_sprite.sprite.color.set_opacity(0.5);
            self.ui_zoom_in_sprite.stop_anima();
        } else {
            if self.ui_zoom_in_sprite.contains(cursor_world_pos) {
                self.ui_zoom_in_sprite.sprite.color.set_opacity(1.0);
                self.ui_zoom_in_sprite.start_anima(&UI_ZOOM_IN_SLICE);
                if input.if_mouse_just_pressed(&MouseButton::Left) {
                    // TODO 这里与 ui 交互了，就不能与场景交互了
                    camera.zoom_in();
                }
            } else {
                self.ui_zoom_in_sprite.sprite.color.set_opacity(0.7);
                self.ui_zoom_in_sprite.stop_anima();
            }
        }

        if !camera.if_can_zoom_out() {
            self.ui_zoom_out_sprite.sprite.color.set_opacity(0.5);
            self.ui_zoom_out_sprite.stop_anima();
        } else {
            if self.ui_zoom_out_sprite.contains(cursor_world_pos) {
                self.ui_zoom_out_sprite.sprite.color.set_opacity(1.0);
                self.ui_zoom_out_sprite.start_anima(&UI_ZOOM_OUT_SLICE);
                if input.if_mouse_just_pressed(&MouseButton::Left) {
                    camera.zoom_out();
                }
            } else {
                self.ui_zoom_out_sprite.sprite.color.set_opacity(0.7);
                self.ui_zoom_out_sprite.stop_anima();
            }
        }

        let camera_move_step = 4.0;
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
        if input.cursor_pos().x < 48.0 {
            if !input.if_keyboard_pressed(&KeyCode::ArrowLeft)
                && !input.if_keyboard_pressed(&KeyCode::ArrowRight)
            {
                camera.add_translation(Vec2::new(-camera_move_step, 0.0));
            }
            // let boundary_deep = 96.0 - input.cursor_pos().x;
        }
        if input.cursor_pos().x > (camera.viewport_size.x - 48.0) {
            if !input.if_keyboard_pressed(&KeyCode::ArrowLeft)
                && !input.if_keyboard_pressed(&KeyCode::ArrowRight)
            {
                camera.add_translation(Vec2::new(camera_move_step, 0.0));
            }
            // let boundary_deep = 96.0 - input.cursor_pos().x;
        }
        if input.cursor_pos().y < 48.0 {
            if !input.if_keyboard_pressed(&KeyCode::ArrowUp)
                && !input.if_keyboard_pressed(&KeyCode::ArrowDown)
            {
                camera.add_translation(Vec2::new(0.0, camera_move_step));
            }
        }
        if input.cursor_pos().y > (camera.viewport_size.y - 48.0) {
            if !input.if_keyboard_pressed(&KeyCode::ArrowUp)
                && !input.if_keyboard_pressed(&KeyCode::ArrowDown)
            {
                camera.add_translation(Vec2::new(0.0, -camera_move_step));
            }
        }

        let Some(scene) = &self.scene else {
            return;
        };
        let scene_world_boundary = Rect::from_corners(
            Vec2::new(0.0, 48.0),
            Vec2::new(scene.size()[0] as f32, scene.size()[1] as f32 - 48.0),
        );

        camera.update_anima(delta);
        camera.update_word_boundary(Some(scene_world_boundary));
        self.screen_repeat.scale = 1.0 / camera.get_scale();

        let scene_center = Vec2::new(scene.size()[0] as f32, scene.size()[1] as f32);
        self.screen_repeat.offset = (camera.get_translation() - scene_center) * 0.4;
        self.ui_zoom_in_sprite.update(camera, delta);
        self.ui_zoom_out_sprite.update(camera, delta);

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

    pub fn render(
        &self,
        render: &Render,
        texture_store: &TextureStore,
        #[cfg(feature = "editor_mode")] egui_render: &mut crate::egui_render::EguiRender,
    ) {
        let mut sprites: Vec<&Sprite> = self.sprites.iter().collect();
        sprites.push(&self.ui_zoom_in_sprite.sprite);
        sprites.push(&self.ui_zoom_out_sprite.sprite);
        render.render(
            texture_store,
            &self.camera,
            &sprites,
            Some(&self.screen_repeat),
            #[cfg(feature = "editor_mode")]
            egui_render,
        );
    }
}
