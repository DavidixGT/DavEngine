use crate::triangle::Game;
use crate::renderer::TriangleRenderer;
use crate::material::Shader;
use crate::mesh::Mesh;
use std::time::Instant;

// 1. Design your custom variable payload structure
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyCustomVariables {
    pub current_time: f32,
    pub player_speed: f32,
    pub global_scale: f32,
    pub padding: f32, // Structs should be padded to multiples of 16 bytes for GPU safety alignment
}

pub struct MyGame {
    custom_shader: Shader,
    start_time: Instant,
    player_ship: Mesh,
    enemy_ship: Mesh,
}

impl Game for MyGame {
    fn init(renderer: &TriangleRenderer) -> Self {
        let shader_code = include_str!("shader.wgsl");
        
        // Calculate the physical size of our struct in bytes
        let uniform_size = std::mem::size_of::<MyCustomVariables>() as u64;

        // 2. Pass the byte calculation requirement rule down during compilation setup
        let custom_shader = Shader::new(renderer, shader_code, uniform_size);
        let start_time = Instant::now();

        let player_ship = Mesh::new([[0.0, 0.5], [-0.3, 0.0], [0.3, 0.0]]);
        let enemy_ship = Mesh::new([[0.0, -0.5], [-0.2, -0.2], [0.2, -0.2]]);

        Self { custom_shader, start_time, player_ship, enemy_ship }
    }

    fn update(&mut self, renderer: &TriangleRenderer, _dt: f32) {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        // 3. Assemble your structural properties dynamically from your loop context
        let current_frame_data = MyCustomVariables {
            current_time: elapsed,
            player_speed: 4.5,
            global_scale: 0.8,
            padding: 0.0,
        };

        // 🎯 ONE CLEAN COMMAND: Updates all variable allocations inside the shader instantly
        renderer.update_shader_buffer(&self.custom_shader, &current_frame_data);

        renderer.render_scene(&self.custom_shader, |ctx| {
            self.player_ship.draw(ctx);
            self.enemy_ship.draw(ctx);
        });
    }
}
