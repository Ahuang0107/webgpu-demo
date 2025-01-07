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
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) blend_mode: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) blend_mode: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
        out.color = model.color;
        out.tex_coords = model.tex_coords;
        out.clip_position = vec4<f32>(model.position.xy, 0.0, 1.0);
        out.blend_mode = model.blend_mode;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//    // 表示片元基于屏幕的 uv 位置，in.clip_position 是基于屏幕的像素坐标，与 png 坐标类似
//    let viewport_uv = vec2<f32>(
//        in.clip_position.x / camera.size.x,
//        in.clip_position.y / camera.size.y
//    );
//    var grab_color = textureSample(t_grab, s_grab, viewport_uv);
//    // NOTE 显示设备通常使用 sRGB 空间（gamma space），导入的 texture 也设定的是 rRGB，进行混合处理时需要再线性空间（linear space）下操作
//    //  https://zhuanlan.zhihu.com/p/66558476
//    grab_color = pow(grab_color, vec4(1.0 / 2.2));

    var color = in.color;
//    var color = from_linear(in.color);
    var texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
//    texture_color = from_linear(texture_color);
    texture_color.a = texture_color.a * color.a;
    var out: vec4<f32> = texture_color;
//    if in.blend_mode == 11u {
//        let result_rgb = multiply(grab_color.rgb, texture_color.rgb);
//        out.r = result_rgb.r * out.a;
//        out.g = result_rgb.g * out.a;
//        out.b = result_rgb.b * out.a;
//    }
//    if in.blend_mode == 30u {
//        let result_rgb = overlay(grab_color.rgb, texture_color.rgb);
//        out.r = result_rgb.r * out.a;
//        out.g = result_rgb.g * out.a;
//        out.b = result_rgb.b * out.a;
//    }
//    if in.blend_mode == 31u {
//        let result_rgb = soft_light(grab_color.rgb, texture_color.rgb);
//        out.r = result_rgb.r * out.a;
//        out.g = result_rgb.g * out.a;
//        out.b = result_rgb.b * out.a;
//    }
//    if in.blend_mode == 32u {
//        let result_rgb = hard_light(grab_color.rgb, texture_color.rgb);
//        out.r = result_rgb.r * out.a;
//        out.g = result_rgb.g * out.a;
//        out.b = result_rgb.b * out.a;
//    }
//    out = to_linear(texture_color);
    return out;
}

fn from_linear(c: vec4<f32>) -> vec4<f32> {
    let higher = vec4<f32>(1.055) * pow(c, vec4(1.0 / 2.4)) - vec4(0.055);
    let lower = c * vec4<f32>(12.92);
    return select(lower, higher, c > vec4<f32>(0.0031308));
}

fn to_linear(c: vec4<f32>) -> vec4<f32> {
    let higher = pow((c + vec4<f32>(0.055)) / vec4<f32>(1.055), vec4<f32>(2.4));
    let lower = c / vec4<f32>(12.92);
    return select(lower, higher, c > vec4<f32>(0.04045));
}

fn multiply(base: vec3<f32>, top: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(base.r * top.r, base.g * top.g, base.b * top.b);
}

fn overlay(base: vec3<f32>, top: vec3<f32>) -> vec3<f32> {
    // NOTE select(f,t,cond)
    return select(2.0 * base * top, 1.0 - 2.0 * (1.0 - base) * (1.0 - top), base > vec3<f32>(0.5, 0.5, 0.5));
}

fn soft_light(base: vec3<f32>, top: vec3<f32>) -> vec3<f32> {
    return (1.0 - base) * base * top + base * (1.0 - (1.0 - base) * (1.0 - top));
}

fn hard_light(base: vec3<f32>, top: vec3<f32>) -> vec3<f32> {
    return select(2.0 * base * top, 1.0 - 2.0 * (1.0 - base) * (1.0 - top), top > vec3(0.5));
}