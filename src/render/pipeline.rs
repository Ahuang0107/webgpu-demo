pub fn create_pipeline(
    label: &str,
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader: &wgpu::ShaderModule,
    vertex_buffer_layout: wgpu::VertexBufferLayout,
    target: wgpu::ColorTargetState,
    depth_stencil: Option<wgpu::DepthStencilState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            }),
        ),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[vertex_buffer_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(target)],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
