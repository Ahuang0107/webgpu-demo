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
        }
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        self.render.resize(new_size.width, new_size.height);
        self.camera.viewport_size = (new_size.width as f32, new_size.height as f32).into();
    }

    fn get_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.render.config.width, self.render.config.height)
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        // 窗口最小化时只更新数据不渲染画面
        if self.camera.viewport_size.x > 0.0 && self.camera.viewport_size.y > 0.0 {
            self.render.render(&self.camera, self.sprites.as_slice());
        }

        Ok(())
    }
}
