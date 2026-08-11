use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use crate::renderer::TriangleRenderer;

// The High-Level App Interface
pub trait Game {
    fn init(renderer: &TriangleRenderer) -> Self;
    fn update(&mut self, renderer: &TriangleRenderer, dt: f32);
}

pub struct EngineRunner;

impl EngineRunner {
    pub fn run<G: Game + 'static>() {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(WindowBuilder::new().with_title("Clean Split Architecture").build(&event_loop).unwrap());
        let mut renderer = TriangleRenderer::new(window.clone());
        
        let mut game_instance = G::init(&renderer);
        let start_time = std::time::Instant::now();

        event_loop.run(move |event, target| {
            use winit::event::Event::WindowEvent;
            use winit::event::WindowEvent as WEvent;
            match event {
                WindowEvent { window_id, event } if window_id == window.id() => match event {
                    WEvent::CloseRequested => target.exit(),
                    WEvent::Resized(new_size) => {
                        renderer.resize(new_size.width, new_size.height);
                        window.request_redraw();
                    }
                    WEvent::RedrawRequested => {
                        let elapsed = start_time.elapsed().as_secs_f32();
                        game_instance.update(&renderer, elapsed);
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
