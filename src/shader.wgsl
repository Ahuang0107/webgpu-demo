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
        out.color = vec4<f32>(model.color, 1.0);
        out.tex_coords = model.tex_coords;
        out.clip_position = vec4<f32>(model.position.xy, 0.0, 1.0);
        out.blend_mode = model.blend_mode;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 表示片元基于屏幕的 uv 位置，in.clip_position 是基于屏幕的像素坐标，与 png 坐标类似
    let viewport_uv = vec2<f32>(
        in.clip_position.x / camera.size.x,
        in.clip_position.y / camera.size.y
    );
    var grab_color = textureSample(t_grab, s_grab, viewport_uv);
    // NOTE 显示设备通常使用 sRGB 空间（gamma space），导入的 texture 也设定的是 rRGB，进行混合处理时需要再线性空间（linear space）下操作
    grab_color = pow(grab_color, vec4(1.0 / 2.2));
    var texture_color = in.color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
    texture_color = pow(texture_color, vec4(1.0 / 2.2));
    var out: vec4<f32> = texture_color;
    if in.blend_mode == 11u {
        out = blend_mode(out, multiply(texture_color.rgb, grab_color.rgb));
    }
    if in.blend_mode == 30u {
        out = blend_mode(out, overlay(texture_color.rgb, grab_color.rgb));
    }
    if in.blend_mode == 31u {
        out = blend_mode(out, soft_light(texture_color.rgb, grab_color.rgb));
    }
    if in.blend_mode == 32u {
        out = blend_mode(out, hard_light(texture_color.rgb, grab_color.rgb));
    }
    out = pow(out, vec4(2.2));
    return out;
}

fn blend_mode(src: vec4<f32>, dst: vec3<f32>) -> vec4<f32> {
    var dst_2 = dst;
    dst_2.r *= src.a; dst_2.g *= src.a; dst_2.b *= src.a;
    return vec4(dst_2, src.a);
}

fn multiply(base: vec3<f32>, top: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(base.r * top.r, base.g * top.g, base.b * top.b);
}

fn overlay(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return select(1.0 - 2.0 * (1.0 - src) * (1.0 - dst), 2.0 * src * dst, src > vec3<f32>(0.5, 0.5, 0.5));
}

fn soft_light(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return (1.0 - src) * src * dst + src * (1.0 - (1.0 - src) * (1.0 - dst));
}

fn hard_light(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    return select(1.0 - 2.0 * (1.0 - src) * (1.0 - dst), 2.0 * src * dst, dst > vec3(0.5));
}

//fn soft_light_ps(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
//    return select(
//        2.0 * src * dst + pow(src, vec3(2.0)) * (1.0 - 2.0 * dst), 
//        2.0 * src * (1.0 - dst) + sqrt(src) * (2.0 * dst - 1.0), 
//        dst < vec3(0.5)
//    );
//}
//
//fn soft_light_pegtop(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
//    return (1.0 - 2.0 * dst) * pow(src, vec3(2.0)) + 2.0 * src * dst;
//}
//
//fn soft_light_illusions_hu(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
//    let index = pow(vec3(2.0), 2.0 * (vec3(0.5) - dst));
//    return pow(src, vec3(index));
//}
//
//fn soft_light_w3c(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
//    return select(
//        src - (1.0 - 2.0 * dst) * src * (1.0 - src), 
//        src + (2.0 * dst - 1.0) * (
//            select(
//                ((16.0 * src - 12.0) * src + 4.0) * src, 
//                sqrt(src), 
//                src <= vec3(0.25)
//            ) - src
//        ), 
//        dst <= vec3(0.5)
//    );
//}