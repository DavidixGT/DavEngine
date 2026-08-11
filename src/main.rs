mod triangle;
mod renderer;
mod material;
mod mesh; // ✅ Register your new file
mod game;

use triangle::EngineRunner;
use game::MyGame;

fn main() {
    EngineRunner::run::<MyGame>();
}
