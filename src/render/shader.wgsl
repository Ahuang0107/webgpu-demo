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
    @location(2) @interpolate(flat) color_blend_mode: u32,
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
    out.color_blend_mode = in.color_blend_mode;

    return out;
}

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var texture = textureSample(sprite_texture, sprite_sampler, in.uv);
    
    if in.color_blend_mode == 11u {
        texture = rgba_blend_multiply(texture, in.color);
    } else {
        texture = in.color * texture;
    }
    
    return texture;
}

fn blend_multiply(backdrop: f32, src: f32) -> f32 {
    return backdrop * src;
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
