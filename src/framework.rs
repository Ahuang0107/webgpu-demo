pub trait Example: 'static + Sized {
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            shader_model: wgpu::ShaderModel::Sm5,
            ..wgpu::DownlevelCapabilities::default()
        }
    }
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_webgl2_defaults() // These downlevel limits will allow the code to run on all possible hardware
    }
    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;
    fn update(&mut self, event: winit::event::WindowEvent);
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &Spawner,
    );
}

pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

impl<'a> Spawner<'a> {
    fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    pub fn spawn_local(&self, future: impl core::future::Future<Output = ()> + 'a) {
        self.executor.spawn(future).detach();
    }
}

struct Setup {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

async fn setup<E: Example>(title: &str) -> Result<Setup, Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("webgpu_demo"))
        .init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)?;

    log::info!("Initializing the surface...");
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
        .expect("No suitable GPU adapters found on the system!");
    let adapter_info = adapter.get_info();
    log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
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

    Ok(Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    })
}

fn start<E: Example>(
    Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }: Setup,
) {
    let spawner = Spawner::new();
    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .expect("Surface isn't supported by the adapter.");
    surface.configure(&device, &config);

    log::info!("Initializing the example...");
    let mut example = E::init(&config, &adapter, &device, &queue);

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter);
        match event {
            winit::event::Event::RedrawRequested(_) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                example.render(&view, &device, &queue, &spawner);

                frame.present();
            }
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }
    });
}

pub fn run<E: Example>(title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let setup = pollster::block_on(setup::<E>(title));
    start::<E>(setup?);
    Ok(())
}
