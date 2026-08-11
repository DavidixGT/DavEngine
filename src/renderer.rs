use crate::material::Shader;
use std::sync::Arc;
use winit::window::Window;

pub struct TriangleRenderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

// A helper wrapper struct to carry the active render pass state between files safely
pub struct RenderContext<'a> {
    pub render_pass: wgpu::RenderPass<'a>,
    pub device: &'a wgpu::Device,
}

impl TriangleRenderer {
    pub fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();
        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self { surface, device, queue, config }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_shader_buffer<T: bytemuck::Pod>(&self, shader: &Shader, data: &T) {
        self.queue.write_buffer(
            &shader.uniform_buffer, 
            0, 
            bytemuck::bytes_of(data) // Casts the entire layout structure into raw binary bytes automatically
        );
    }

    // 🎯 FLICKER FIX: Run everything inside a custom execution block closure
    pub fn render_scene<F>(&self, material: &Shader, draw_calls: F) 
    where
        F: for<'a> FnOnce(&mut RenderContext<'a>),
    {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            // Open ONE single canvas pass and clear the window background once
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Set up our shared pipeline settings once
            render_pass.set_pipeline(&material.render_pipeline);
            render_pass.set_bind_group(0, &material.bind_group, &[]);

            // Package the render pass state up so our mesh files can access it
            let mut context = RenderContext {
                render_pass,
                device: &self.device,
            };

            // Execute whatever mesh drawing items are passed in from game.rs
            draw_calls(&mut context);
        } // The render pass safely closes out here

        // Present all modifications to your screen at the exact same millisecond
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
