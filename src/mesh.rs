use crate::renderer::RenderContext;
use crate::triangle::Triangle;
use wgpu::util::DeviceExt;

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl Mesh {
    // Ingests an abstract Triangle primitive and allocates it in VRAM
    pub fn from_triangle(renderer: &crate::renderer::TriangleRenderer, triangle: &Triangle) -> Self {
        let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&triangle.positions),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,
            vertex_count: 3,
        }
    }

    // High-utility render loop draw abstraction
    pub fn draw<'a>(&self, ctx: &mut RenderContext<'a>) {
        ctx.render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        ctx.render_pass.draw(0..self.vertex_count, 0..1);
    }
}
