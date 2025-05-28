mod edit_mode;

use super::assets::*;
use crate::{App, AppConfig, Audio, BlendMode, Camera2D, Color, Fps, Render, Sprite, Transform};
use glam::{Vec2, Vec3};
use isometric_engine::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use wgpu::SurfaceError;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

pub struct AppData {
    config: AppConfig,
    render: Render,
    audio: Audio,
    camera: Camera2D,
    sprites: Vec<Sprite>,
    size: PhysicalSize<u32>,
    if_size_changed: bool,
    fps: Fps,
    /// 因为 web 上运行时，需要玩家点击了窗口后，才能初始化 AudioContext 所以需要检测第一次点击，重新初始化一遍 audio
    #[cfg(target_arch = "wasm32")]
    if_focused: bool,
    ui_cursor: Sprite,
    bg: Sprite,
    package: Package,
    scene: Scene,
    image_map: HashMap<MetaModel, u32>,
    cursor_pos: PhysicalPosition<f64>,
    keyboard_pressed: Vec<KeyCode>,
    mouse_pressed: Vec<MouseButton>,
}

pub enum AppState {
    MainMenu,
}

impl App for AppData {
    async fn new(window: Arc<Window>) -> Self {
        let mut render = Render::new(window.clone())
            .await
            .expect("Failed to create render");
        let mut audio = Audio::default();
        audio.resume_audio_context();
        audio.load_source("pickup", AUDIO_PICKUP.into());
        audio.load_source("place", AUDIO_PLACE.into());
        audio.load_source("bgm", AUDIO_BGM.into());
        audio.load_source("bgm2", AUDIO_BGM_2.into());
        audio.load_source("ambient", AUDIO_AMBIENT.into());
        audio.load_source("record_press", AUDIO_RECORD_PRESS.into());
        audio.play_sound("bgm");
        audio.play_sound("ambient");

        let mut camera = Camera2D::new(Vec2::new(
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        ));
        camera.transform.translation.x = 620.0;
        camera.transform.translation.y = 600.0;
        camera.transform.scale.x = 0.5;
        camera.transform.scale.y = 0.5;
        camera.near = -2000.0;
        let ui_cursor_image_handle = render.load_texture_raw(UI_CURSOR);
        let ui_cursor = Sprite {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0)),
            texture_id: ui_cursor_image_handle,
            anchor: Vec2::new(-0.5, 0.5),
            ..Default::default()
        };
        let bg_image_handle = render.load_texture_raw(BG);
        let bg = Sprite {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            texture_id: bg_image_handle,
            anchor: Vec2::new(-0.5, -0.5),
            ..Default::default()
        };

        let scene = Scene::from_bytes(SCENE_SIDEBOARD);
        let package = Package::unpack_from_bytes(PACKAGE_SIDEBOARD).unwrap();
        let mut image_map: HashMap<MetaModel, u32> =
            HashMap::with_capacity(package.sprite_image_map.len());
        for (key, image) in package.sprite_image_map.iter() {
            image_map.insert(*key, render.load_texture(image));
        }

        let collect_sprites = scene.collect_sprites();
        let collect_sprite_masks = scene.collect_sprite_masks();
        let mut sprites: Vec<Sprite> =
            Vec::with_capacity(collect_sprites.len() + collect_sprite_masks.len());
        for sprite_pin in collect_sprites {
            if let Some(image_handle) = image_map.get(&sprite_pin.meta) {
                let [x, y, z] = sprite_pin.get_xyz();
                let blend_mode = sprite_pin.blend_mode();
                sprites.push(Sprite {
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                    texture_id: *image_handle,
                    anchor: Vec2::new(-0.5, -0.5),
                    color: Color::new([255, 255, 255, sprite_pin.get_opacity()]),
                    blend_mode: match blend_mode {
                        isometric_engine::BlendMode::Normal => BlendMode::Normal,
                        isometric_engine::BlendMode::Multiply => BlendMode::Multiply,
                        isometric_engine::BlendMode::Overlay => BlendMode::Overlay,
                        isometric_engine::BlendMode::SoftLight => BlendMode::SoftLight,
                        isometric_engine::BlendMode::HardLight => BlendMode::HardLight,
                    },
                    ..Default::default()
                });
            }
        }

        for sprite_mask in collect_sprite_masks {
            if let Some(image_handle) = image_map.get(&sprite_mask.meta) {
                let [x, y] = sprite_mask.get_acl_offset();
                let [mask_start, mask_end] = sprite_mask.get_range();
                sprites.push(Sprite {
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0)),
                    texture_id: *image_handle,
                    anchor: Vec2::new(-0.5, -0.5),
                    mask: Some([mask_start as f32, mask_end as f32]),
                    ..Default::default()
                });
            }
        }

        Self {
            config: AppConfig::default(),
            render,
            audio,
            camera,
            sprites,
            size: window.inner_size(),
            // 默认为 true 确保渲染第一帧前会调整 surface 大小
            if_size_changed: true,
            fps: Fps::new(),
            #[cfg(target_arch = "wasm32")]
            if_focused: false,
            ui_cursor,
            bg,
            package,
            scene,
            image_map,
            cursor_pos: PhysicalPosition::default(),
            keyboard_pressed: Vec::new(),
            mouse_pressed: Vec::new(),
        }
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.if_size_changed = true;
    }

    fn get_config(&self) -> &AppConfig {
        &self.config
    }

    fn keyboard_input(&mut self, event: &KeyEvent) -> bool {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    if !event.repeat {
                        self.keyboard_pressed.push(key_code);
                    }
                }
                ElementState::Released => {}
            }
            return true;
        }
        false
    }

    fn mouse_click(&mut self, state: ElementState, button: MouseButton) -> bool {
        match state {
            ElementState::Pressed => {
                self.mouse_pressed.push(button);
            }
            ElementState::Released => {}
        }
        false
    }

    fn cursor_move(&mut self, position: PhysicalPosition<f64>) -> bool {
        self.cursor_pos = position;
        false
    }

    fn update(&mut self, delta: Duration) {
        #[cfg(target_arch = "wasm32")]
        if self.mouse_pressed.contains(&MouseButton::Left) {
            if !self.if_focused {
                self.if_focused = true;
                self.audio.resume_audio_context();
            }
        }

        self.audio.clean_finished_sink();

        let camera = &mut self.camera;
        if self.keyboard_pressed.contains(&KeyCode::KeyF) {
            self.config.fullscreen = true;
        }
        if self.keyboard_pressed.contains(&KeyCode::KeyB) {
            self.config.decorations = false;
        }
        #[cfg(feature = "windows_wallpaper")]
        if self.keyboard_pressed.contains(&KeyCode::KeyW) {
            self.config.set_as_wallpaper = true;
        }
        if self.keyboard_pressed.contains(&KeyCode::KeyL) {
            if let Some(sink) = self.audio.get_sink(0) {
                sink.set_volume(0.1);
            }
            if let Some(sink) = self.audio.get_sink(1) {
                sink.set_volume(0.1);
            }
        }
        if self.keyboard_pressed.contains(&KeyCode::KeyU) {
            if let Some(sink) = self.audio.get_sink(0) {
                sink.set_volume(1.0);
            }
            if let Some(sink) = self.audio.get_sink(1) {
                sink.set_volume(1.0);
            }
        }
        for key_code in self.keyboard_pressed.drain(..) {
            match key_code {
                KeyCode::KeyZ => {
                    // NOTE 这里不能修改 z 的 scale 因为这会影响到 near 和 far
                    //  这在 3D 游戏中是需要逻辑但是 2D 不需要
                    camera.transform.scale.x -= 0.1;
                    camera.transform.scale.y -= 0.1;
                }
                KeyCode::KeyX => {
                    camera.transform.scale.x += 0.1;
                    camera.transform.scale.y += 0.1;
                }
                KeyCode::ArrowLeft => {
                    camera.transform.translation.x -= 1.0;
                }
                KeyCode::ArrowRight => {
                    camera.transform.translation.x += 1.0;
                }
                KeyCode::ArrowUp => {
                    camera.transform.translation.y += 1.0;
                }
                KeyCode::ArrowDown => {
                    camera.transform.translation.y -= 1.0;
                }
                KeyCode::KeyT => {
                    self.scene
                        .take_out_new_item()
                        .expect("Failed to take out-new-item");
                }
                KeyCode::KeyP => {
                    self.audio.play_sound_with_volume("bgm", 0.4);
                }
                _ => {}
            }
        }
        let world_position = self
            .camera
            .viewport_to_world(Vec2::new(
                self.cursor_pos.x as f32,
                self.cursor_pos.y as f32,
            ))
            .truncate();
        self.ui_cursor.transform.translation.x = world_position.x;
        self.ui_cursor.transform.translation.y = world_position.y;
        let click_type = if self.mouse_pressed.contains(&MouseButton::Left) {
            1
        } else if self.mouse_pressed.contains(&MouseButton::Right) {
            2
        } else {
            0
        };
        let _sync_result = self
            .scene
            .sync(
                delta.as_micros() as u64,
                [world_position.x as i32, world_position.y as i32],
                click_type,
                self.package.items.as_slice(),
                &mut self.audio,
            )
            .expect("failed to sync scene");
        self.mouse_pressed.clear();

        let collect_sprites = self.scene.collect_sprites();
        let collect_sprite_masks = self.scene.collect_sprite_masks();
        self.sprites = Vec::with_capacity(collect_sprites.len() + collect_sprite_masks.len());
        for sprite_pin in collect_sprites {
            if let Some(image_handle) = self.image_map.get(&sprite_pin.meta) {
                let [x, y, z] = sprite_pin.get_xyz();
                let blend_mode = sprite_pin.blend_mode();
                self.sprites.push(Sprite {
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                    texture_id: *image_handle,
                    anchor: Vec2::new(-0.5, -0.5),
                    color: Color::new([255, 255, 255, sprite_pin.get_opacity()]),
                    blend_mode: match blend_mode {
                        isometric_engine::BlendMode::Normal => BlendMode::Normal,
                        isometric_engine::BlendMode::Multiply => BlendMode::Multiply,
                        isometric_engine::BlendMode::Overlay => BlendMode::Overlay,
                        isometric_engine::BlendMode::SoftLight => BlendMode::SoftLight,
                        isometric_engine::BlendMode::HardLight => BlendMode::HardLight,
                    },
                    ..Default::default()
                });
            }
        }

        for sprite_mask in collect_sprite_masks {
            if let Some(image_handle) = self.image_map.get(&sprite_mask.meta) {
                let [x, y] = sprite_mask.get_acl_offset();
                let [mask_start, mask_end] = sprite_mask.get_range();
                self.sprites.push(Sprite {
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0)),
                    texture_id: *image_handle,
                    anchor: Vec2::new(-0.5, -0.5),
                    mask: Some([mask_start as f32, mask_end as f32]),
                    ..Default::default()
                });
            }
        }
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        #[cfg(feature = "profiling")]
        profiling::scope!("Render Frame");
        if self.if_size_changed {
            self.camera.viewport_size = (self.size.width as f32, self.size.height as f32).into();
            // NOTE 之前把 surface_configure 放在这里，发现缩放窗口时会卡顿，于是就移到了外面，每帧都重新 surface_configure
            //  但是后来发现有性能问题，帧率一直很低，只有 200-300 FPS，远低于 bevy 的性能
            //  于是用 Tracy Profiler 测试了一下，发现每帧大部分时间都花在了 surface_configure 上（3ms左右）
            //  现在将 surface_configure 移回这里，性能大大增高，能到 4000 FPS，同时，也没有发现缩放窗口卡顿的问题
            //  虽然不知道之前缩放窗口卡顿的问题是为什么，但先这样吧
            self.render.resize(self.size.width, self.size.height);
            self.if_size_changed = false;
        }

        // 窗口最小化时只更新数据不渲染画面
        if self.size.width > 0 && self.size.height > 0 {
            let mut sprites: Vec<&Sprite> = self.sprites.iter().collect();
            sprites.push(&self.ui_cursor);
            sprites.push(&self.bg);
            self.render.render(&self.camera, sprites.as_slice());
        }
        self.fps.update();
        #[cfg(feature = "profiling")]
        profiling::finish_frame!();

        Ok(())
    }
}
