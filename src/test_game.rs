use crate::game::{BaseGame, Game, TextObject};
use crate::mesh::Mesh;
use crate::triangle::Triangle;
use std::ops::{Deref, DerefMut};

/// Concrete demo game. Embeds a `BaseGame` and implements `Deref`/`DerefMut`
/// for it, so every base method (`add_mesh`, `uniforms`, ...) is callable
/// straight on `self` — no `base_mut` boilerplate needed. HUD text is owned
/// as plain `TextObject` fields and handed to the base each frame by
/// `hud_texts`.
pub struct TestGame {
    base: BaseGame,
    title_text: TextObject,
    fps_text: TextObject,
    elapsed_time: f32,
}

impl Deref for TestGame {
    type Target = BaseGame;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TestGame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Game for TestGame {
    /// Build the game state: GPU meshes + owned HUD text fields.
    fn init(window: std::sync::Arc<winit::window::Window>) -> Self {
        let mut base = BaseGame::new(window);

        // Player + enemy ships committed to GPU mesh resources + registered
        // with the base — the base renders them every frame.
        let player_tri = Triangle::new([[0.0, 0.5], [-0.3, 0.0], [0.3, 0.0]]);
        let enemy_tri = Triangle::new([[0.0, -0.5], [-0.2, -0.2], [0.2, -0.2]]);
        base.add_mesh(Mesh::from_triangle(&base.renderer, &player_tri));
        base.add_mesh(Mesh::from_triangle(&base.renderer, &enemy_tri));

        Self {
            base,
            title_text: TextObject::new(
                "WGPU ENGINE", -0.9, 0.9, 0.12, [1.0, 1.0, 0.0, 1.0],
            ),
            fps_text: TextObject::new(
                "FPS demo", -0.9, 0.78, 0.06, [1.0, 1.0, 1.0, 1.0],
            ),
            elapsed_time: 0.0,
        }
    }

    // Pure logic — the FPS counter is just a plain field, so mutating it
    // is a straightforward `self.fps_text.text = ...` write. No indices,
    // no registry lookups, no borrow tricks.
    fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
        self.uniforms.current_time = self.elapsed_time;

        self.fps_text.text = format!(
            "FPS demo | t = {:.2}s | dt = {:.2}ms",
            self.elapsed_time,
            dt * 1000.0
        );
    }

    // Hand the base a snapshot of our HUD text to draw on top of the meshes.
    fn hud_texts(&self) -> Vec<TextObject> {
        vec![self.title_text.clone(), self.fps_text.clone()]
    }
    // No start / resize / render / base_mut — all hidden in engine/base.
}
