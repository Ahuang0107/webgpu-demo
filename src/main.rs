use crate::sprite::Sprite;
use blend_mode::*;
use std::sync::Arc;
use winit::event::StartCause;
use winit::event_loop::EventLoop;

mod blend_mode;
mod camera;
mod render;
mod sprite;
mod texture;
mod vertex;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("webgpu_demo"))
        .init();

    let event_loop = winit::event_loop::EventLoop::new()?;
    let size = winit::dpi::PhysicalSize::new(256 * 6, 256 * 3);
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)?;
    let window = Arc::new(window);

    let mut render = render::Render::new(window.clone()).await?;

    let texture1 = render.load_texture(include_bytes!("example.png"))?;
    let texture2 = render.load_texture(include_bytes!("example2.png"))?;
    let texture3 = render.load_texture(include_bytes!("example3.png"))?;
    let texture4 = render.load_texture(include_bytes!("mask_example.png"))?;
    let texture5 = render.load_texture(include_bytes!("black_20_alpha_text.png"))?;
    let texture6 = render.load_texture(include_bytes!("black_test.png"))?;

    let sprites = vec![
        Sprite::new([100.0, 100.0], [300.0, 300.0], texture1),
        Sprite::new([100.0, 100.0], [600.0, 400.0], texture4).with_mask_in(),
        Sprite::new([350.0, 100.0], [264.0, 264.0], texture2).with_blend_mode(BlendMode::Normal),
        Sprite::new([100.0, 100.0], [600.0, 400.0], texture4).with_mask_out(),
        Sprite::new([300.0, 300.0], [400.0, 200.0], texture3).with_blend_mode(BlendMode::SoftLight),
        Sprite::new([700.0, 300.0], [64.0, 64.0], texture5),
        Sprite::new([800.0, 300.0], [64.0, 64.0], texture6).with_opacity(51),
        Sprite::new([150.0, 250.0], [450.0, 100.0], 0).with_blend_mode(BlendMode::Blur),
    ]
    .into_iter()
    .map(|s| s.with_window_size([window.inner_size().width, window.inner_size().height]))
    .collect::<Vec<Sprite>>();

    render.sprites.extend(sprites);

    let mut last_frame_time = std::time::Instant::now(); // 上一帧的时间
    let mut frame_count = 0; // 渲染的帧数
    let mut total_render_time = std::time::Duration::new(0, 0); // 总渲染时间

    log::info!("Entering render loop...");
    let _ = EventLoop::run(event_loop, move |event, target| match event {
        winit::event::Event::NewEvents(StartCause::Init) => {}
        winit::event::Event::NewEvents(StartCause::Poll) => {}
        winit::event::Event::Resumed => {}
        winit::event::Event::Suspended => {}
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::RedrawRequested => {
                let frame_start = std::time::Instant::now();
                render.render();
                let render_duration = frame_start.elapsed();
                total_render_time += render_duration;
                frame_count += 1;
                if last_frame_time.elapsed() >= std::time::Duration::new(1, 0) {
                    let fps = frame_count as f32 / last_frame_time.elapsed().as_secs_f32();
                    println!("FPS: {:.2}", fps);
                    println!(
                        "Average render time per frame: {:.2}ms",
                        total_render_time.as_secs_f32() * 1000.0 / frame_count as f32
                    );
                    last_frame_time = std::time::Instant::now();
                    frame_count = 0;
                    total_render_time = std::time::Duration::new(0, 0);
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                target.exit();
            }
            winit::event::WindowEvent::Resized(physical_size) => {
                render.resize(physical_size.width, physical_size.height);
                window.request_redraw();
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                // println!("CursorMoved: {position:?}");
                // render.sprites[1].x = position.x as f32;
                // render.sprites[1].y = position.y as f32;
                // render.sprites[3].x = position.x as f32;
                // render.sprites[3].y = position.y as f32;
                render.sprites[7].x = position.x as f32;
                render.sprites[7].y = position.y as f32;
                window.request_redraw();
            }
            _ => {}
        },
        _ => {}
    });

    Ok(())
}

fn main() {
    pollster::block_on(run()).expect("");
}
