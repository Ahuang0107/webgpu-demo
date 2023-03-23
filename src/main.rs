use wgpu::util::DeviceExt;

mod vertex;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("webgpu_demo"))
        .init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop)?;

    log::info!("initializing the surface...");
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });
    let surface = unsafe { instance.create_surface(&window) }?;
    let size = window.inner_size();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("no suitable GPU adapters found on the system!");
    let adapter_info = adapter.get_info();
    log::info!("using {} ({:?})", adapter_info.name, adapter_info.backend);
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await?;

    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .expect("surface isn't supported by the adapter.");
    surface.configure(&device, &config);

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex::Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(config.format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::RedrawRequested(_) => {
            let frame = surface
                .get_current_texture()
                .expect("Failed to acquire next surface texture!");
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            {
                let p0 = (50.0_f32, 50.0);
                let p1 = (50.0_f32, 150.0);
                let p2 = (150.0_f32, 150.0);
                let p3 = (150.0_f32, 50.0);
                let size = window.inner_size();
                let w_2 = (size.width / 2) as f32;
                let h_2 = (size.height / 2) as f32;
                let center = (w_2, h_2);
                let p0_v = vertex::Vertex::new((p0.0 - center.0) / w_2, -(p0.1 - center.1) / h_2);
                let p1_v = vertex::Vertex::new((p1.0 - center.0) / w_2, -(p1.1 - center.1) / h_2);
                let p2_v = vertex::Vertex::new((p2.0 - center.0) / w_2, -(p2.1 - center.1) / h_2);
                let p3_v = vertex::Vertex::new((p3.0 - center.0) / w_2, -(p3.1 - center.1) / h_2);

                let vertices: &[vertex::Vertex] = &[p0_v, p1_v, p2_v, p3_v];
                let indices: &[u16] = &[0, 1, 2, 0, 2, 3];

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                render_pass.set_pipeline(&render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
            queue.submit(Some(encoder.finish()));

            frame.present();
        }
        winit::event::Event::MainEventsCleared => {
            window.request_redraw();
        }
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
            winit::event::WindowEvent::Resized(physical_size) => {
                config.width = physical_size.width;
                config.height = physical_size.height;
                surface.configure(&device, &config);
            }
            _ => {}
        },
        _ => {}
    });
}

fn main() {
    pollster::block_on(run()).expect("");
}
