use crate::renderer::TriangleRenderer;
use crate::material::Shader;
use crate::triangle::Triangle;
use crate::mesh::Mesh;
use crate::text_renderer::TextRenderer;
use std::time::Instant;

pub trait Game {
    fn init(renderer: &TriangleRenderer) -> Self;
    fn update(&mut self, renderer: &TriangleRenderer, dt: f32);
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyCustomVariables {
    pub current_time: f32,
    pub player_speed: f32,
    pub global_scale: f32,
    pub padding: f32,
}

pub struct MyGame {
    custom_shader: Shader,
    start_time: Instant,
    player_ship_mesh: Mesh,
    enemy_ship_mesh: Mesh,
    text_renderer: TextRenderer,
}

impl Game for MyGame {
    fn init(renderer: &TriangleRenderer) -> Self {
        let shader_code = include_str!("shader.wgsl");
        let uniform_size = std::mem::size_of::<MyCustomVariables>() as u64;
        let custom_shader = Shader::new(renderer, shader_code, uniform_size);
        let start_time = Instant::now();

        // 1. Describe vectors using the triangle layout
        let player_tri = Triangle::new([[0.0, 0.5], [-0.3, 0.0], [0.3, 0.0]]);
        let enemy_tri = Triangle::new([[0.0, -0.5], [-0.2, -0.2], [0.2, -0.2]]);

        // 2. Commit them to high-performance GPU Mesh resources
        let player_ship_mesh = Mesh::from_triangle(renderer, &player_tri);
        let enemy_ship_mesh = Mesh::from_triangle(renderer, &enemy_tri);

        // 🔤 Text renderer: bitmap font atlas + per-draw color/scale
        let text_renderer = TextRenderer::new(renderer);

        Self { custom_shader, start_time, player_ship_mesh, enemy_ship_mesh, text_renderer }
    }

    fn update(&mut self, renderer: &TriangleRenderer, _dt: f32) {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        let current_frame_data = MyCustomVariables {
            current_time: elapsed,
            player_speed: 4.5,
            global_scale: 0.8,
            padding: 0.0,
        };

        renderer.update_shader_buffer(&self.custom_shader, &current_frame_data);

        renderer.render_scene(&self.custom_shader, |ctx| {
            self.player_ship_mesh.draw(ctx);
            self.enemy_ship_mesh.draw(ctx);

            // 🔤 Overlay some HUD text in the same pass
            self.text_renderer.set_color(1.0, 1.0, 0.0, 1.0);
            self.text_renderer.set_scale(1.5);
            self.text_renderer.draw_string(ctx, "WGPU ENGINE", -0.9, 0.85);
            self.text_renderer.set_color(1.0, 1.0, 1.0, 1.0);
            self.text_renderer.set_scale(1.0);
            self.text_renderer.draw_string(
                ctx,
                &format!("FPS demo | t = {:.2}s", elapsed),
                -0.9,
                0.75,
            );
        });
    }
}
