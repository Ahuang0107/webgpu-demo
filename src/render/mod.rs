mod camera;
mod pipeline;
mod rect;
mod render_item;
mod sprite;
mod sprite_instance;
mod transform;

pub use camera::*;
pub use pipeline::*;
pub use rect::*;
pub use render_item::*;
pub use sprite::*;
pub use sprite_instance::*;
pub use transform::*;

use glam::Vec2;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub const MASK_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Stencil8;

pub struct Render {
    pub surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    view_uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    mask_texture: wgpu::Texture,
    grab_texture: wgpu::Texture,
    grab_texture_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    mask_start_pipeline: wgpu::RenderPipeline,
    mask_end_pipeline: wgpu::RenderPipeline,
    textures: HashMap<u32, (Vec2, wgpu::BindGroup)>,
}

impl Render {
    pub async fn new(
        window: std::sync::Arc<winit::window::Window>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("initializing the surface...");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
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
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                #[cfg(not(target_arch = "wasm32"))]
                required_limits: wgpu::Limits::default(),
                #[cfg(target_arch = "wasm32")]
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                ..Default::default()
            })
            .await?;
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        if config.width > 0 && config.height > 0 {
            surface.configure(&device, &config);
        }

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
            &[&view_uniform_bind_group_layout, &texture_bind_group_layout],
            &shader,
            SpriteInstance::desc(),
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
            &[&view_uniform_bind_group_layout, &texture_bind_group_layout],
            &mask_shader,
            SpriteInstance::desc(),
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
            &[&view_uniform_bind_group_layout, &texture_bind_group_layout],
            &mask_shader,
            SpriteInstance::desc(),
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

        Ok(Render {
            surface,
            device,
            queue,
            config,
            view_uniform_bind_group_layout,
            texture_bind_group_layout,
            mask_texture,
            grab_texture,
            grab_texture_bind_group,
            render_pipeline,
            mask_start_pipeline,
            mask_end_pipeline,
            textures: HashMap::new(),
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
    pub fn load_texture(&mut self, image_bytes: &[u8]) -> u32 {
        let image = image::load_from_memory(image_bytes).unwrap();
        let image = image.to_rgba8();
        let image_size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Example Texture"),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: Some(image.height()),
            },
            image_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
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

        let key = self.textures.len() as u32;
        self.textures.insert(
            key,
            (
                Vec2::new(image.width() as f32, image.height() as f32),
                texture_bind_group,
            ),
        );
        key
    }
    pub fn render(&mut self, camera: &Camera2D, sprites: &[Sprite]) {
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
            layout: &self.view_uniform_bind_group_layout,
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
            if let Some((image_size, _)) = self.textures.get(&sprite.texture_id) {
                let sprite_instance = SpriteInstance::from(
                    &sprite.calculate_transform(*image_size),
                    &sprite.calculate_uv_offset_scale(*image_size),
                );
                sprite_instances.push(sprite_instance);
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
            #[cfg(feature = "profiling")]
            profiling::scope!("Begin Render Pass");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 220.0 / 255.0,
                            g: 215.0 / 255.0,
                            b: 203.0 / 255.0,
                            a: 1.0,
                        }),
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
                if let Some((_, texture)) = self.textures.get(&render_item.texture_id()) {
                    match render_item {
                        RenderItem::Sprite { .. } => {
                            render_pass.set_pipeline(&self.render_pipeline);
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

        #[cfg(feature = "profiling")]
        profiling::scope!("Submit Commands");
        self.queue.submit(std::iter::once(encoder.finish()));

        frame.present();
    }
}
