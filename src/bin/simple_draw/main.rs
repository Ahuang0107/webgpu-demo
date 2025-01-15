mod render;
mod vertex;

use render::*;
use vertex::*;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("simple_draw=trace"),
    )
    .init();
    pollster::block_on(run()).expect("");
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let size = winit::dpi::PhysicalSize::new(256 * 6, 256 * 3);
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)?;
    let window = std::sync::Arc::new(window);

    let mut render = Render::new(window.clone()).await?;

    // render.set_texture(image::open("./src/example.png")?.to_rgba8());

    log::info!("Entering render loop...");
    let _ = winit::event_loop::EventLoop::run(event_loop, move |event, target| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::RedrawRequested => {
                log::info!("Redraw requested...");
                render.render();
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                window.request_redraw();
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
        _ => {}
    });

    Ok(())
}
