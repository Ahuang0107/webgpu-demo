struct CameraUniform {
    view_proj: mat4x4<f32>,
    size: vec4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;
@group(2) @binding(0) var t_grab: texture_2d<f32>;
@group(2) @binding(1) var s_grab: sampler;

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
        out.color = vec4<f32>(model.color, 1.0);
        out.tex_coords = model.tex_coords;
        out.clip_position = vec4<f32>(model.position.xy, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 表示片元基于屏幕的 uv 位置，in.clip_position 是基于屏幕的像素坐标，与 png 坐标类似
    let viewport_uv = vec2<f32>(
        in.clip_position.x / camera.size.x,
        in.clip_position.y / camera.size.y
    );
    let grab_color = textureSample(t_grab, s_grab, viewport_uv);
    // 从屏幕捕获的 grab texture 已经是 srgb 空间了
//    let srgb_grab_color = pow(grab_color, vec4(1.0 / 2.2));
    let texture_color = in.color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // 将最终的颜色结果从线性空间转换回 sRGB 空间输出到屏幕
    let srgb_texture_color = pow(texture_color, vec4(1.0 / 2.2));
    if camera.size.z == 1.0 {
        return vec4<f32>(hard_light(srgb_texture_color.rgb, grab_color.rgb), srgb_texture_color.a);
    } else{
        return srgb_texture_color;
    }
}

fn hard_light(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(hard_light_channel(a.x, b.x), hard_light_channel(a.y, b.y), hard_light_channel(a.z, b.z));
}

fn hard_light_channel(a: f32, b: f32) -> f32 {
    return select(2. * a * b, 1. - 2. * (1. - a) * (1. -b), b < 0.5);
}