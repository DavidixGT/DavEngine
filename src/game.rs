use crate::triangle::Game;
use crate::renderer::TriangleRenderer;
use crate::material::Material;
use crate::mesh::Mesh;
use std::time::Instant;

pub struct MyGame {
    material: Material,
    start_time: Instant,
    player_ship: Mesh,
    enemy_ship: Mesh,
}

impl Game for MyGame {
    fn init(renderer: &TriangleRenderer) -> Self {
        let shader_code = include_str!("shader.wgsl");
        let material = Material::new(renderer, shader_code);
        let start_time = Instant::now();

        let player_ship = Mesh::new([[0.0, 0.5], [-0.3, 0.0], [0.3, 0.0]]);
        let enemy_ship = Mesh::new([[0.0, -0.5], [-0.2, -0.2], [0.2, -0.2]]);

        Self { material, start_time, player_ship, enemy_ship }
    }

    fn update(&mut self, renderer: &TriangleRenderer, _dt: f32) {
        let current_time = self.start_time.elapsed().as_secs_f32();
        renderer.update_material_time(&self.material, current_time);

        // 🎯 EXECUTE SCENE PASS: Draws all mesh objects cleanly back-to-back
        renderer.render_scene(&self.material, |ctx| {
            self.player_ship.draw(ctx);
            self.enemy_ship.draw(ctx);
        });
    }
}
