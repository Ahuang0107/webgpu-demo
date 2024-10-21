use vertex::*;

mod camera;
mod render;
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

    let data = vec![
        (
            [
                (100.0_f32, 100.0),
                (100.0_f32, 400.0),
                (400.0_f32, 400.0),
                (400.0_f32, 100.0),
            ],
            texture1,
        ),
        (
            [
                (350.0_f32, 100.0),
                (350.0_f32, 364.0),
                (614.0_f32, 364.0),
                (614.0_f32, 100.0),
            ],
            texture2,
        ),
    ];

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(_) => {
            let instances = data
                .iter()
                .map(|(points, texture_id)| {
                    let (vertices, indices) = cal_vertices(points.clone(), window.inner_size());
                    (vertices, indices, *texture_id)
                })
                .collect::<Vec<([Vertex; 4], [u16; 6], u32)>>();
            render.flash_instances(instances);
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
            _ => {}
        },
        _ => {}
    });
}

fn cal_vertices<'a>(
    p: [(f32, f32); 4],
    size: winit::dpi::PhysicalSize<u32>,
) -> ([Vertex; 4], [u16; 6]) {
    let p0 = p[0];
    let p1 = p[1];
    let p2 = p[2];
    let p3 = p[3];
    let w_2 = (size.width / 2) as f32;
    let h_2 = (size.height / 2) as f32;
    let center = (w_2, h_2);
    let p0_v = Vertex::new((p0.0 - center.0) / w_2, -(p0.1 - center.1) / h_2, 0.0, 0.0);
    let p1_v = Vertex::new((p1.0 - center.0) / w_2, -(p1.1 - center.1) / h_2, 0.0, 1.0);
    let p2_v = Vertex::new((p2.0 - center.0) / w_2, -(p2.1 - center.1) / h_2, 1.0, 1.0);
    let p3_v = Vertex::new((p3.0 - center.0) / w_2, -(p3.1 - center.1) / h_2, 1.0, 0.0);

    let vertices: [Vertex; 4] = [p0_v, p1_v, p2_v, p3_v];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    (vertices, indices)
}

fn main() {
    pollster::block_on(run()).expect("");
}
