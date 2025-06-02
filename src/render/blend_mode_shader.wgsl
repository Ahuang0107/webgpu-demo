fn affine3_to_square(affine: mat3x4<f32>) -> mat4x4<f32> {
    return transpose(mat4x4<f32>(
        affine[0],
        affine[1],
        affine[2],
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    ));
}

struct View {
    clip_from_world: mat4x4<f32>,
    viewport: vec4<f32>,
};

@group(0) @binding(0) var<uniform> view: View;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) i_model_transpose_col0: vec4<f32>,
    @location(1) i_model_transpose_col1: vec4<f32>,
    @location(2) i_model_transpose_col2: vec4<f32>,
    @location(3) i_uv_offset_scale: vec4<f32>,
    @location(4) color: vec4<f32>,
    @location(5) color_blend_mode: u32,
    @location(6) blend_mode: u32,
    @location(7) _padding: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
    @location(2) @interpolate(flat) blend_mode: u32,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let vertex_position = vec3<f32>(
        f32(in.index & 0x1u),
        f32((in.index & 0x2u) >> 1u),
        0.0
    );

    out.clip_position = view.clip_from_world * affine3_to_square(mat3x4<f32>(
        in.i_model_transpose_col0,
        in.i_model_transpose_col1,
        in.i_model_transpose_col2,
    )) * vec4<f32>(vertex_position, 1.0);
    out.uv = vec2<f32>(vertex_position.xy) * in.i_uv_offset_scale.zw + in.i_uv_offset_scale.xy;
    out.color = in.color;
    out.blend_mode = in.blend_mode;

    return out;
}

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@group(2) @binding(0) var grab_texture: texture_2d<f32>;
@group(2) @binding(1) var grab_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var src = textureSample(sprite_texture, sprite_sampler, in.uv);
    src.a = src.a * in.color.a;

    let viewport_uv = vec2<f32>(
        in.clip_position.x / view.viewport.z,
        in.clip_position.y / view.viewport.w
    );
    var backdrop = textureSample(grab_texture, grab_sampler, viewport_uv);

    var out: vec4<f32> = src;
    if in.blend_mode == 11u {
        out = rgba_blend_multiply(backdrop, src);
    } else if in.blend_mode == 31u {
        out = rgba_blend_soft_light(backdrop, src);
    } else if in.blend_mode == 32u {
        out = rgba_blend_hard_light(backdrop, src);
    }

    return out;
}

fn blend_multiply(backdrop: f32, src: f32) -> f32 {
    return backdrop * src;
}

fn blend_screen(backdrop: f32, src: f32) -> f32 {
    return backdrop + src - blend_multiply(backdrop, src);
}

fn blend_soft_light(base: f32, src: f32) -> f32 {
    let d = select(sqrt(base), ((16.0 * base - 12.0) * base + 4.0) * base, base <= 0.25);
    return select(
        base + (2.0 * src - 1.0) * (d - base),
        base - (1.0 - 2.0 * src) * base * (1.0 - d),
        src <= 0.5
    );
}

fn blend_hard_light(backdrop: f32, src: f32) -> f32 {
    return select(blend_screen(backdrop, (src * 2.0 - 1.0)), blend_multiply(backdrop, src * 2.0), src < 0.5);
}

fn rgba_blend_normal(backdrop: vec4<f32>, src: vec4<f32>) -> vec4<f32> {
    if backdrop.a == 0.0 {
        return src;
    }
    if src.a == 0.0 {
        return backdrop;
    }

    let ra = src.a + backdrop.a - src.a * backdrop.a;
    let rr = backdrop.r + (src.r - backdrop.r) * src.a / ra;
    let rg = backdrop.g + (src.g - backdrop.g) * src.a / ra;
    let rb = backdrop.b + (src.b - backdrop.b) * src.a / ra;

    return vec4(rr, rg, rb, ra);
}

fn rgba_blend_multiply(backdrop: vec4<f32>, src: vec4<f32>) -> vec4<f32> {
    let r = blend_multiply(backdrop.r, src.r);
    let g = blend_multiply(backdrop.g, src.g);
    let b = blend_multiply(backdrop.b, src.b);
    let new_src = vec4(r, g, b, src.a);
    return rgba_blend_normal(backdrop, new_src);
}

fn rgba_blend_soft_light(backdrop: vec4<f32>, src: vec4<f32>) -> vec4<f32> {
    let r = blend_soft_light(backdrop.r, src.r);
    let g = blend_soft_light(backdrop.g, src.g);
    let b = blend_soft_light(backdrop.b, src.b);
    let new_src = vec4(r, g, b, src.a);
    return rgba_blend_normal(backdrop, new_src);
}

fn rgba_blend_hard_light(backdrop: vec4<f32>, src: vec4<f32>) -> vec4<f32> {
    let r = blend_hard_light(backdrop.r, src.r);
    let g = blend_hard_light(backdrop.g, src.g);
    let b = blend_hard_light(backdrop.b, src.b);
    let new_src = vec4(r, g, b, src.a);
    return rgba_blend_normal(backdrop, new_src);
}
