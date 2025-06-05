struct ScreenRepeat {
    uv_offset_scale: vec4<f32>,
    color: vec4<f32>,
    viewport: vec4<f32>,
}

@group(0) @binding(0) var<uniform> screen_repeat: ScreenRepeat;

struct VertexInput {
    @builtin(vertex_index) index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    // 全屏四边形顶点数据
    let pos = array(
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, -1.0),
        vec2(1.0, 1.0)
    );
    let uv = array(
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(0.0, 1.0),
        vec2(1.0, 0.0),
        vec2(1.0, 1.0)
    );

    var out: VertexOutput;
    out.clip_position = vec4(pos[in.index], 0.0, 1.0);
    out.uv = uv[in.index];

    return out;
}

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size_u32 = textureDimensions(sprite_texture);
    let texture_size = vec2<f32>(f32(texture_size_u32.x), f32(texture_size_u32.y));
    // 首先是根据镜头缩放，来调整图案重复的间隔
    let texture_size_fixed = texture_size * screen_repeat.uv_offset_scale.zw;

    // 需要基于镜头中心位置缩放，所以将 clip_position 从以左上角为原点，转换成以屏幕中心为原点
    // NOTE in.clip_position.xy 就是基于左上角的当前片元的坐标，跟 png 的 xy 坐标类似
    var clip_position = in.clip_position.xy - screen_repeat.viewport.zw / vec2<f32>(2.0, 2.0);
    let clip_offset = screen_repeat.uv_offset_scale.xy * screen_repeat.uv_offset_scale.zw;
    clip_position = clip_position + clip_offset;

    // 根据 clip_position 和图案重复间隔，计算对应的 UV
    var uv = vec2(
        (clip_position.x % texture_size_fixed.x) / texture_size_fixed.x,
        (clip_position.y % texture_size_fixed.y) / texture_size_fixed.y
    );
    // 因为 clip_position 转成基于屏幕中心为原点了，所以算出来的 uv 可能是负的，需要转成正的
    uv = (uv + vec2(1.0)) % vec2(1.0);
    let texture = textureSample(sprite_texture, sprite_sampler, uv);

    return rgba_blend_multiply(texture, screen_repeat.color);
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