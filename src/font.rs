use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[derive(Copy, Clone, Debug)]
pub struct CachedGlyph {
    pub uv: [f32; 4],
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
}

pub struct Font {
    rasterizer: Option<fontdue::Font>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pixels_per_em: f32,
    layout: wgpu::BindGroupLayout,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    glyph_cache: HashMap<char, CachedGlyph>,
    is_pixel_font: bool,
    atlas_size: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}

impl Font {
    pub const ATLAS_SIZE: u32 = 1024;

    pub fn from_ttf(device: &wgpu::Device, queue: &wgpu::Queue, ttf: &[u8], pixels_per_em: f32) -> Self {
        let rasterizer = fontdue::Font::from_bytes(ttf, fontdue::FontSettings::default())
            .expect("failed to parse ttf font");
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TTF Atlas Texture"),
            size: wgpu::Extent3d {
                width: Self::ATLAS_SIZE,
                height: Self::ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut font = Self::wrap_common(device, queue, texture, pixels_per_em, false);
        font.rasterizer = Some(rasterizer);
        font
    }

    pub fn default_pixel_font(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut texture_bytes = vec![0u8; 8 * 1024];
        for ch in 0u8..128 {
            let glyph = font8x8::BASIC_UNICODE[ch as usize].1;
            for row in 0..8 {
                for col in 0..8 {
                    let on = (glyph[row] >> col) & 1;
                    texture_bytes[(ch as usize * 8 + row) * 8 + col] = if on == 1 { 255 } else { 0 };
                }
            }
        }
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Pixel Font Atlas Texture"),
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
        let mut font = Self::wrap_common(device, queue, texture, 8.0, true);
        for ch in 0u8..128 {
            let ascii = ch as f32;
            let glyph = CachedGlyph {
                uv: [
                    0.5 / 8.0,
                    (ascii * 8.0 + 0.5) / 1024.0,
                    7.5 / 8.0,
                    (ascii * 8.0 + 7.5) / 1024.0,
                ],
                width: 8,
                height: 8,
                bearing_x: 0.0,
                bearing_y: 8.0,
                advance: 8.0,
            };
            font.glyph_cache.insert(ch as char, glyph);
        }
        font
    }

    fn wrap_common(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: wgpu::Texture,
        pixels_per_em: f32,
        is_pixel_font: bool,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Font Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font Atlas Bind Group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            rasterizer: None,
            device: device.clone(),
            queue: queue.clone(),
            pixels_per_em,
            layout,
            texture,
            texture_view: view,
            sampler,
            bind_group,
            glyph_cache: HashMap::new(),
            is_pixel_font,
            atlas_size: Self::ATLAS_SIZE,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
        }
    }

    pub fn pixels_per_em(&self) -> f32 {
        self.pixels_per_em
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn atlas_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn glyph(&mut self, ch: char) -> CachedGlyph {
        if let Some(g) = self.glyph_cache.get(&ch) {
            return *g;
        }
        if self.is_pixel_font {
            return CachedGlyph {
                uv: [0.0; 4],
                width: 0,
                height: 0,
                bearing_x: 0.0,
                bearing_y: 0.0,
                advance: 8.0,
            };
        }
        let rasterizer = self.rasterizer.as_ref().expect("TTF rasterizer missing");
        let idx = rasterizer.lookup_glyph_index(ch);
        let (metrics, bitmap) = if idx != 0 {
            rasterizer.rasterize(ch, self.pixels_per_em)
        } else {
            return CachedGlyph {
                uv: [0.0; 4],
                width: 0,
                height: 0,
                bearing_x: 0.0,
                bearing_y: 0.0,
                advance: self.pixels_per_em * 0.5,
            };
        };
        let w = metrics.width as u32;
        let h = metrics.height as u32;
        let Some((gx, gy)) = self.pack(w, h) else {
            eprintln!("[Font] atlas full, skipping glyph '{}'", ch);
            return CachedGlyph {
                uv: [0.0; 4],
                width: 0,
                height: 0,
                bearing_x: metrics.xmin as f32,
                bearing_y: metrics.ymin as f32,
                advance: metrics.advance_width,
            };
        };
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: gx, y: gy, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w.max(1)),
                rows_per_image: Some(h.max(1)),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        let atlas_size = self.atlas_size as f32;
        let glyph = CachedGlyph {
            uv: [
                (gx as f32 + 0.5) / atlas_size,
                (gy as f32 + 0.5) / atlas_size,
                (gx as f32 + w as f32 - 0.5) / atlas_size,
                (gy as f32 + h as f32 - 0.5) / atlas_size,
            ],
            width: w,
            height: h,
            bearing_x: metrics.xmin as f32,
            bearing_y: metrics.ymin as f32,
            advance: metrics.advance_width,
        };
        self.glyph_cache.insert(ch, glyph);
        glyph
    }

    fn pack(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        let pad = 1u32;
        let w = w + pad;
        let h = h + pad;
        if w > self.atlas_size || h > self.atlas_size {
            return None;
        }
        if self.cursor_x + w > self.atlas_size {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }
        if self.cursor_y + h > self.atlas_size {
            return None;
        }
        let pos = (self.cursor_x, self.cursor_y);
        self.cursor_x += w;
        self.row_height = self.row_height.max(h);
        Some(pos)
    }
}