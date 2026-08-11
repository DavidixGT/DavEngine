// 1. Declare the incoming layout from the CPU (bound to group 0, binding 0)
struct TimeUniform {
    value: f32,
};
@group(0) @binding(0) var<uniform> time: TimeUniform;

struct VertexInput { @location(0) position: vec2<f32> };


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // 2. Use the time variable to dynamically shift the triangle positions over time (making it bob up and down)
    let offset_y = sin(time.value) * 1;
    let animated_position = vec2<f32>(model.position.x, model.position.y);

    out.clip_position = vec4<f32>(animated_position, 0.0, 1.0);
    //model.color.x = model.color.x + offset_y;
    out.color = vec4<f32>(1, 1, 0 + offset_y, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
