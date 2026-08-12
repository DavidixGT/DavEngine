use crate::font::Font;
use crate::game::TextObject;
use crate::renderer::{RenderContext, TriangleRenderer};

/// One textured vertex: clip-space position + atlas UV.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

/// Rasterize the TTF at this many pixels per em. Larger = crisper glyphs,
/// but uses more atlas space.
const TTF_PIXELS_PER_EM: f32 = 64.0;

/// Default em-size of the font in clip-space units (the viewport spans -1..1).
const DEFAULT_SCALE: f32 = 0.1;

pub struct TextRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    color_buffer: wgpu::Buffer,
    color_bind_group: wgpu::BindGroup,
    font: Font,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity_bytes: u64,
    vertex_offset: u64,
    color: [f32; 4],
    scale: f32,
}

impl TextRenderer {
    /// Build a text renderer that rasterizes the given TTF font bytes
    /// (e.g. `include_bytes!("assets/fonts/myfont.ttf")`). The font atlas is
    /// created immediately, so the byte slice may be dropped afterwards.
    pub fn from_bytes(renderer: &TriangleRenderer, ttf: &[u8]) -> Self {
        let device = renderer.device.clone();
        let queue = renderer.queue.clone();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text_shader.wgsl").into()),
        });

        // ---- Bind group 0: per-draw text color ----
        let color_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Color Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // ---- TTF font atlas (owned; binds at group 1) ----
        let font = Font::from_ttf(&device, &queue, ttf, TTF_PIXELS_PER_EM);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&color_layout, font.bind_group_layout()],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // ---- Text color uniform (starts white) ----
        let color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Color Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Color Bind Group"),
            layout: &color_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
        });

        // ---- Dynamic vertex buffer (grows on demand) ----
        // Multiple draw_string calls per frame share this one buffer. Each draw
        // gets its own private region (see vertex_offset) so that a later draw
        // never overwrites the vertices of an earlier draw.
        let vertex_stride = std::mem::size_of::<TextVertex>() as u64;
        let initial_capacity_bytes = 1024 * vertex_stride;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: initial_capacity_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            render_pipeline,
            color_buffer,
            color_bind_group,
            font,
            vertex_buffer,
            vertex_capacity_bytes: initial_capacity_bytes,
            vertex_offset: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            scale: DEFAULT_SCALE,
        }
    }

    /// Load a TTF font from a file and build a text renderer from it.
    /// The path is resolved relative to the current working directory
    /// (the project root when running `cargo run`).
    pub fn from_file(renderer: &TriangleRenderer, path: &str) -> Self {
        let bytes = std::fs::read(path)
            .unwrap_or_else(|e| panic!("[TextRenderer] failed to read font '{path}': {e}"));
        Self::from_bytes(renderer, &bytes)
    }

    /// Call once at the start of each frame, before any `draw_string` calls.
    /// Resets the per-draw vertex cursor so each draw gets its own region of
    /// the shared vertex buffer instead of overwriting earlier draws.
    pub fn begin_frame(&mut self) {
        self.vertex_offset = 0;
    }

    /// Set the text color (RGBA, 0..1) applied to all subsequent draws.
    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color = [r, g, b, a];
        self.queue.write_buffer(&self.color_buffer, 0, bytemuck::bytes_of(&self.color));
    }

    /// Font em-size in clip-space units. 1.0 = the text spans the full
    /// viewport height. Typical HUD text is 0.05..0.15.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.001);
    }

    /// Draw a batch of plain text objects. Takes `&mut self` plus a separate
    /// `&[&TextObject]` slice so callers can borrow their text objects and this
    /// renderer as disjoint fields (avoids whole-struct borrows). The objects
    /// are plain owned structs, so the latest field state is drawn.
    pub fn draw_objects<'a>(&mut self, ctx: &mut RenderContext<'a>, objects: &[&TextObject]) {
        for obj in objects {
            self.set_color(obj.color[0], obj.color[1], obj.color[2], obj.color[3]);
            self.set_scale(obj.scale);
            self.draw_string(ctx, &obj.text, obj.x, obj.y);
        }
    }

    /// Draw a line of text. `(start_x, start_y)` is the pen position of the
    /// first glyph's baseline in clip-space coordinates (y increases upward).
    /// Supports `\n`.
    pub fn draw_string<'a>(&mut self, ctx: &mut RenderContext<'a>, text: &str, start_x: f32, start_y: f32) {
        // Conversion factor from rasterized font pixels to clip-space units.
        let px_to_clip = self.scale / self.font.pixels_per_em();
        let line_h = self.scale * 1.2;

        let mut verts: Vec<TextVertex> = Vec::new();
        let mut pen_x = start_x;
        let mut pen_y = start_y;

        for ch in text.chars() {
            if ch == '\n' {
                pen_x = start_x;
                pen_y -= line_h;
                continue;
            }

            let glyph = self.font.glyph(ch);
            let w = glyph.width as f32;
            let h = glyph.height as f32;

            if w > 0.0 && h > 0.0 {
                // Bitmap top-left corner in clip space. The glyph bitmap spans
                // [bearing_x, bearing_x + w] x [bearing_y + h, bearing_y]
                // relative to the baseline, with fontdue's y-up convention.
                let x_left = pen_x + glyph.bearing_x * px_to_clip;
                let x_right = x_left + w * px_to_clip;
                let y_top = pen_y + (glyph.bearing_y + h) * px_to_clip;
                let y_bottom = pen_y + glyph.bearing_y * px_to_clip;

                let u0 = glyph.uv[0];
                let v0 = glyph.uv[1];
                let u1 = glyph.uv[2];
                let v1 = glyph.uv[3];

                verts.extend_from_slice(&[
                    TextVertex { position: [x_left, y_top], uv: [u0, v0] },
                    TextVertex { position: [x_right, y_top], uv: [u1, v0] },
                    TextVertex { position: [x_left, y_bottom], uv: [u0, v1] },
                    TextVertex { position: [x_right, y_top], uv: [u1, v0] },
                    TextVertex { position: [x_right, y_bottom], uv: [u1, v1] },
                    TextVertex { position: [x_left, y_bottom], uv: [u0, v1] },
                ]);
            }

            pen_x += glyph.advance * px_to_clip;
        }

        if verts.is_empty() {
            return;
        }

        let vertex_stride = std::mem::size_of::<TextVertex>() as u64;
        let bytes = verts.len() as u64 * vertex_stride;

        // Grow the buffer when this draw (plus previous draws this frame) won't
        // fit. Earlier draws keep referencing the old buffer, which wgpu keeps
        // alive until the queue finishes with it.
        if self.vertex_offset + bytes > self.vertex_capacity_bytes {
            self.vertex_capacity_bytes = (self.vertex_offset + bytes).next_power_of_two();
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Vertex Buffer (resized)"),
                size: self.vertex_capacity_bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Write into this draw's private region, then advance the cursor.
        self.queue.write_buffer(&self.vertex_buffer, self.vertex_offset, bytemuck::cast_slice(&verts));
        let region = self.vertex_offset..self.vertex_offset + bytes;
        self.vertex_offset += bytes;

        ctx.render_pass.set_pipeline(&self.render_pipeline);
        ctx.render_pass.set_bind_group(0, &self.color_bind_group, &[]);
        ctx.render_pass.set_bind_group(1, self.font.atlas_bind_group(), &[]);
        ctx.render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(region));
        ctx.render_pass.draw(0..verts.len() as u32, 0..1);
    }
}