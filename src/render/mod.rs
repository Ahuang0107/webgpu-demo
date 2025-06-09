mod blend_mode;
mod camera;
mod color;
mod pipeline;
mod rect;
mod render_item;
mod screen_repeat;
mod sprite;
mod sprite_instance;
mod texture_store;
mod transform;
mod ui_sprite;

pub use blend_mode::*;
pub use camera::*;
pub use color::*;
pub use pipeline::*;
pub use rect::*;
pub use render_item::*;
pub use screen_repeat::*;
pub use sprite::*;
pub use sprite_instance::*;
pub use texture_store::TextureStore;
pub use transform::*;
pub use ui_sprite::*;

use wgpu::util::DeviceExt;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub const MASK_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Stencil8;

pub struct Render {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    mask_texture: wgpu::Texture,
    grab_texture: wgpu::Texture,
    grab_texture_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    blend_mode_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    mask_start_pipeline: wgpu::RenderPipeline,
    mask_end_pipeline: wgpu::RenderPipeline,
    screen_repeat_pipeline: wgpu::RenderPipeline,
}

impl Render {
    pub async fn new(
        window: std::sync::Arc<winit::window::Window>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("initializing the surface...");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            #[cfg(feature = "profiling")]
            flags: wgpu::InstanceFlags::DEBUG,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
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
                    ..Default::default()
                },
                None,
            )
            .await?;
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        if config.width > 0 && config.height > 0 {
            surface.configure(&device, &config);
        }

        let uniform_bind_group_layout =
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
        let mask_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Mask Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: MASK_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
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
            format: TEXTURE_FORMAT,
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

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline = create_pipeline(
            "Render Pipeline",
            &device,
            &[&uniform_bind_group_layout, &texture_bind_group_layout],
            &shader,
            &[SpriteInstance::desc()],
            wgpu::ColorTargetState {
                format: TEXTURE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            Some(wgpu::DepthStencilState {
                format: MASK_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
        );
        let blend_mode_shader =
            device.create_shader_module(wgpu::include_wgsl!("blend_mode_shader.wgsl"));
        let blend_mode_pipeline = create_pipeline(
            "Blend Mode Pipeline",
            &device,
            &[
                &uniform_bind_group_layout,
                &texture_bind_group_layout,
                &texture_bind_group_layout,
            ],
            &blend_mode_shader,
            &[SpriteInstance::desc()],
            wgpu::ColorTargetState {
                format: TEXTURE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            Some(wgpu::DepthStencilState {
                format: MASK_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
        );
        let blur_shader = device.create_shader_module(wgpu::include_wgsl!("blur_shader.wgsl"));
        let blur_pipeline = create_pipeline(
            "Blur Pipeline",
            &device,
            &[
                &uniform_bind_group_layout,
                &texture_bind_group_layout,
                &texture_bind_group_layout,
            ],
            &blur_shader,
            &[SpriteInstance::desc()],
            wgpu::ColorTargetState {
                format: TEXTURE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            Some(wgpu::DepthStencilState {
                format: MASK_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
        );
        let mask_shader = device.create_shader_module(wgpu::include_wgsl!("mask_shader.wgsl"));
        let mask_start_pipeline = create_pipeline(
            "Mask Start Pipeline",
            &device,
            &[&uniform_bind_group_layout, &texture_bind_group_layout],
            &mask_shader,
            &[SpriteInstance::desc()],
            wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::empty(),
            },
            Some(wgpu::DepthStencilState {
                format: MASK_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        pass_op: wgpu::StencilOperation::IncrementClamp,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: Default::default(),
            }),
        );
        let mask_end_pipeline = create_pipeline(
            "Mask End Pipeline",
            &device,
            &[&uniform_bind_group_layout, &texture_bind_group_layout],
            &mask_shader,
            &[SpriteInstance::desc()],
            wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::empty(),
            },
            Some(wgpu::DepthStencilState {
                format: MASK_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        pass_op: wgpu::StencilOperation::DecrementClamp,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: Default::default(),
            }),
        );
        let screen_repeat_shader =
            device.create_shader_module(wgpu::include_wgsl!("screen_repeat_shader.wgsl"));
        let screen_repeat_pipeline = create_pipeline(
            "Screen Repeat Pipeline",
            &device,
            &[&uniform_bind_group_layout, &texture_bind_group_layout],
            &screen_repeat_shader,
            &[],
            wgpu::ColorTargetState {
                format: TEXTURE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            None,
        );

        Ok(Render {
            surface,
            device,
            queue,
            config,
            uniform_bind_group_layout,
            texture_bind_group_layout,
            mask_texture,
            grab_texture,
            grab_texture_bind_group,
            render_pipeline,
            blend_mode_pipeline,
            blur_pipeline,
            mask_start_pipeline,
            mask_end_pipeline,
            screen_repeat_pipeline,
        })
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        if self.config.width > 0 && self.config.height > 0 {
            self.surface.configure(&self.device, &self.config);

            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let mask_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Mask Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: MASK_TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let grab_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Grab Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let grab_texture_bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

            self.mask_texture = mask_texture;
            self.grab_texture = grab_texture;
            self.grab_texture_bind_group = grab_texture_bind_group;
        }
    }
    pub fn render(
        &self,
        texture_store: &TextureStore,
        camera: &Camera2D,
        sprites: &Vec<&Sprite>,
        screen_repeat: Option<&ScreenRepeat>,
        #[cfg(feature = "editor_mode")] egui_render: &mut crate::egui_render::EguiRender,
    ) {
        #[cfg(feature = "profiling")]
        profiling::scope!("Create Frame View");
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture!");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mask_view = self
            .mask_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        #[cfg(feature = "profiling")]
        profiling::scope!("Create ViewUniform Bind Group");
        let view_uniform = camera.get_view_uniform();
        let view_uniform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    contents: bytemuck::cast_slice(&[view_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    label: None,
                });
        let view_uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        #[cfg(feature = "profiling")]
        profiling::scope!("Create Command Encoder");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder (Tracy)"),
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&[2_u32, 0, 1, 1, 3, 2]),
                usage: wgpu::BufferUsages::INDEX,
            });

        #[cfg(feature = "profiling")]
        profiling::scope!("Convert Sprites");
        let mut render_items: Vec<RenderItem> = Vec::with_capacity(sprites.len());
        let mut sprite_instances: Vec<SpriteInstance> = Vec::with_capacity(sprites.len());
        for (index, sprite) in sprites.into_iter().enumerate() {
            let index = index as u32;
            if let Some((image_size, _)) = texture_store.get(&sprite.texture_id) {
                let sprite_instance = SpriteInstance::from(
                    &sprite.calculate_transform(*image_size),
                    &sprite.calculate_uv_offset_scale(*image_size),
                    sprite.color,
                    sprite.color_blend_mode,
                    sprite.blend_mode,
                );
                sprite_instances.push(sprite_instance);
                match sprite.blend_mode {
                    BlendMode::Normal => {
                        if let Some([mask_start, mask_end]) = sprite.mask {
                            render_items.push(RenderItem::SpriteMaskStart {
                                range: index..index + 1,
                                texture_id: sprite.texture_id,
                                sort_key: mask_start,
                            });
                            render_items.push(RenderItem::SpriteMaskEnd {
                                range: index..index + 1,
                                texture_id: sprite.texture_id,
                                sort_key: mask_end,
                            });
                        } else {
                            render_items.push(RenderItem::Sprite {
                                range: index..index + 1,
                                texture_id: sprite.texture_id,
                                sort_key: sprite.transform.translation.z,
                            });
                        }
                    }
                    BlendMode::Blur => {
                        render_items.push(RenderItem::BlurSprite {
                            range: index..index + 1,
                            texture_id: sprite.texture_id,
                            sort_key: sprite.transform.translation.z,
                        });
                    }
                    _ => {
                        render_items.push(RenderItem::BlendModeSprite {
                            range: index..index + 1,
                            texture_id: sprite.texture_id,
                            sort_key: sprite.transform.translation.z,
                        });
                    }
                }
            } else {
                log::warn!("Unable to find texture({})", sprite.texture_id);
            }
        }
        let buffer_size = size_of::<SpriteInstance>() * sprite_instances.len();
        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let range = 0..buffer_size;
        let bytes: &[u8] = bytemuck::cast_slice(&sprite_instances);
        self.queue.write_buffer(&vertex_buffer, 0, &bytes[range]);

        #[cfg(feature = "profiling")]
        profiling::scope!("Sort Render Items");
        radsort::sort_by_key(&mut render_items, |item| (item.sort_key(), item.type_key()));

        {
            if let Some(screen_repeat) = screen_repeat {
                if let Some((_, texture)) = texture_store.get(&screen_repeat.texture_id) {
                    let screen_repeat_uniform = screen_repeat.get_uniform(camera);
                    let screen_repeat_uniform_buffer =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                contents: bytemuck::cast_slice(&[screen_repeat_uniform]),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                                label: None,
                            });
                    let screen_repeat_uniform_bind_group =
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.uniform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: screen_repeat_uniform_buffer.as_entire_binding(),
                            }],
                            label: None,
                        });

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
                    render_pass.set_pipeline(&self.screen_repeat_pipeline);
                    render_pass.set_bind_group(0, &screen_repeat_uniform_bind_group, &[]);
                    render_pass.set_bind_group(1, texture, &[]);
                    render_pass.draw(0..6_u32, 0..1);
                }
            }
        }

        {
            #[cfg(feature = "profiling")]
            profiling::scope!("Begin Render Pass");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &mask_view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_stencil_reference(0);

            #[cfg(feature = "profiling")]
            profiling::scope!("Draw Items");
            for render_item in render_items {
                if let Some((_, texture)) = texture_store.get(&render_item.texture_id()) {
                    match render_item {
                        RenderItem::Sprite { .. } => {
                            render_pass.set_pipeline(&self.render_pipeline);
                        }
                        RenderItem::BlendModeSprite { .. } | RenderItem::BlurSprite { .. } => {
                            drop(render_pass);
                            encoder.copy_texture_to_texture(
                                frame.texture.as_image_copy(),
                                self.grab_texture.as_image_copy(),
                                wgpu::Extent3d {
                                    width: self.grab_texture.width(),
                                    height: self.grab_texture.height(),
                                    depth_or_array_layers: 1,
                                },
                            );
                            render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &frame_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: &mask_view,
                                        depth_ops: None,
                                        stencil_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        }),
                                    },
                                ),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });
                            render_pass.set_stencil_reference(0);

                            match render_item {
                                RenderItem::BlendModeSprite { .. } => {
                                    render_pass.set_pipeline(&self.blend_mode_pipeline);
                                }
                                RenderItem::BlurSprite { .. } => {
                                    render_pass.set_pipeline(&self.blur_pipeline);
                                }
                                _ => {}
                            }
                            render_pass.set_bind_group(2, &self.grab_texture_bind_group, &[]);
                        }
                        RenderItem::SpriteMaskStart { .. } => {
                            render_pass.set_pipeline(&self.mask_start_pipeline);
                        }
                        RenderItem::SpriteMaskEnd { .. } => {
                            render_pass.set_pipeline(&self.mask_end_pipeline);
                        }
                    }
                    render_pass.set_bind_group(0, &view_uniform_bind_group, &[]);
                    render_pass.set_bind_group(1, texture, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..6_u32, 0, render_item.range().clone());
                }
            }
        }

        #[cfg(feature = "editor_mode")]
        egui_render.render(&self.device, &self.queue, &mut encoder, &frame_view);

        #[cfg(feature = "profiling")]
        profiling::scope!("Submit Commands");
        self.queue.submit(std::iter::once(encoder.finish()));

        frame.present();
    }
}
