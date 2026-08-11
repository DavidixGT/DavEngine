use crate::renderer::RenderContext;
use wgpu::util::DeviceExt;

pub struct Mesh {
    pub positions: [[f32; 2]; 3],
}

impl Mesh {
    pub fn new(positions: [[f32; 2]; 3]) -> Self {
        Self { positions }
    }

    // 🎯 DRAW IN SCENE: Binds this specific mesh data inside the running frame pass
    pub fn draw<'a>(&self, ctx: &mut RenderContext<'a>) {
        // Build the geometry buffer data block on the hardware
        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vector Buffer"),
            contents: bytemuck::cast_slice(&self.positions),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Attach vectors to the shared frame stream and issue the draw call
        ctx.render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        ctx.render_pass.draw(0..3, 0..1);
    }
}
