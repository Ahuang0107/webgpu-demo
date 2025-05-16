use crate::{run, App, Camera2D, Fps, Rect, Render, Sprite, Transform, PKG_NAME};
use glam::{Vec2, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::SurfaceError;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::KeyEvent;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() {
    run::<AppData>(PKG_NAME).expect("TODO: panic message");
}

struct AppData {
    render: Render,
    camera: Camera2D,
    sprites: Vec<Sprite>,
    size: PhysicalSize<u32>,
    if_size_changed: bool,
    fps: Fps,
}

impl App for AppData {
    async fn new(window: Arc<Window>) -> Self {
        let mut render = Render::new(window.clone())
            .await
            .expect("Failed to create render");

        let camera = Camera2D::new(Vec2::new(
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        ));
        let example_1 = render.load_texture(include_bytes!("example.png"));
        let example_2 = render.load_texture(include_bytes!("example2.png"));
        // TODO 当窗口尺寸为奇数时，会因为浮点数精度问题，导致渲染出来的 sprite slice不完整
        let font = render.load_texture(include_bytes!("monogram-bitmap.png"));
        let font_map: HashMap<char, Rect> = HashMap::from([
            ('0', Rect::new(0.0, 12.0, 6.0, 24.0)),
            ('1', Rect::new(6.0, 12.0, 12.0, 24.0)),
            ('2', Rect::new(12.0, 12.0, 18.0, 24.0)),
            ('3', Rect::new(18.0, 12.0, 24.0, 24.0)),
            ('4', Rect::new(24.0, 12.0, 30.0, 24.0)),
            ('5', Rect::new(30.0, 12.0, 36.0, 24.0)),
            ('6', Rect::new(36.0, 12.0, 42.0, 24.0)),
            ('7', Rect::new(42.0, 12.0, 48.0, 24.0)),
            ('8', Rect::new(48.0, 12.0, 54.0, 24.0)),
            ('9', Rect::new(54.0, 12.0, 60.0, 24.0)),
        ]);
        let sprites = vec![
            Sprite {
                transform: Transform::from_translation(Vec3::new(150.0, 100.0, 1.0)),
                texture_id: example_1,
                ..Default::default()
            },
            Sprite {
                transform: Transform::from_translation(Vec3::new(-200.0, -100.0, 0.0)),
                texture_id: example_2,
                ..Default::default()
            },
            Sprite {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
                texture_id: font,
                rect: Some(font_map.get(&'5').unwrap().clone()),
                ..Default::default()
            },
            Sprite {
                transform: Transform::from_translation(Vec3::new(6.0, 0.0, 100.0)),
                texture_id: font,
                rect: Some(font_map.get(&'2').unwrap().clone()),
                ..Default::default()
            },
            Sprite {
                transform: Transform::from_translation(Vec3::new(12.0, 0.0, 100.0)),
                texture_id: font,
                rect: Some(font_map.get(&'0').unwrap().clone()),
                ..Default::default()
            },
        ];

        Self {
            render,
            camera,
            sprites,
            size: window.inner_size(),
            // 默认为 true 确保渲染第一帧前会调整 surface 大小
            if_size_changed: true,
            fps: Fps::new(),
        }
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.if_size_changed = true;
    }

    fn get_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.render.config.width, self.render.config.height)
    }

    fn keyboard_input(&mut self, event: &KeyEvent) -> bool {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            let camera = &mut self.camera;
            match key_code {
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
                _ => {}
            }
        }
        false
    }

    fn cursor_move(&mut self, _position: PhysicalPosition<f64>) -> bool {
        false
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
            self.render.render(&self.camera, self.sprites.as_slice());
        }
        self.fps.update();
        #[cfg(feature = "profiling")]
        profiling::finish_frame!();

        Ok(())
    }
}
