struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
        out.color = vec4<f32>(model.color,1.0);
        out.tex_coords = model.tex_coords;
        out.clip_position = camera.view_proj * vec4<f32>(model.position,0.0,1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 将最终的颜色结果从线性空间转换回 sRGB 空间输出到屏幕
    return pow(in.color * textureSample(t_diffuse, s_diffuse, in.tex_coords), vec4(1.0 / 2.2));
}

