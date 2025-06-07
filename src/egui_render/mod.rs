use egui::ViewportId;
use egui_winit::EventResponse;
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct EguiRender {
    pub window: Arc<Window>,
    pub context: egui::Context,
    pub winit_state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
    pub screen_descriptor: egui_wgpu::ScreenDescriptor,
    pub clipped_primitives: Vec<egui::epaint::ClippedPrimitive>,
    pub textures_delta: egui::TexturesDelta,
}

impl EguiRender {
    pub fn new(
        window: Arc<Window>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        stencil_format: wgpu::TextureFormat,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let egui_context = egui::Context::default();

        // // 在初始化 egui 时设置默认字体
        // let mut fonts = egui::FontDefinitions::default();
        // // 添加你的字体
        // fonts.font_data.insert(
        //     "NotoSans-Regular".to_owned(),
        //     Arc::new(egui::FontData::from_static(include_bytes!(
        //         "NotoSans-Regular.ttf"
        //     ))),
        // );
        // // 设置字体家族
        // fonts
        //     .families
        //     .entry(egui::FontFamily::Proportional)
        //     .or_default()
        //     .insert(0, "NotoSans-Regular".to_owned());
        // // 应用字体设置
        // egui_context.set_fonts(fonts);

        let winit_state = egui_winit::State::new(
            egui_context.clone(),
            ViewportId::ROOT,
            &window,
            Some(1.0),
            None,
            None,
        );

        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            None,
            1,
            config.present_mode != wgpu::PresentMode::AutoNoVsync,
        );

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: 1.0,
        };

        Self {
            window,
            context: egui_context,
            winit_state,
            renderer,
            screen_descriptor,
            clipped_primitives: Vec::new(),
            textures_delta: egui::TexturesDelta::default(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.winit_state.on_window_event(&self.window, event)
    }

    pub fn update(&mut self, run_ui: impl FnMut(&egui::Context)) {
        let raw_input = self.winit_state.take_egui_input(&self.window);
        let full_output = self.context.run(raw_input, run_ui);

        self.winit_state
            .handle_platform_output(&self.window, full_output.platform_output);

        if full_output.shapes.is_empty() {
            self.clipped_primitives.clear();
            // log::debug!("No shapes to tessellate");
        } else {
            self.clipped_primitives = self.context.tessellate(full_output.shapes, 1.0);
            // log::debug!(
            //     "Generated {} clipped primitives",
            //     self.clipped_primitives.len()
            // );
        }
        self.textures_delta = full_output.textures_delta;
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        // 上传所有纹理
        for (id, image_delta) in &self.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        // 2. 检查有效数据
        if self.clipped_primitives.is_empty() {
            log::warn!("No primitives to render");
            return;
        }

        // 上传所有网格
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                self.screen_descriptor.size_in_pixels[0],
                self.screen_descriptor.size_in_pixels[1],
            ],
            pixels_per_point: self.screen_descriptor.pixels_per_point,
        };

        let _command_buffer = self.renderer.update_buffers(
            device,
            queue,
            encoder,
            self.clipped_primitives.as_slice(),
            &screen_descriptor,
        );

        let egui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui main render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let mut egui_render_pass = egui_render_pass.forget_lifetime();

        self.renderer.render(
            &mut egui_render_pass,
            self.clipped_primitives.as_slice(),
            &screen_descriptor,
        );

        drop(egui_render_pass);

        // 清理纹理
        for id in &self.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
