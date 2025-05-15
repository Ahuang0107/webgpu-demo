#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use glam::{Affine3A, Mat4, Quat, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;
use winit::keyboard::{KeyCode, PhysicalKey};

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("basic_draw=trace"))
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
    let sprite = Sprite {
        transform: Transform::IDENTITY,
        image_size: Vec2::new(256.0, 256.0),
        rect: None,
        custom_size: None,
        flip_x: false,
        flip_y: false,
        anchor: Vec2::ZERO,
    };

    log::info!("Entering render loop...");
    let _ = winit::event_loop::EventLoop::run(event_loop, move |event, target| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::RedrawRequested => {
                // log::info!("Redraw requested...");
                render.render(&camera, &sprite);
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

pub struct Render {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    view_uniform_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(unused)]
    texture_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
}

pub struct Camera2D {
    /// Specifies the origin of the viewport as a normalized position from 0 to 1, where (0, 0) is the bottom left
    /// and (1, 1) is the top right. This determines where the camera's position sits inside the viewport.
    ///
    /// When the projection scales due to viewport resizing, the position of the camera, and thereby `viewport_origin`,
    /// remains at the same relative point.
    ///
    /// Consequently, this is pivot point when scaling. With a bottom left pivot, the projection will expand
    /// upwards and to the right. With a top right pivot, the projection will expand downwards and to the left.
    /// Values in between will caused the projection to scale proportionally on each axis.
    ///
    /// Defaults to `(0.5, 0.5)`, which makes scaling affect opposite sides equally, keeping the center
    /// point of the viewport centered.
    ///
    /// 来自 bevy 的 OrthographicProjection 中的 viewport_origin
    pub viewport_origin: Vec2,
    pub viewport_size: Vec2,
    pub transform: Transform,
}

impl Camera2D {
    fn new(viewport_size: Vec2) -> Camera2D {
        Camera2D {
            viewport_origin: Vec2::splat(0.5),
            viewport_size,
            transform: Transform::IDENTITY,
        }
    }
    fn area(&self) -> Rect {
        let origin_x = self.viewport_size.x * self.viewport_origin.x;
        let origin_y = self.viewport_size.y * self.viewport_origin.y;

        Rect::new(
            -origin_x,
            -origin_y,
            self.viewport_size.x - origin_x,
            self.viewport_size.y - origin_y,
        )
    }
    fn get_clip_from_view(&self) -> Mat4 {
        let area = self.area();
        Mat4::orthographic_rh(area.min.x, area.max.x, area.min.y, area.max.y, 1000.0, 0.0)
    }
    fn get_world_from_view(&self) -> Mat4 {
        self.transform.compute_matrix()
    }
    fn get_view_from_world(&self) -> Mat4 {
        self.get_world_from_view().inverse()
    }
    pub fn get_clip_from_world(&self) -> Mat4 {
        self.get_clip_from_view() * self.get_view_from_world()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniform {
    clip_from_world: Mat4,
    /// viewport_origin(default = `[0.5, 0.5]`) + viewport_size(default = window_size)
    viewport: Vec4,
}

pub struct Sprite {
    pub transform: Transform,
    pub image_size: Vec2,
    /// Select an area of the texture
    pub rect: Option<Rect>,
    /// Change the on-screen size of the sprite
    pub custom_size: Option<Vec2>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor: Vec2,
}

pub struct Transform {
    /// Position of the entity. In 2d, the last value of the `Vec3` is used for z-ordering.
    pub translation: Vec3,
    /// Rotation of the entity.
    pub rotation: Quat,
    /// Scale of the entity.
    pub scale: Vec3,
}

impl Transform {
    /// An identity [`Transform`] with no translation, rotation, and a scale of 1 on all axes.
    pub const IDENTITY: Self = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Returns the 3d affine transformation matrix from this transforms translation,
    /// rotation, and scale.
    #[inline]
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Returns the 3d affine transformation matrix from this transforms translation,
    /// rotation, and scale.
    #[inline]
    pub fn compute_affine(&self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Sprite {
    pub fn calculate_transform(&self) -> Affine3A {
        let quad_size = self
            .custom_size
            .unwrap_or_else(|| self.rect.map(|r| r.size()).unwrap_or(self.image_size));

        self.transform.compute_affine()
            * Affine3A::from_scale_rotation_translation(
                quad_size.extend(1.0),
                Quat::IDENTITY,
                (quad_size * (-self.anchor - Vec2::splat(0.5))).extend(0.0),
            )
    }
    pub fn calculate_uv_offset_scale(&self) -> Vec4 {
        let mut uv_offset_scale: Vec4;

        // If a rect is specified, adjust UVs and the size of the quad
        if let Some(rect) = self.rect {
            let rect_size = rect.size();
            uv_offset_scale = Vec4::new(
                rect.min.x / self.image_size.x,
                rect.max.y / self.image_size.y,
                rect_size.x / self.image_size.x,
                -rect_size.y / self.image_size.y,
            );
        } else {
            uv_offset_scale = Vec4::new(0.0, 1.0, 1.0, -1.0);
        }

        if self.flip_x {
            uv_offset_scale.x += uv_offset_scale.z;
            uv_offset_scale.z *= -1.0;
        }
        if self.flip_y {
            uv_offset_scale.y += uv_offset_scale.w;
            uv_offset_scale.w *= -1.0;
        }

        uv_offset_scale
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    #[inline]
    pub fn from_corners(p0: Vec2, p1: Vec2) -> Self {
        Self {
            min: p0.min(p1),
            max: p0.max(p1),
        }
    }
    #[inline]
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::from_corners(Vec2::new(x0, y0), Vec2::new(x1, y1))
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteInstance {
    // Affine 4x3 transposed to 3x4
    pub i_model_transpose: [Vec4; 3],
    pub i_uv_offset_scale: [f32; 4],
}

impl SpriteInstance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x4,
        1 => Float32x4,
        2 => Float32x4,
        3 => Float32x4,
    ];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
    #[inline]
    fn from(transform: &Affine3A, uv_offset_scale: &Vec4) -> Self {
        let transpose_model_3x3 = transform.matrix3.transpose();
        Self {
            i_model_transpose: [
                transpose_model_3x3.x_axis.extend(transform.translation.x),
                transpose_model_3x3.y_axis.extend(transform.translation.y),
                transpose_model_3x3.z_axis.extend(transform.translation.z),
            ],
            i_uv_offset_scale: uv_offset_scale.to_array(),
        }
    }
}

impl Render {
    pub async fn new(
        window: std::sync::Arc<winit::window::Window>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("initializing the surface...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no suitable GPU adapters found on the system!");
        let adapter_info = adapter.get_info();
        log::info!("adapter info: {adapter_info:?}");
        let caps = surface.get_capabilities(&adapter);
        log::info!("capabilities: {caps:?}");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    label: None,
                },
                None,
            )
            .await?;
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let view_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &view_uniform_bind_group_layout,
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[SpriteInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let image_bytes = include_bytes!("example.png");
        let image = image::load_from_memory(image_bytes).unwrap();
        let image = image.to_rgba8();
        let image_size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Example Texture"),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: Some(image.height()),
            },
            image_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        Ok(Render {
            surface,
            device,
            queue,
            config,
            view_uniform_bind_group_layout,
            texture_bind_group_layout,
            render_pipeline,
            texture_bind_group,
        })
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        log::info!("resize to ({},{})", width, height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
    pub fn render(&mut self, camera: &Camera2D, sprite: &Sprite) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture!");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let clip_from_world = camera.get_clip_from_world();
        let view_uniform = ViewUniform {
            clip_from_world,
            viewport: Vec4::new(
                camera.viewport_origin.x,
                camera.viewport_origin.y,
                camera.viewport_size.x,
                camera.viewport_size.y,
            ),
        };
        let view_uniform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    contents: bytemuck::cast_slice(&[view_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    label: None,
                });
        let view_uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.view_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let sprite_instance = SpriteInstance::from(
            &sprite.calculate_transform(),
            &sprite.calculate_uv_offset_scale(),
        );
        // let sprite_instances = vec![sprite_instance];
        // let sprite_instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Vertex Buffer"),
        //     size: size_of::<SpriteInstance>() as wgpu::BufferAddress,
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        //     mapped_at_creation: false,
        // });
        // let bytes: &[u8] = bytemuck::must_cast_slice(&sprite_instances);
        // self.queue
        //     .write_buffer(&sprite_instance_buffer, 0, &bytes[0..]);

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&[sprite_instance]),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                // NOTE 注意渲染结果一直有问题的原因在，这里的 index 数据的数据类型，是需要跟
                //  render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                //  时的 IndexFormat 保持一直的，之前 IndexFormat 定义是 Uint16 但是这里给的 index 数据并没有指定类型
                //  默认情况下是 i32 类型的，就出现了隐式数据对齐问题，指定 u16 就没有问题了
                contents: bytemuck::cast_slice(&[2_u32, 0, 1, 1, 3, 2]),
                usage: wgpu::BufferUsages::INDEX,
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &view_uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6_u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        frame.present();
    }
}
