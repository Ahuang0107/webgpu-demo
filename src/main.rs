use crate::sprite::Sprite;
use blend_mode::*;

mod blend_mode;
mod camera;
mod render;
mod sprite;
mod texture;
mod vertex;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("webgpu_demo"))
        .init();

    let event_loop = winit::event_loop::EventLoop::new();
    let size = winit::dpi::PhysicalSize::new(256 * 6, 256 * 3);
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)?;

    let mut render = render::Render::new(&window).await?;

    let texture1 = render.load_texture(include_bytes!("example.png"))?;
    let texture2 = render.load_texture(include_bytes!("example2.png"))?;
    let texture3 = render.load_texture(include_bytes!("example3.png"))?;
    let texture4 = render.load_texture(include_bytes!("mask_example.png"))?;

    let sprites = vec![
        Sprite::new([100.0, 100.0], [300.0, 300.0], texture1),
        Sprite::new([100.0, 100.0], [600.0, 400.0], texture4).with_mask_in(),
        Sprite::new([350.0, 100.0], [264.0, 264.0], texture2).with_blend_mode(BlendMode::Normal),
        Sprite::new([100.0, 100.0], [600.0, 400.0], texture4).with_mask_out(),
        Sprite::new([300.0, 300.0], [400.0, 200.0], texture3).with_blend_mode(BlendMode::SoftLight),
    ]
    .into_iter()
    .map(|s| s.with_window_size([window.inner_size().width, window.inner_size().height]))
    .collect::<Vec<Sprite>>();

    render.sprites.extend(sprites);

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(_) => {
            render.render();
        }
        winit::event::Event::MainEventsCleared => {
            window.request_redraw();
        }
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
            winit::event::WindowEvent::Resized(physical_size) => {
                render.resize(physical_size.width, physical_size.height);
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                // println!("CursorMoved: {position:?}");
                render.sprites[1].x = position.x as f32;
                render.sprites[1].y = position.y as f32;
                render.sprites[3].x = position.x as f32;
                render.sprites[3].y = position.y as f32;
            }
            _ => {}
        },
        _ => {}
    });
}

fn main() {
    pollster::block_on(run()).expect("");
}
