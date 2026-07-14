//! colored-line batch for world overlays: star lanes, orbit rings, the debug
//! grid, selection outlines, move-order lines, and the box-select rectangle.
//! a sibling to the sprite batch, drawn on top of the tiles in the world pass.

use bytemuck::{Pod, Zeroable};
use egui::Color32;
use wgpu::util::DeviceExt;

use crate::control_state::ControlState;
use crate::palette;
use crate::viewport::Viewport;
use crate::world::types::EntityType;
use crate::world::{EntityId, World};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct LineVertex {
    pos: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Globals {
    screen: [f32; 2],
    _pad: [f32; 2],
}

pub struct LineBatch {
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    capacity_bytes: u64,
    verts: Vec<LineVertex>,
    /// number of leading vertices that belong behind the tiles (lanes, orbits);
    /// the rest are drawn on top.
    background_verts: u32,
}

/// don't draw the debug grid when this many world tiles are visible per axis
/// (it would be a wall of lines and a perf sink).
const GRID_MAX_TILES: f64 = 400.0;
/// segments used to approximate an orbit ring.
const ORBIT_SEGMENTS: usize = 48;
/// minimum zoom for planet and gas-giant orbit rings.
const PLANET_ORBIT_MIN_ZOOM: f64 = 0.4;
/// minimum zoom for moon orbit rings.
const MOON_ORBIT_MIN_ZOOM: f64 = 1.0;

impl LineBatch {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("line globals"),
            contents: bytemuck::bytes_of(&Globals {
                screen: [1.0, 1.0],
                _pad: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("line globals layout"),
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
            label: Some("line globals bind group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("line shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("line pipeline layout"),
            bind_group_layouts: &[Some(&globals_layout)],
            immediate_size: 0,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let initial_capacity = (std::mem::size_of::<LineVertex>() * 2048) as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("line vertices"),
            size: initial_capacity,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            globals_buffer,
            globals_bind_group,
            vertex_buffer,
            capacity_bytes: initial_capacity,
            verts: Vec::new(),
            background_verts: 0,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        viewport: &Viewport,
        controls: &ControlState,
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

        self.verts.clear();
        // behind the tiles: lanes and orbit rings.
        self.push_lanes(world, viewport);
        self.push_orbits(world, viewport);
        self.background_verts = self.verts.len() as u32;
        // on top of the tiles: grid, move orders, selection, drag box.
        if controls.debug_enabled {
            self.push_grid(viewport, screen_w, screen_h);
        }
        self.push_move_orders(world, viewport);
        self.push_selection(world, viewport, controls);
        self.push_select_box(controls);

        if self.verts.is_empty() {
            return;
        }
        let bytes: &[u8] = bytemuck::cast_slice(&self.verts);
        let needed = bytes.len() as u64;
        if needed > self.capacity_bytes {
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("line vertices"),
                size: needed,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.capacity_bytes = needed;
        }
        queue.write_buffer(&self.vertex_buffer, 0, bytes);
    }

    fn line(&mut self, a: (f64, f64), b: (f64, f64), color: [f32; 4]) {
        self.verts.push(LineVertex {
            pos: [a.0 as f32, a.1 as f32],
            color,
        });
        self.verts.push(LineVertex {
            pos: [b.0 as f32, b.1 as f32],
            color,
        });
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: [f32; 4]) {
        self.line((x, y), (x + w, y), color);
        self.line((x + w, y), (x + w, y + h), color);
        self.line((x + w, y + h), (x, y + h), color);
        self.line((x, y + h), (x, y), color);
    }

    fn push_lanes(&mut self, world: &World, viewport: &Viewport) {
        let color = rgba(palette::DGRAY, 0.5);
        for &(a, b) in world.iter_lanes() {
            if let Some((start, end)) = lane_world_endpoints(world, a, b) {
                let p0 = viewport.world_to_screen_px(start.0, start.1);
                let p1 = viewport.world_to_screen_px(end.0, end.1);
                self.line(p0, p1, color);
            }
        }
    }

    fn push_orbits(&mut self, world: &World, viewport: &Viewport) {
        // skip orbit rings when zoomed far out; they collapse into noise.
        if viewport.zoom < PLANET_ORBIT_MIN_ZOOM {
            return;
        }
        let scale = viewport.world_tile_pixel_size_on_screen();
        let color = rgba(palette::DGRAY, 0.35);
        for (entity, info) in world.iter_orbitals() {
            let Some(entity_type) = world.get_entity_type(entity) else {
                continue;
            };
            if !orbit_is_visible(entity_type, viewport.zoom) {
                continue;
            }
            let Some(center) = orbit_anchor_screen_center(world, viewport, info.anchor) else {
                continue;
            };
            let radius = info.radius * scale;
            let mut prev: Option<(f64, f64)> = None;
            for i in 0..=ORBIT_SEGMENTS {
                let theta = (i as f64 / ORBIT_SEGMENTS as f64) * std::f64::consts::TAU;
                let point = (
                    center.0 + radius * theta.cos(),
                    center.1 + radius * theta.sin(),
                );
                if let Some(p) = prev {
                    self.line(p, point, color);
                }
                prev = Some(point);
            }
        }
    }

    fn push_grid(&mut self, viewport: &Viewport, screen_w: u32, screen_h: u32) {
        let scale = viewport.world_tile_pixel_size_on_screen();
        let origin_x = viewport.anchor.x - (screen_w as f64 / 2.0) / scale;
        let origin_y = viewport.anchor.y - (screen_h as f64 / 2.0) / scale;
        let view_w = screen_w as f64 / scale;
        let view_h = screen_h as f64 / scale;
        if view_w > GRID_MAX_TILES || view_h > GRID_MAX_TILES {
            return;
        }
        let color = rgba(palette::DGRAY, 0.2);
        for x in (origin_x.floor() as i32)..=((origin_x + view_w).ceil() as i32) {
            let sx = (x as f64 - origin_x) * scale;
            self.line((sx, 0.0), (sx, screen_h as f64), color);
        }
        for y in (origin_y.floor() as i32)..=((origin_y + view_h).ceil() as i32) {
            let sy = (y as f64 - origin_y) * scale;
            self.line((0.0, sy), (screen_w as f64, sy), color);
        }
    }

    fn push_move_orders(&mut self, world: &World, viewport: &Viewport) {
        let color = rgba(palette::LGREEN, 0.8);
        for (ship, dest) in &world.move_orders {
            if let Some(pos) = world.get_location_f64(*ship) {
                let p0 = viewport.world_to_screen_px(pos.x, pos.y);
                let p1 = viewport.world_to_screen_px(dest.x, dest.y);
                self.line(p0, p1, color);
            }
        }
    }

    fn push_selection(&mut self, world: &World, viewport: &Viewport, controls: &ControlState) {
        let scale = viewport.world_tile_pixel_size_on_screen();
        let color = rgba(palette::YELLOW, 1.0);
        for &id in &controls.selection {
            if let Some(pos) = world.get_location_f64(id) {
                let center = viewport.world_to_screen_px(pos.x, pos.y);
                let size = (world.get_render_size(id) * scale).max(2.0);
                self.rect(
                    center.0 - size / 2.0 - 2.0,
                    center.1 - size / 2.0 - 2.0,
                    size + 4.0,
                    size + 4.0,
                    color,
                );
            }
        }
    }

    fn push_select_box(&mut self, controls: &ControlState) {
        if let (Some(start), Some(end)) = (controls.selection_box_start, controls.last_mouse_pos) {
            let x = start.0.min(end.0);
            let y = start.1.min(end.1);
            let w = (start.0 - end.0).abs();
            let h = (start.1 - end.1).abs();
            self.rect(x, y, w, h, rgba(palette::WHITE, 1.0));
        }
    }

    /// draw the behind-the-tiles lines (lanes, orbits).
    pub fn draw_background(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.draw_range(render_pass, 0..self.background_verts);
    }

    /// draw the on-top lines (grid, move orders, selection, drag box).
    pub fn draw_foreground(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.draw_range(render_pass, self.background_verts..self.verts.len() as u32);
    }

    fn draw_range(&self, render_pass: &mut wgpu::RenderPass<'_>, range: std::ops::Range<u32>) {
        if range.is_empty() {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.globals_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(range, 0..1);
    }
}

fn rgba(color: Color32, alpha: f32) -> [f32; 4] {
    [
        color.r() as f32 / 255.0,
        color.g() as f32 / 255.0,
        color.b() as f32 / 255.0,
        alpha,
    ]
}

fn orbit_is_visible(entity_type: EntityType, zoom: f64) -> bool {
    match entity_type {
        EntityType::Planet | EntityType::GasGiant => zoom >= PLANET_ORBIT_MIN_ZOOM,
        EntityType::Moon => zoom >= MOON_ORBIT_MIN_ZOOM,
        EntityType::Star | EntityType::Ship => false,
    }
}

fn lane_world_endpoints(
    world: &World,
    a: EntityId,
    b: EntityId,
) -> Option<((f64, f64), (f64, f64))> {
    let pa = world.get_location_f64(a)?;
    let pb = world.get_location_f64(b)?;
    let radius_a = world.get_system_radius(a);
    let radius_b = world.get_system_radius(b);
    let dx = pb.x - pa.x;
    let dy = pb.y - pa.y;
    let distance = dx.hypot(dy);

    if distance <= radius_a + radius_b {
        return None;
    }

    let unit_x = dx / distance;
    let unit_y = dy / distance;
    Some((
        (pa.x + unit_x * radius_a, pa.y + unit_y * radius_a),
        (pb.x - unit_x * radius_b, pb.y - unit_y * radius_b),
    ))
}

fn orbit_anchor_screen_center(
    world: &World,
    viewport: &Viewport,
    anchor: EntityId,
) -> Option<(f64, f64)> {
    let anchor = world.get_location_f64(anchor)?;
    Some(viewport.world_to_screen_px(anchor.x, anchor.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::location::Point;

    #[test]
    fn orbit_anchor_screen_center_preserves_fractional_motion() {
        let mut world = World::default();
        let sol = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let earth = world.spawn_planet("earth".to_string(), sol, 16.0, 0.1, 0.0);
        let _moon = world.spawn_moon("moon".to_string(), earth, 4.0, 0.0, 0.0);
        let viewport = Viewport::default();

        let precise = world.get_location_f64(earth).unwrap();
        let rounded = world.get_location(earth).unwrap();
        let center = orbit_anchor_screen_center(&world, &viewport, earth).unwrap();
        let expected = viewport.world_to_screen_px(precise.x, precise.y);
        let rounded_center = viewport.world_to_screen_px(rounded.x as f64, rounded.y as f64);

        assert!((center.0 - expected.0).abs() < f64::EPSILON);
        assert!((center.1 - expected.1).abs() < f64::EPSILON);
        assert!((center.0 - rounded_center.0).abs() > f64::EPSILON);
        assert!((center.1 - rounded_center.1).abs() > f64::EPSILON);
    }

    #[test]
    fn orbit_visibility_uses_entity_specific_zoom_thresholds() {
        assert!(orbit_is_visible(EntityType::Planet, 0.4));
        assert!(orbit_is_visible(EntityType::GasGiant, 0.4));
        assert!(!orbit_is_visible(EntityType::Planet, 0.399));
        assert!(!orbit_is_visible(EntityType::GasGiant, 0.399));

        assert!(orbit_is_visible(EntityType::Moon, 1.0));
        assert!(!orbit_is_visible(EntityType::Moon, 0.999));
        assert!(!orbit_is_visible(EntityType::Star, 10.0));
        assert!(!orbit_is_visible(EntityType::Ship, 10.0));
    }

    #[test]
    fn lane_endpoints_stop_outside_buffered_system_radii() {
        let mut world = World::default();
        let a = world.spawn_star("a".to_string(), Point { x: 0, y: 0 });
        let b = world.spawn_star("b".to_string(), Point { x: 100, y: 0 });
        world.spawn_planet("a-1".to_string(), a, 10.0, 0.0, 0.0);
        let b_planet = world.spawn_planet("b-1".to_string(), b, 20.0, 0.0, 0.0);
        world.spawn_moon("b-1-moon".to_string(), b_planet, 5.0, 0.0, 0.0);

        let (start, end) = lane_world_endpoints(&world, a, b).unwrap();

        assert_eq!(world.get_system_radius(a), 12.0);
        assert_eq!(world.get_system_radius(b), 30.0);
        assert_eq!(start, (12.0, 0.0));
        assert_eq!(end, (70.0, 0.0));
    }

    #[test]
    fn lane_endpoints_omit_touching_or_overlapping_systems() {
        for distance in [24, 23] {
            let mut world = World::default();
            let a = world.spawn_star("a".to_string(), Point { x: 0, y: 0 });
            let b = world.spawn_star("b".to_string(), Point { x: distance, y: 0 });
            world.spawn_planet("a-1".to_string(), a, 10.0, 0.0, 0.0);
            world.spawn_planet("b-1".to_string(), b, 10.0, 0.0, 0.0);

            assert!(lane_world_endpoints(&world, a, b).is_none());
        }
    }
}
