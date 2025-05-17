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
    @location(5) blend_mode: u32,
    @location(6) _padding: vec3<u32>,
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
    var texture = textureSample(sprite_texture, sprite_sampler, in.uv);
    if texture.a == 0.0 {
        discard;
    }

    let kernel: array<f32, 9> = array<f32, 9>(0.075, 0.124, 0.075, 0.124, 0.204, 0.124, 0.075, 0.124, 0.075);
    let offset_x: array<f32, 9> = array<f32, 9>(-2.0, 0.0, 2.0, -2.0, 0.0, 2.0, -2.0, 0.0, 2.0);
    let offset_y: array<f32, 9> = array<f32, 9>(-2.0, -2.0, -2.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0);

    var color: vec3<f32> = vec3<f32>(0.0);
    for (var i: i32 = 0; i < 9; i = i + 1) {
        let sample_uv = vec2<f32>(
            (in.clip_position.x + offset_x[i]) / view.viewport.z,
            (in.clip_position.y + offset_y[i]) / view.viewport.w
        );
        color += textureSample(grab_texture, grab_sampler, sample_uv).rgb * kernel[i];
    }
    let out = vec4(color.rgb, 1.0);

    return out;
}
