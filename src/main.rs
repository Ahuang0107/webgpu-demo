#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use glam::{Quat, Vec2, Vec3};
use std::sync::Arc;
use webgpu_demo::*;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

fn main() -> Result<(), impl std::error::Error> {
    println!("{PKG_NAME}");
    run::<AppData>(PKG_NAME)
}

struct AppData {
    render: Render,
    camera: Camera2D,
    sprites: Vec<Sprite>,
    size: PhysicalSize<u32>,
    if_size_changed: bool,
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
        let sprites = vec![
            Sprite {
                transform: Transform {
                    translation: Vec3::new(150.0, 100.0, 1.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                texture_id: example_1,
                rect: None,
                custom_size: None,
                flip_x: false,
                flip_y: false,
                anchor: Vec2::ZERO,
            },
            Sprite {
                transform: Transform {
                    translation: Vec3::new(-200.0, -100.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                texture_id: example_2,
                rect: None,
                custom_size: None,
                flip_x: false,
                flip_y: false,
                anchor: Vec2::ZERO,
            },
        ];

        Self {
            render,
            camera,
            sprites,
            size: window.inner_size(),
            // 默认为 true 确保渲染第一帧前会调整 surface 大小
            if_size_changed: true,
        }
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.if_size_changed = true;
    }

    fn get_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.render.config.width, self.render.config.height)
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        if self.if_size_changed {
            self.camera.viewport_size = (self.size.width as f32, self.size.height as f32).into();
            self.if_size_changed = false;
        }
        // TODO 暂时不清楚为什么，必须要每一帧都调整 surface 大小，才能保证调整窗口尺寸时是丝滑的，否则就会很卡顿
        self.render.resize(self.size.width, self.size.height);
        // 窗口最小化时只更新数据不渲染画面
        if self.size.width > 0 && self.size.height > 0 {
            self.render.render(&self.camera, self.sprites.as_slice());
        }

        Ok(())
    }
}
