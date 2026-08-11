use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};
use crate::renderer::TriangleRenderer;
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
        
        let mut renderer = TriangleRenderer::new(window.clone());
        let mut game_instance = G::init(&renderer);
        
        // 🟢 Track individual frame steps accurately
        let mut last_frame_time = std::time::Instant::now();

        event_loop.run(move |event, target| {
            use winit::event::Event::WindowEvent;
            use winit::event::WindowEvent as WEvent;
            
            match event {
                WindowEvent { window_id, event } if window_id == window.id() => match event {
                    WEvent::CloseRequested => target.exit(),
                    WEvent::Resized(new_size) => {
                        if new_size.width > 0 && new_size.height > 0 {
                            renderer.resize(new_size.width, new_size.height);
                        }
                        window.request_redraw();
                    }
                    WEvent::RedrawRequested => {
                        // 🟢 Calculate true slice time delta
                        let now = std::time::Instant::now();
                        let dt = now.duration_since(last_frame_time).as_secs_f32();
                        last_frame_time = now;

                        // 🟢 Run game math loop with precise time step slice boundaries
                        game_instance.update(&renderer, dt);
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
