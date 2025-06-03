mod edit_mode;

use super::assets::*;
use crate::input::Input;
use crate::utils::collect_sprites;
use crate::{App, AppConfig, Audio, Camera2D, Color, Fps, Render, ScreenRepeat, Sprite, Transform};
use glam::{Vec2, Vec3};
use isometric_engine::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::event::{MouseButton, WindowEvent};
use winit::keyboard::KeyCode;
use winit::window::Window;

pub struct AppData {
    config: AppConfig,
    input: Input,
    render: Render,
    audio: Audio,
    camera: Camera2D,
    sprites: Vec<Sprite>,
    screen_repeat: ScreenRepeat,
    size: PhysicalSize<u32>,
    if_size_changed: bool,
    fps: Fps,
    /// 因为 web 上运行时，需要玩家点击了窗口后，才能初始化 AudioContext 所以需要检测第一次点击，重新初始化一遍 audio
    #[cfg(target_arch = "wasm32")]
    if_focused: bool,
    ui_cursor: Sprite,
    package: Package,
    scene: Scene,
    image_map: HashMap<MetaModel, u32>,
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
        // audio.play_sound("bgm");
        // audio.play_sound("ambient");

        let mut camera = Camera2D::new(Vec2::new(
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        ));
        camera.transform.translation.x = 620.0;
        camera.transform.translation.y = 600.0;
        camera.zoom_in();
        camera.near = -2000.0;
        let ui_cursor_image_handle = render.load_texture_raw(UI_CURSOR);
        let ui_cursor = Sprite {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0)),
            texture_id: ui_cursor_image_handle,
            anchor: Vec2::new(-0.5, 0.5),
            ..Default::default()
        };

        let scene = Scene::from_bytes(SCENE_SIDEBOARD);
        let package = Package::unpack_from_bytes(PACKAGE_SIDEBOARD).unwrap();
        let mut image_map: HashMap<MetaModel, u32> =
            HashMap::with_capacity(package.sprite_image_map.len());
        for (key, image) in package.sprite_image_map.iter() {
            image_map.insert(*key, render.load_texture(image));
        }

        let screen_repeat = ScreenRepeat {
            texture_id: render.load_texture_raw(BG_CHECKER),
            offset: Vec2::ZERO,
            scale: 1.0 / camera.transform.scale.truncate(),
            color: Color::from((107, 13, 56)),
        };

        Self {
            config: AppConfig::default(),
            input: Input::default(),
            render,
            audio,
            camera,
            sprites: Vec::new(),
            screen_repeat,
            size: window.inner_size(),
            // 默认为 true 确保渲染第一帧前会调整 surface 大小
            if_size_changed: true,
            fps: Fps::new(),
            #[cfg(target_arch = "wasm32")]
            if_focused: false,
            ui_cursor,
            package,
            scene,
            image_map,
        }
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.if_size_changed = true;
    }

    fn get_config(&self) -> &AppConfig {
        &self.config
    }

    fn on_window_input(&mut self, event: &WindowEvent) {
        self.input.handle_window_event(event);
    }

    fn update(&mut self, delta: Duration) {
        #[cfg(target_arch = "wasm32")]
        if self.input.if_mouse_just_pressed(&MouseButton::Left) {
            if !self.if_focused {
                self.if_focused = true;
                self.audio.resume_audio_context();
            }
        }

        self.audio.clean_finished_sink();

        let camera = &mut self.camera;
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyF) {
            self.config.fullscreen = true;
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyB) {
            self.config.decorations = false;
        }
        #[cfg(feature = "windows_wallpaper")]
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyW) {
            self.config.set_as_wallpaper = true;
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyL) {
            if let Some(sink) = self.audio.get_sink(0) {
                sink.set_volume(0.1);
            }
            if let Some(sink) = self.audio.get_sink(1) {
                sink.set_volume(0.1);
            }
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyU) {
            if let Some(sink) = self.audio.get_sink(0) {
                sink.set_volume(1.0);
            }
            if let Some(sink) = self.audio.get_sink(1) {
                sink.set_volume(1.0);
            }
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyS) {
            // TODO 这里不能简单的直接序列化，存档需要有一些额外的操作，有些物品是不需要持久化状态的，所以需要保存时将状态重置到初始状态。
            std::fs::write(
                "src/assets/scenes/SideBoardScene.json",
                self.scene.to_bytes(),
            )
            .unwrap();
            log::info!("Saved scene");
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyZ) {
            camera.zoom_in();
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyX) {
            camera.zoom_out();
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::ArrowLeft) {
            camera.transform.translation.x -= 1.0;
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::ArrowRight) {
            camera.transform.translation.x += 1.0;
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::ArrowUp) {
            camera.transform.translation.y += 1.0;
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::ArrowDown) {
            camera.transform.translation.y -= 1.0;
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyT) {
            self.scene
                .take_out_new_item()
                .expect("Failed to take out-new-item");
        }
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyP) {
            self.audio.play_sound_with_volume("bgm", 0.4);
        }

        camera.update_anima(delta.as_millis() as u64);
        self.screen_repeat.scale = 1.0 / self.camera.transform.scale.truncate();

        let world_position = self
            .camera
            .viewport_to_world(Vec2::new(
                self.input.cursor_pos().x as f32,
                self.input.cursor_pos().y as f32,
            ))
            .truncate();
        self.ui_cursor.transform.translation.x = world_position.x;
        self.ui_cursor.transform.translation.y = world_position.y;
        let click_type = if self.input.if_mouse_just_pressed(&MouseButton::Left) {
            1
        } else if self.input.if_mouse_just_pressed(&MouseButton::Right) {
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

        self.sprites = collect_sprites(&self.scene, &self.image_map);

        self.input.fresh();
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
            self.render
                .render(&self.camera, sprites.as_slice(), Some(&self.screen_repeat));
        }
        self.fps.update();
        #[cfg(feature = "profiling")]
        profiling::finish_frame!();

        Ok(())
    }
}
