struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Group 0: per-draw text color, updated whenever set_color is called
@group(0) @binding(0) var<uniform> text_color: vec4<f32>;

// Group 1: the 8x1024 glyph atlas + nearest-neighbor sampler
@group(1) @binding(0) var font_atlas: texture_2d<f32>;
@group(1) @binding(1) var font_sampler: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // R8Unorm atlas: the red channel carries the glyph's coverage
    let alpha = textureSample(font_atlas, font_sampler, in.uv).r;
    return vec4<f32>(text_color.rgb, text_color.a * alpha);
}