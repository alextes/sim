//! instanced textured-quad batch that draws the tile world.
//!
//! each visible entity (and background star) becomes one instance: a screen
//! rect + atlas uv rect + rgba tint. one draw call renders them all. the cpu
//! side reuses the `Viewport` transform and `Tileset` uv mapping ported from
//! the old sdl renderer.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::background::BackgroundLayer;
use crate::gfx::atlas::Atlas;
use crate::palette;
use crate::tileset::Tileset;
use crate::viewport::{Viewport, TILE_PIXEL_WIDTH};
use crate::world::types::EntityType;
use crate::world::World;

/// only label stars once tiles are at least this zoomed in.
const STAR_LABEL_MIN_ZOOM: f64 = 0.7;
/// star label glyph size, in world tiles.
const STAR_LABEL_WORLD_SIZE: f64 = 1.2;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Instance {
    dest: [f32; 2],
    size: [f32; 2],
    uv_min: [f32; 2],
    uv_size: [f32; 2],
    tint: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Globals {
    screen: [f32; 2],
    _pad: [f32; 2],
}

pub struct SpriteBatch {
    pipeline: wgpu::RenderPipeline,
    atlas: Atlas,
    tileset: Tileset,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instance_capacity_bytes: u64,
    instances: Vec<Instance>,
}

impl SpriteBatch {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let atlas = Atlas::new(device, queue);

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite globals"),
            contents: bytemuck::bytes_of(&Globals {
                screen: [1.0, 1.0],
                _pad: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite globals layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite globals bind group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite pipeline layout"),
            bind_group_layouts: &[Some(&globals_layout), Some(&atlas.bind_group_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // dest
                1 => Float32x2, // size
                2 => Float32x2, // uv_min
                3 => Float32x2, // uv_size
                4 => Float32x4, // tint
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // start with a small instance buffer; it grows on demand in `prepare`.
        let initial_capacity = (std::mem::size_of::<Instance>() * 1024) as u64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite instances"),
            size: initial_capacity,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            atlas,
            tileset: Tileset::new(),
            globals_buffer,
            globals_bind_group,
            instance_buffer,
            instance_capacity_bytes: initial_capacity,
            instances: Vec::new(),
        }
    }

    /// rebuild the instance set for this frame and upload it.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        viewport: &Viewport,
        background: &BackgroundLayer,
        screen: (u32, u32),
    ) {
        let (screen_w, screen_h) = screen;
        queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::bytes_of(&Globals {
                screen: [screen_w as f32, screen_h as f32],
                _pad: [0.0, 0.0],
            }),
        );

        self.instances.clear();
        self.push_background(background, viewport, screen_w, screen_h);
        self.push_entities(world, viewport, screen_w, screen_h);

        if self.instances.is_empty() {
            return;
        }
        let bytes: &[u8] = bytemuck::cast_slice(&self.instances);
        let needed = bytes.len() as u64;
        if needed > self.instance_capacity_bytes {
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sprite instances"),
                size: needed,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_capacity_bytes = needed;
        }
        queue.write_buffer(&self.instance_buffer, 0, bytes);
    }

    /// parallax starfield: a fixed-size (non-zooming) layer that scrolls slower
    /// than the world, drawn first so it sits behind everything.
    fn push_background(
        &mut self,
        background: &BackgroundLayer,
        viewport: &Viewport,
        screen_w: u32,
        screen_h: u32,
    ) {
        const BASE_PARALLAX: f64 = 0.05;
        const ZOOM_INFLUENCE: f64 = 0.05;
        let factor = BASE_PARALLAX + ZOOM_INFLUENCE * viewport.zoom;
        let anchor_x = viewport.anchor.x * factor;
        let anchor_y = viewport.anchor.y * factor;
        let tile = TILE_PIXEL_WIDTH as f64;
        let origin_x = anchor_x - (screen_w as f64 / 2.0) / tile;
        let origin_y = anchor_y - (screen_h as f64 / 2.0) / tile;
        let view_w = screen_w as f64 / tile;
        let view_h = screen_h as f64 / tile;

        let tint_base = [
            palette::LGRAY.r() as f32 / 255.0,
            palette::LGRAY.g() as f32 / 255.0,
            palette::LGRAY.b() as f32 / 255.0,
        ];

        for star in &background.stars {
            if !in_view(star.pos.x, star.pos.y, origin_x, origin_y, view_w, view_h) {
                continue;
            }
            let (uv_min, uv_size) = self.tileset.uv(star.glyph);
            let sx = (star.pos.x - origin_x) * tile;
            let sy = (star.pos.y - origin_y) * tile;
            self.instances.push(Instance {
                dest: [sx as f32, sy as f32],
                size: [tile as f32, tile as f32],
                uv_min,
                uv_size,
                tint: [
                    tint_base[0],
                    tint_base[1],
                    tint_base[2],
                    star.alpha as f32 / 255.0,
                ],
            });
        }
    }

    /// world entities, each as a glyph tile centered on its position, tinted by
    /// its color and scaled by its render size.
    fn push_entities(&mut self, world: &World, viewport: &Viewport, screen_w: u32, screen_h: u32) {
        let scale = viewport.world_tile_pixel_size_on_screen();
        let origin_x = viewport.anchor.x - (screen_w as f64 / 2.0) / scale;
        let origin_y = viewport.anchor.y - (screen_h as f64 / 2.0) / scale;
        let view_w = screen_w as f64 / scale;
        let view_h = screen_h as f64 / scale;

        for entity in world.iter_entities() {
            let Some(pos) = world.get_location_f64(entity) else {
                continue;
            };
            if !in_view(pos.x, pos.y, origin_x, origin_y, view_w, view_h) {
                continue;
            }

            let glyph = world.get_render_glyph(entity);
            let (uv_min, uv_size) = self.tileset.uv(glyph);
            let size_px = (world.get_render_size(entity) * scale).max(2.0);
            let center_x = (pos.x - origin_x) * scale;
            let center_y = (pos.y - origin_y) * scale;

            let tint = match world.get_entity_color(entity) {
                Some(c) => [
                    c.r as f32 / 255.0,
                    c.g as f32 / 255.0,
                    c.b as f32 / 255.0,
                    1.0,
                ],
                None => [1.0, 1.0, 1.0, 1.0],
            };

            self.instances.push(Instance {
                dest: [
                    (center_x - size_px / 2.0) as f32,
                    (center_y - size_px / 2.0) as f32,
                ],
                size: [size_px as f32, size_px as f32],
                uv_min,
                uv_size,
                tint,
            });

            // label stars with their name below the glyph when zoomed in.
            if viewport.zoom > STAR_LABEL_MIN_ZOOM
                && world.get_entity_type(entity) == Some(EntityType::Star)
            {
                if let Some(name) = world.get_entity_name(entity) {
                    let text = name.to_lowercase();
                    let glyph_px = (STAR_LABEL_WORLD_SIZE * scale) as f32;
                    let text_width = text.chars().count() as f32 * glyph_px;
                    let label_x = center_x as f32 - text_width / 2.0;
                    let label_y = center_y as f32 + size_px as f32 / 2.0 + 2.0;
                    let tint = [
                        palette::LGRAY.r() as f32 / 255.0,
                        palette::LGRAY.g() as f32 / 255.0,
                        palette::LGRAY.b() as f32 / 255.0,
                        0.86,
                    ];
                    self.push_text(&text, label_x, label_y, glyph_px, tint);
                }
            }
        }
    }

    /// emit one glyph instance per character, left to right from (x, y).
    fn push_text(&mut self, text: &str, x: f32, y: f32, glyph_px: f32, tint: [f32; 4]) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if ch != ' ' {
                let (uv_min, uv_size) = self.tileset.uv(ch);
                self.instances.push(Instance {
                    dest: [cursor_x, y],
                    size: [glyph_px, glyph_px],
                    uv_min,
                    uv_size,
                    tint,
                });
            }
            cursor_x += glyph_px;
        }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        let count = self.instances.len() as u32;
        if count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
        render_pass.set_bind_group(1, &self.atlas.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..count);
    }
}

/// true if a 1x1 world tile at (x, y) overlaps the visible world bbox.
fn in_view(x: f64, y: f64, origin_x: f64, origin_y: f64, view_w: f64, view_h: f64) -> bool {
    x + 1.0 > origin_x && x < origin_x + view_w && y + 1.0 > origin_y && y < origin_y + view_h
}
