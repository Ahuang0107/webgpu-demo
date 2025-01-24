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
    return to_linear(texture);
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
