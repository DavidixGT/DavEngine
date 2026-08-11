struct MyShaderVariables {
    current_time: f32,
    player_speed: f32,
    global_scale: f32,
    padding: f32,
};
@group(0) @binding(0) var<uniform> config: MyShaderVariables;

struct VertexInput { @location(0) position: vec2<f32> };
struct VertexOutput { @builtin(position) clip_position: vec4<f32> };

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // You can now access any variables inside the global uniform block dynamically!
    let offset_y = sin(config.current_time * config.player_speed) * 0.1;
    let scaled_pos = model.position * config.global_scale;

    out.clip_position = vec4<f32>(scaled_pos.x, scaled_pos.y + offset_y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.5, 0.0, 1.0);
}
