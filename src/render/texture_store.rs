use crate::assets::AssetsId;
use crate::{Render, TEXTURE_FORMAT};
use glam::Vec2;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TextureStore {
    textures: HashMap<AssetsId, (Vec2, wgpu::BindGroup)>,
    auto_increment_key: u32,
}

impl TextureStore {
    pub fn get(&self, id: &AssetsId) -> Option<&(Vec2, wgpu::BindGroup)> {
        self.textures.get(id)
    }
    pub fn load_texture_raw(
        &mut self,
        render: &Render,
        (assets_id, assets_bytes): (AssetsId, &'static [u8]),
    ) -> AssetsId {
        let image = image::load_from_memory(assets_bytes).unwrap();
        let image = image.to_rgba8();
        self.load_texture_with_key(render, &image, Some(assets_id))
    }
    pub fn load_texture(&mut self, render: &Render, image: &image::RgbaImage) -> AssetsId {
        self.load_texture_with_key(render, image, None)
    }
    fn load_texture_with_key(
        &mut self,
        render: &Render,
        image: &image::RgbaImage,
        key: Option<AssetsId>,
    ) -> AssetsId {
        let image_size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = render.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Example Texture"),
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        render.queue.write_texture(
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
        let sampler = render
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let texture_bind_group = render.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render.texture_bind_group_layout,
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

        let key = if let Some(key) = key {
            key
        } else {
            self.auto_increment_key += 1;
            AssetsId::from_u32(self.auto_increment_key)
        };
        self.textures.insert(
            key,
            (
                Vec2::new(image.width() as f32, image.height() as f32),
                texture_bind_group,
            ),
        );
        key
    }
}
