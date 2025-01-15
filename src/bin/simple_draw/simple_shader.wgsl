struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
        out.clip_position = vec4(model.position.xy, 0.0, 1.0);
        out.uv = model.uv;
    return out;
}

@group(0) @binding(0) var texture_t: texture_2d<f32>;
@group(0) @binding(1) var texture_s: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(texture_t, texture_s, in.uv);
    return texture;
}
