mod renderer;
mod material;
mod triangle; // 🟢 Abstract geometry layout
mod mesh; 
mod game;
mod engine; 

// 🟢 Bind the concrete game state type out of game.rs
use game::MyGame;
use engine::EngineRunner;

fn main() {
    // 🟢 Point the hardware engine directly to your active game module
    EngineRunner::run::<MyGame>();
}
