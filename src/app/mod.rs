mod edit_mode;
mod in_game;
mod main_menu;

use super::assets::*;
use crate::app::in_game::InGame;
use crate::app::main_menu::MainMenu;
use crate::input::Input;
use crate::{App, AppConfig, Audio, Camera2D, Fps, Render, Sprite, TextureStore, Transform};
use glam::{Vec2, Vec3};
use std::sync::Arc;
use std::time::Duration;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::window::Window;

pub struct AppData {
    config: AppConfig,
    input: Input,
    render: Render,
    texture_store: TextureStore,
    audio: Audio,
    camera: Camera2D,
    size: PhysicalSize<u32>,
    if_size_changed: bool,
    fps: Fps,
    /// 因为 web 上运行时，需要玩家点击了窗口后，才能初始化 AudioContext 所以需要检测第一次点击，重新初始化一遍 audio
    #[cfg(target_arch = "wasm32")]
    if_focused: bool,
    ui_cursor: Sprite,
    app_state: AppState,
    next_app_state: Option<AppState>,
    main_menu: MainMenu,
    in_game: InGame,
}

#[derive(Debug, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

impl App for AppData {
    async fn new(window: Arc<Window>) -> Self {
        let render = Render::new(window.clone())
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

        let mut texture_store = TextureStore::default();

        let ui_cursor_image_handle = texture_store.load_texture_raw(&render, UI_CURSOR);
        let ui_cursor = Sprite {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 500.0)),
            texture_id: ui_cursor_image_handle,
            anchor: Vec2::new(-0.5, 0.5),
            ..Default::default()
        };

        let main_menu = MainMenu::new(&render, &mut texture_store, &mut camera);

        Self {
            config: AppConfig::default(),
            input: Input::default(),
            render,
            texture_store,
            audio,
            camera,
            size: window.inner_size(),
            // 默认为 true 确保渲染第一帧前会调整 surface 大小
            if_size_changed: true,
            fps: Fps::new(),
            #[cfg(target_arch = "wasm32")]
            if_focused: false,
            ui_cursor,
            app_state: AppState::default(),
            next_app_state: None,
            main_menu,
            in_game: InGame::default(),
        }
    }

    fn get_config(&self) -> &AppConfig {
        &self.config
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.if_size_changed = true;
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

        if let Some(next_app_state) = self.next_app_state.take() {
            self.app_state = next_app_state;
        }

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
        if self.input.if_keyboard_just_pressed(&KeyCode::KeyP) {
            self.audio.play_sound_with_volume("bgm", 0.4);
        }

        // TODO 需要支持基于屏幕坐标来渲染 ui
        //  同时每个场景应该有自己的 camera 而不是公用一个 camera
        // let world_position = self
        //     .camera
        //     .viewport_to_world(Vec2::new(
        //         self.input.cursor_pos().x as f32,
        //         self.input.cursor_pos().y as f32,
        //     ))
        //     .truncate();
        // self.ui_cursor.transform.translation.x = world_position.x;
        // self.ui_cursor.transform.translation.y = world_position.y;

        match self.app_state {
            AppState::MainMenu => {
                if self.input.if_keyboard_just_pressed(&KeyCode::KeyS) {
                    self.next_app_state = Some(AppState::InGame);
                    self.in_game =
                        InGame::new(&self.render, &mut self.texture_store, &mut self.camera);
                }
            }
            AppState::InGame => {
                self.in_game
                    .update(delta, &self.input, &mut self.audio, &mut self.camera);
            }
        }

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
            match self.app_state {
                AppState::MainMenu => {
                    self.main_menu
                        .render(&self.render, &self.texture_store, &self.camera);
                }
                AppState::InGame => {
                    self.in_game
                        .render(&self.render, &self.texture_store, &self.camera);
                }
            }
        }
        self.fps.update();
        #[cfg(feature = "profiling")]
        profiling::finish_frame!();

        Ok(())
    }
}
