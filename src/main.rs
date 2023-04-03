mod camera;
mod render;
mod texture;
mod vertex;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("webgpu_demo"))
        .init();

    let event_loop = winit::event_loop::EventLoop::new();
    let size = winit::dpi::PhysicalSize::new(1424, 720);
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)?;

    let mut render = render::Render::new(&window).await?;

    let p0 = (160.0_f32, 160.0);
    let p1 = (160.0_f32, 320.0);
    let p2 = (320.0_f32, 320.0);
    let p3 = (320.0_f32, 160.0);
    let size = window.inner_size();
    let w_2 = (size.width / 2) as f32;
    let h_2 = (size.height / 2) as f32;
    let center = (w_2, h_2);
    let p0_v = vertex::Vertex::new((p0.0 - center.0) / w_2, -(p0.1 - center.1) / h_2, 0.0, 0.0);
    let p1_v = vertex::Vertex::new((p1.0 - center.0) / w_2, -(p1.1 - center.1) / h_2, 0.0, 1.0);
    let p2_v = vertex::Vertex::new((p2.0 - center.0) / w_2, -(p2.1 - center.1) / h_2, 1.0, 1.0);
    let p3_v = vertex::Vertex::new((p3.0 - center.0) / w_2, -(p3.1 - center.1) / h_2, 1.0, 0.0);

    let vertices: [vertex::Vertex; 4] = [p0_v, p1_v, p2_v, p3_v];
    let indices: &[u16] = &[0, 1, 2, 0, 2, 3];

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(_) => {
            render.render(&vertices, &indices);
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

fn main() {
    pollster::block_on(run()).expect("");
}
