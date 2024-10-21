use std::collections::HashMap;
use std::num::NonZeroU32;
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
    textures: HashMap<u32, crate::texture::Texture>,
    instances: Vec<(wgpu::Buffer, wgpu::Buffer, u32)>,
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

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("surface isn't supported by the adapter.");
        surface.configure(&device, &config);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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
            textures: HashMap::new(),
            instances: Vec::new(),
        })
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        log::info!("resize to ({},{})", width, height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config)
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
    pub fn flash_instances<T: bytemuck::Zeroable + bytemuck::Pod>(
        &mut self,
        instances: Vec<([T; 4], [u16; 6], u32)>,
    ) {
        self.instances.clear();
        for (vertices, indices, texture_id) in instances {
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
                .push((vertex_buffer, index_buffer, texture_id));
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

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_width = self.config.width;
        let texture_height = self.config.height;
        assert_eq!(0, texture_width % 256);
        assert_eq!(0, texture_height % 256);

        let first_grab_texture_desc = wgpu::TextureDescriptor {
            label: Some("Grab Texture"),
            size: wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let first_grab_texture = self.device.create_texture(&first_grab_texture_desc);
        let first_grab_view =
            first_grab_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let u32_size = size_of::<u32>() as u32;
        let output_buffer_size = (u32_size
            * first_grab_texture.width()
            * first_grab_texture.height()) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = self.device.create_buffer(&output_buffer_desc);

        // let second_grab_texture = self.device.create_texture(&wgpu::TextureDescriptor {
        //     label: Some("Grab Texture"),
        //     size: wgpu::Extent3d {
        //         width: self.config.width,
        //         height: self.config.height,
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Bgra8Unorm,
        //     usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        //     view_formats: &[],
        // });
        // let second_grab_view =
        //     second_grab_texture.create_view(&wgpu::TextureViewDescriptor::default());
        // let second_grab_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &self.texture_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&second_grab_view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&sampler),
        //         },
        //     ],
        //     label: None,
        // });

        let empty_grab_texture_desc = wgpu::TextureDescriptor {
            label: Some("Grab Texture"),
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
        };
        let empty_grab_texture = self.device.create_texture(&empty_grab_texture_desc);
        let empty_grab_view =
            empty_grab_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let empty_grab_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&empty_grab_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &first_grab_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let (vertex_buffer, index_buffer, texture_id) = &self.instances[0];
            if let Some(texture) = self.textures.get(texture_id) {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &camera_bind_group, &[]);
                render_pass.set_bind_group(1, &texture.bind_group, &[]);
                render_pass.set_bind_group(2, &empty_grab_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &first_grab_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(u32_size * first_grab_texture.width()),
                    rows_per_image: NonZeroU32::new(first_grab_texture.height()),
                },
            },
            first_grab_texture_desc.size,
        );

        self.queue.submit(Some(encoder.finish()));

        // log::info!("first_grab_view: {first_grab_view:?}");
        // let first_grab_image = first_grab_texture.as_image_copy();
        // log::info!(
        //     "first_grab_texture: {:?}",
        //     first_grab_texture.as_image_copy()
        // );
        // let first_grab_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &self.texture_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&first_grab_view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&sampler),
        //         },
        //     ],
        //     label: None,
        // });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let (vertex_buffer, index_buffer, texture_id) = &self.instances[1];
            if let Some(texture) = self.textures.get(texture_id) {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &camera_bind_group, &[]);
                render_pass.set_bind_group(1, &texture.bind_group, &[]);
                render_pass.set_bind_group(2, &empty_grab_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));

        frame.present();

        // 首先这里确实能够得到第一个pass的渲染结果，但是依旧存在两个问题：
        // 1. 这个渲染结果就不在屏幕上了，只在导出的 texture 上了。
        // 2. 并且想要的得到 texture 上的内容只能异步的得到。
        let buffer_slice = output_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |_result| {});
        self.device.poll(wgpu::Maintain::Wait);
        let data = buffer_slice.get_mapped_range();
        let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            first_grab_texture.width(),
            first_grab_texture.height(),
            data,
        )
        .unwrap();
        buffer.save("image.png").unwrap();
    }
}
