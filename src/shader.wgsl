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
    let viewport_uv = vec2<f32>(
        ((in.clip_position.x / camera.size.x) + 1.0) / 2.0,
        ((in.clip_position.y / camera.size.y) + 1.0) / 2.0
    );
    // TODO 现在的显示结果，已经说明了能够 grab 到屏幕信息了，接下来需要做的就只是找到正确的位置了
    let grab_color = textureSample(t_grab, s_grab, vec2<f32>(0.2, 0.4));
    let srgb_grab_color = pow(grab_color, vec4(1.0 / 2.2));
    let texture_color = in.color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // 将最终的颜色结果从线性空间转换回 sRGB 空间输出到屏幕
    let srgb_texture_color = pow(texture_color, vec4(1.0 / 2.2));
    if camera.size.z == 1.0 {
        return srgb_grab_color;
    } else{
        return srgb_texture_color;
    }
//    return srgb_grab_color * srgb_texture_color;
}

