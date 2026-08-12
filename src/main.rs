mod renderer;
mod material;
mod triangle; // 🟢 Abstract geometry layout
mod mesh;
mod game;
mod engine;
mod font;
mod text_renderer;
mod test_game;

// 🟢 Bind the concrete game state type out of test_game.rs
use test_game::TestGame;
use engine::EngineRunner;

fn main() {
    // 🟢 Point the hardware engine directly to your active game module
    EngineRunner::run::<TestGame>();
}