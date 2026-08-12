use crate::renderer::{RenderContext, TriangleRenderer};
use wgpu::util::DeviceExt;

/// One textured vertex: clip-space position + atlas UV.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

/// Glyph cell size in clip-space units at scale 1.0.
const CELL_SIZE: f32 = 0.015;

pub struct TextRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    color_buffer: wgpu::Buffer,
    color_bind_group: wgpu::BindGroup,
    font_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: u32,
    color: [f32; 4],
    scale: f32,
}

impl TextRenderer {
    pub fn new(renderer: &TriangleRenderer) -> Self {
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

        // ---- Bind group 1: font atlas texture + sampler ----
        let font_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Font Atlas Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&color_layout, &font_layout],
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

        // ---- Font atlas: 8x1024, one 8x8 glyph cell per ASCII character ----
        // font8x8 stores each glyph as 8 row bytes; bit 0 of a byte = leftmost pixel.
        let mut texture_bytes = vec![0u8; 8 * 1024];
        for ch in 0u8..128 {
            // BASIC_UNICODE is a [FontUnicode; 128] array; element .1 is the [u8; 8] glyph.
            let glyph = font8x8::BASIC_UNICODE[ch as usize].1;
            for row in 0..8 {
                for col in 0..8 {
                    let on = (glyph[row] >> col) & 1;
                    texture_bytes[(ch as usize * 8 + row) * 8 + col] = if on == 1 { 255 } else { 0 };
                }
            }
        }

        let texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("Font Atlas Texture"),
                size: wgpu::Extent3d {
                    width: 8,
                    height: 1024,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &texture_bytes,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Font Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let font_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font Atlas Bind Group"),
            layout: &font_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
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
        let initial_capacity: u32 = 1024;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: initial_capacity as u64 * std::mem::size_of::<TextVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            render_pipeline,
            color_buffer,
            color_bind_group,
            font_bind_group,
            vertex_buffer,
            vertex_capacity: initial_capacity,
            color: [1.0, 1.0, 1.0, 1.0],
            scale: 1.0,
        }
    }

    /// Set the text color (RGBA, 0..1) applied to all subsequent draws.
    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color = [r, g, b, a];
        self.queue.write_buffer(&self.color_buffer, 0, bytemuck::bytes_of(&self.color));
    }

    /// Global glyph scale. 1.0 = default cell size. Larger = bigger text.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.001);
    }

    /// Draw a line of text. `(start_x, start_y)` is the top-left of the first
    /// glyph in clip-space coordinates (y increases upward). Supports `\n`.
    pub fn draw_string<'a>(&mut self, ctx: &mut RenderContext<'a>, text: &str, start_x: f32, start_y: f32) {
        let mut verts: Vec<TextVertex> = Vec::new();
        let cell = CELL_SIZE * self.scale;
        let char_w = cell * 8.0;
        let line_h = cell * 10.0;

        let mut cursor_x = start_x;
        let mut cursor_y = start_y;

        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = start_x;
                cursor_y -= line_h;
                continue;
            }
            let ascii = ch as u32;
            if ascii > 127 {
                continue;
            }

            // Half-texel inset UVs so nearest sampling never bleeds between glyphs.
            let u0 = 0.5 / 8.0;
            let u1 = 7.5 / 8.0;
            let v0 = (ascii as f32 * 8.0 + 0.5) / 1024.0;
            let v1 = (ascii as f32 * 8.0 + 7.5) / 1024.0;

            let x0 = cursor_x;
            let y0 = cursor_y; // top (clip y = up)
            let x1 = cursor_x + char_w;
            let y1 = cursor_y - cell * 8.0; // bottom

            verts.extend_from_slice(&[
                TextVertex { position: [x0, y0], uv: [u0, v0] },
                TextVertex { position: [x1, y0], uv: [u1, v0] },
                TextVertex { position: [x0, y1], uv: [u0, v1] },
                TextVertex { position: [x1, y0], uv: [u1, v0] },
                TextVertex { position: [x1, y1], uv: [u1, v1] },
                TextVertex { position: [x0, y1], uv: [u0, v1] },
            ]);

            cursor_x += char_w;
        }

        if verts.is_empty() {
            return;
        }

        let needed = verts.len() as u32;
        if needed > self.vertex_capacity {
            self.vertex_capacity = needed;
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Vertex Buffer (resized)"),
                size: self.vertex_capacity as u64 * std::mem::size_of::<TextVertex>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&verts));

        ctx.render_pass.set_pipeline(&self.render_pipeline);
        ctx.render_pass.set_bind_group(0, &self.color_bind_group, &[]);
        ctx.render_pass.set_bind_group(1, &self.font_bind_group, &[]);
        ctx.render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        ctx.render_pass.draw(0..needed, 0..1);
    }
}