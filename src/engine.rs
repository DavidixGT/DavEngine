use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};
use crate::game::Game;

pub struct EngineRunner;

impl EngineRunner {
    pub fn run<G: Game + 'static>() {
        let event_loop = EventLoop::new().unwrap();

        let window = Arc::new(
            WindowBuilder::new()
                .with_title("DavEngine - Modular Split Architecture")
                .build(&event_loop)
                .unwrap()
        );

        // The game builds its own renderer inside init.
        let mut game_instance = G::init(window.clone());

        // 🏁 Spawn-time hook: games can add text/objects here.
        Game::start(&mut game_instance);

        // Track individual frame steps accurately
        let mut last_frame_time = std::time::Instant::now();

        event_loop.run(move |event, target| {
            use winit::event::Event::WindowEvent;
            use winit::event::WindowEvent as WEvent;

            match event {
                WindowEvent { window_id, event } if window_id == window.id() => match event {
                    WEvent::CloseRequested => target.exit(),
                    WEvent::Resized(new_size) => {
                        Game::resize(&mut game_instance, new_size.width, new_size.height);
                        window.request_redraw();
                    }
                    WEvent::RedrawRequested => {
                        // Time step since the last frame
                        let now = std::time::Instant::now();
                        let dt = {
                            let since = now.duration_since(last_frame_time).as_secs_f32();
                            last_frame_time = now;
                            since
                        };
                        Game::update(&mut game_instance, dt);
                        // 🎨 Render is entirely hidden in the base — the game
                        // only provides logic in `update`, then the engine
                        // drives the frame through the trait.
                        Game::render(&mut game_instance);
                    }
                    _ => {}
                },
                winit::event::Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        }).unwrap();
    }
}