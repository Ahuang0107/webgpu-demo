#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use glam::{Quat, Vec2, Vec3};
use webgpu_demo::{Camera2D, Render, Sprite, Transform};
use winit::keyboard::{KeyCode, PhysicalKey};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("webgpu_demo=trace"),
    )
    .init();
    pollster::block_on(run()).expect("run failed.");
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let size = winit::dpi::PhysicalSize::new(1920, 1080);
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)?;
    let window = std::sync::Arc::new(window);

    let mut render = Render::new(window.clone()).await?;
    let mut camera = Camera2D::new(Vec2::new(size.width as f32, size.height as f32));
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

    log::info!("Entering render loop...");
    let _ = winit::event_loop::EventLoop::run(event_loop, move |event, target| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::RedrawRequested => {
                // log::info!("Redraw requested...");
                render.render(&camera, sprites.as_slice());
            }
            winit::event::WindowEvent::CursorMoved { .. } => {
                window.request_redraw();
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
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
                    window.request_redraw();
                }
            }
            winit::event::WindowEvent::Resized(physical_size) => {
                render.resize(physical_size.width, physical_size.height);
                window.request_redraw();
            }
            winit::event::WindowEvent::CloseRequested => {
                target.exit();
            }
            _ => {}
        },
        winit::event::Event::DeviceEvent { event, .. } => match event {
            winit::event::DeviceEvent::Key(_) => {}
            _ => {}
        },
        _ => {}
    });

    Ok(())
}
