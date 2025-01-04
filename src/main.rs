use crate::sprite::Sprite;
use blend_mode::*;
use vertex::*;

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

    let pos1 = (100.0, 100.0);
    let size1 = (300.0, 300.0);
    let pos2 = (350.0, 100.0);
    let size2 = (264.0, 264.0);
    let pos3 = (300.0, 300.0);
    let size3 = (400.0, 200.0);
    let data = vec![
        (
            [
                pos1,
                (pos1.0, pos1.1 + size1.1),
                (pos1.0 + size1.0, pos1.1 + size1.1),
                (pos1.0 + size1.0, pos1.1),
            ],
            BlendMode::Normal,
            texture1,
        ),
        (
            [
                pos2,
                (pos2.0, pos2.1 + size2.1),
                (pos2.0 + size2.0, pos2.1 + size2.1),
                (pos2.0 + size2.0, pos2.1),
            ],
            BlendMode::SoftLight,
            texture2,
        ),
        (
            [
                pos3,
                (pos3.0, pos3.1 + size3.1),
                (pos3.0 + size3.0, pos3.1 + size3.1),
                (pos3.0 + size3.0, pos3.1),
            ],
            BlendMode::SoftLight,
            texture3,
        ),
    ];

    let instances = data
        .iter()
        .map(|(points, blend_mode, texture_id)| {
            let (vertices, indices) =
                cal_vertices(points.clone(), window.inner_size(), *blend_mode);
            Sprite {
                vertices,
                indices,
                texture_id: *texture_id,
                blend_mode: *blend_mode,
                if_mask: false,
            }
        })
        .collect::<Vec<Sprite>>();

    render.flash_instances(instances);
    render.render();

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(_) => {}
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
            _ => {}
        },
        _ => {}
    });
}

fn cal_vertices<'a>(
    p: [(f32, f32); 4],
    size: winit::dpi::PhysicalSize<u32>,
    blend_mode: BlendMode,
) -> ([Vertex; 4], [u16; 6]) {
    let p0 = p[0];
    let p1 = p[1];
    let p2 = p[2];
    let p3 = p[3];
    let w_2 = (size.width / 2) as f32;
    let h_2 = (size.height / 2) as f32;
    let center = (w_2, h_2);
    let p0_v = Vertex::new(
        (p0.0 - center.0) / w_2,
        -(p0.1 - center.1) / h_2,
        0.0,
        0.0,
        blend_mode,
    );
    let p1_v = Vertex::new(
        (p1.0 - center.0) / w_2,
        -(p1.1 - center.1) / h_2,
        0.0,
        1.0,
        blend_mode,
    );
    let p2_v = Vertex::new(
        (p2.0 - center.0) / w_2,
        -(p2.1 - center.1) / h_2,
        1.0,
        1.0,
        blend_mode,
    );
    let p3_v = Vertex::new(
        (p3.0 - center.0) / w_2,
        -(p3.1 - center.1) / h_2,
        1.0,
        0.0,
        blend_mode,
    );

    let vertices: [Vertex; 4] = [p0_v, p1_v, p2_v, p3_v];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    (vertices, indices)
}

fn main() {
    pollster::block_on(run()).expect("");
}
