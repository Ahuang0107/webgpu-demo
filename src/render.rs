use std::collections::HashMap;
use std::fmt::Debug;
use wgpu::util::DeviceExt;

pub struct Render {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    camera: crate::camera::Camera,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    grab_texture: wgpu::Texture,
    grab_texture_bind_group: wgpu::BindGroup,
    empty_texture_bind_group: wgpu::BindGroup,
    textures: HashMap<u32, crate::texture::Texture>,
    instances: Vec<(wgpu::Buffer, wgpu::Buffer, u32, u32)>,
}

impl Render {
    pub async fn new(window: &winit::window::Window) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("initializing the surface...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window) }?;
        let size = window.inner_size();
        let camera = crate::camera::Camera::new(size.width, size.height);
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

        let caps = surface.get_capabilities(&adapter);
        log::info!("caps: {caps:?}");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let camera_bind_group_layout =
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
        let grab_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Grab Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let grab_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &grab_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &device.create_sampler(&wgpu::SamplerDescriptor::default()),
                    ),
                },
            ],
            label: None,
        });

        let empty_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Empty Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let empty_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &empty_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &device.create_sampler(&wgpu::SamplerDescriptor::default()),
                    ),
                },
            ],
            label: None,
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[crate::vertex::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            camera,
            camera_bind_group_layout,
            texture_bind_group_layout,
            grab_texture,
            grab_texture_bind_group,
            empty_texture_bind_group,
            textures: HashMap::new(),
            instances: Vec::new(),
        })
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        log::info!("resize to ({},{})", width, height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.camera.resize(width, height);
        let grab_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Grab Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let grab_texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &grab_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &self
                            .device
                            .create_sampler(&wgpu::SamplerDescriptor::default()),
                    ),
                },
            ],
            label: None,
        });
        self.grab_texture = grab_texture;
        self.grab_texture_bind_group = grab_texture_bind_group;
    }
    pub fn load_texture(
        &mut self,
        texture_bytes: &[u8],
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let texture = crate::texture::Texture::from_bytes(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            texture_bytes,
        )?;
        let next_id = (self.textures.len() + 1) as u32;
        self.textures.insert(next_id, texture);
        Ok(next_id)
    }
    pub fn flash_instances<T: bytemuck::Zeroable + bytemuck::Pod + Debug>(
        &mut self,
        instances: Vec<([T; 4], [u16; 6], u32, u32)>,
    ) {
        self.instances.clear();
        for (vertices, indices, texture_id, blend_mode) in instances {
            // log::info!("vertices: {vertices:?}");
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
            self.instances
                .push((vertex_buffer, index_buffer, texture_id, blend_mode));
        }
    }
    pub fn render(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture!");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let camera_uniform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    contents: bytemuck::cast_slice(&[self.camera.build_view_projection_matrix()]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    label: None,
                });
        let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            for (index, (vertex_buffer, index_buffer, texture_id, blend_mode)) in
                self.instances.iter().enumerate()
            {
                {
                    // TODO find the problem, in bevy all the command submit after RenderGraphRunner run all graphs
                    //  I can not do the copy staff during in any RenderPhases
                    // let mut encoder = self
                    //     .device
                    //     .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    if *blend_mode != 0 {
                        encoder.copy_texture_to_texture(
                            frame.texture.as_image_copy(),
                            self.grab_texture.as_image_copy(),
                            wgpu::Extent3d {
                                width: self.grab_texture.width(),
                                height: self.grab_texture.height(),
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                    // self.queue.submit(Some(encoder.finish()));
                }
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if index == 0 {
                                wgpu::LoadOp::Clear(wgpu::Color::WHITE)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                if let Some(texture) = self.textures.get(texture_id) {
                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.set_bind_group(0, &camera_bind_group, &[]);
                    render_pass.set_bind_group(1, &texture.bind_group, &[]);
                    if *blend_mode != 0 {
                        render_pass.set_bind_group(2, &self.grab_texture_bind_group, &[]);
                    } else {
                        render_pass.set_bind_group(2, &self.empty_texture_bind_group, &[]);
                    }
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..1);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
