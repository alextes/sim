use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use tracing::error;

use crate::colors;
use crate::event_handling::ControlState;
use crate::initialization::{INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH};
use crate::location::PointF64;
use crate::world::types::EntityType;
use crate::world::{EntityId, World};

use super::{SpriteSheetRenderer, TILE_PIXEL_WIDTH};

const PLANET_ORBIT_MIN_ZOOM: f64 = 0.4;
const MOON_ORBIT_MIN_ZOOM: f64 = 1.0;
const STAR_MAP_ZOOM_THRESHOLD: f64 = PLANET_ORBIT_MIN_ZOOM;

struct ViewportRenderContext {
    view_world_origin_x: f64,
    view_world_origin_y: f64,
    world_tile_actual_pixel_size_on_screen: f64,
    view_bbox_world_x_min: f64,
    view_bbox_world_x_max: f64,
    view_bbox_world_y_min: f64,
    view_bbox_world_y_max: f64,
    tile_on_screen_render_w: u32,
}

struct DrawSpriteContext<'a, 'b, 'c, 'd, 'e, 'f> {
    canvas: &'a mut Canvas<Window>,
    renderer: &'b SpriteSheetRenderer<'f>,
    world: &'c World,
    viewport: &'d Viewport,
    ctx: &'e ViewportRenderContext,
}

fn draw_move_orders(
    canvas: &mut Canvas<Window>,
    world: &World,
    selection: &[EntityId],
    ctx: &ViewportRenderContext,
) {
    for &id in selection {
        let destination = match world.move_orders.get(&id) {
            Some(dest) => dest,
            None => {
                continue;
            }
        };

        let pos_a = match world.get_location_f64(id) {
            Some(pos) => pos,
            None => {
                error!("selected entity {} has move order but no position", id);
                continue;
            }
        };

        // draw line
        let ax = (pos_a.x - ctx.view_world_origin_x) * ctx.world_tile_actual_pixel_size_on_screen;
        let ay = (pos_a.y - ctx.view_world_origin_y) * ctx.world_tile_actual_pixel_size_on_screen;
        let bx =
            (destination.x - ctx.view_world_origin_x) * ctx.world_tile_actual_pixel_size_on_screen;
        let by =
            (destination.y - ctx.view_world_origin_y) * ctx.world_tile_actual_pixel_size_on_screen;

        canvas.set_draw_color(colors::GREEN);
        let p1 = sdl2::rect::Point::new(ax.round() as i32, ay.round() as i32);
        let p2 = sdl2::rect::Point::new(bx.round() as i32, by.round() as i32);
        canvas.draw_line(p1, p2).ok();

        // draw marker 'x'
        let marker_size = (ctx.tile_on_screen_render_w as f64 * 0.5).max(4.0);
        let p1 = sdl2::rect::Point::new(
            (bx - marker_size / 2.0).round() as i32,
            (by - marker_size / 2.0).round() as i32,
        );
        let p2 = sdl2::rect::Point::new(
            (bx + marker_size / 2.0).round() as i32,
            (by + marker_size / 2.0).round() as i32,
        );
        canvas.draw_line(p1, p2).ok();
        let p1 = sdl2::rect::Point::new(
            (bx - marker_size / 2.0).round() as i32,
            (by + marker_size / 2.0).round() as i32,
        );
        let p2 = sdl2::rect::Point::new(
            (bx + marker_size / 2.0).round() as i32,
            (by - marker_size / 2.0).round() as i32,
        );
        canvas.draw_line(p1, p2).ok();
    }
}

fn draw_star_lanes(canvas: &mut Canvas<Window>, world: &World, ctx: &ViewportRenderContext) {
    let old_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(
        colors::LGRAY.r,
        colors::LGRAY.g,
        colors::LGRAY.b,
        80, // more visible alpha
    ));
    for &(a, b) in world.iter_lanes() {
        if let (Some(pos_a_f64), Some(pos_b_f64)) =
            (world.get_location_f64(a), world.get_location_f64(b))
        {
            // center points of stars
            let pos_a = PointF64 {
                x: pos_a_f64.x + 0.5,
                y: pos_a_f64.y + 0.5,
            };
            let pos_b = PointF64 {
                x: pos_b_f64.x + 0.5,
                y: pos_b_f64.y + 0.5,
            };

            let radius_a = world.get_system_radius(a);
            let radius_b = world.get_system_radius(b);

            let dx = pos_b.x - pos_a.x;
            let dy = pos_b.y - pos_a.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < radius_a + radius_b {
                // systems are overlapping, don't draw a lane to avoid weirdness.
                continue;
            }

            let unit_dx = dx / dist;
            let unit_dy = dy / dist;

            let start_x = pos_a.x + unit_dx * radius_a;
            let start_y = pos_a.y + unit_dy * radius_a;

            let end_x = pos_b.x - unit_dx * radius_b;
            let end_y = pos_b.y - unit_dy * radius_b;

            // now convert to screen coords
            let screen_start_x =
                (start_x - ctx.view_world_origin_x) * ctx.world_tile_actual_pixel_size_on_screen;
            let screen_start_y =
                (start_y - ctx.view_world_origin_y) * ctx.world_tile_actual_pixel_size_on_screen;
            let screen_end_x =
                (end_x - ctx.view_world_origin_x) * ctx.world_tile_actual_pixel_size_on_screen;
            let screen_end_y =
                (end_y - ctx.view_world_origin_y) * ctx.world_tile_actual_pixel_size_on_screen;

            let p1 = sdl2::rect::Point::new(
                screen_start_x.round() as i32,
                screen_start_y.round() as i32,
            );
            let p2 =
                sdl2::rect::Point::new(screen_end_x.round() as i32, screen_end_y.round() as i32);
            let _ = canvas.draw_line(p1, p2);
        }
    }
    canvas.set_blend_mode(old_blend_mode);
}

fn draw_orbit_lines(
    canvas: &mut Canvas<Window>,
    world: &World,
    viewport: &Viewport,
    ctx: &ViewportRenderContext,
) {
    let old_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    for (entity_id, orbital_info) in world.iter_orbitals() {
        let entity_type = match world.get_entity_type(entity_id) {
            Some(t) => t,
            None => continue,
        };

        let anchor_pos = match world.get_location_f64(orbital_info.anchor) {
            Some(pos) => pos,
            None => continue,
        };

        let (should_draw, color) = match entity_type {
            EntityType::Planet | EntityType::GasGiant if viewport.zoom >= PLANET_ORBIT_MIN_ZOOM => {
                (
                    true,
                    sdl2::pixels::Color::RGBA(
                        colors::LBLUE.r,
                        colors::LBLUE.g,
                        colors::LBLUE.b,
                        120,
                    ),
                )
            }
            EntityType::Moon if viewport.zoom >= MOON_ORBIT_MIN_ZOOM => (
                true,
                sdl2::pixels::Color::RGBA(colors::LGRAY.r, colors::LGRAY.g, colors::LGRAY.b, 80),
            ),
            _ => (false, sdl2::pixels::Color::BLACK), // don't draw
        };

        if should_draw {
            let center_x = (anchor_pos.x - ctx.view_world_origin_x)
                * ctx.world_tile_actual_pixel_size_on_screen;
            let center_y = (anchor_pos.y - ctx.view_world_origin_y)
                * ctx.world_tile_actual_pixel_size_on_screen;

            let radius_pixels = orbital_info.radius * ctx.world_tile_actual_pixel_size_on_screen;

            canvas.set_draw_color(color);
            draw_circle(
                canvas,
                center_x.round() as i32,
                center_y.round() as i32,
                radius_pixels.round() as i32,
            );
        }
    }

    canvas.set_blend_mode(old_blend_mode);
}

fn is_in_view(pos: PointF64, ctx: &ViewportRenderContext) -> bool {
    let entity_world_x_f64 = pos.x;
    let entity_world_y_f64 = pos.y;

    entity_world_x_f64 + 1.0 > ctx.view_bbox_world_x_min
        && entity_world_x_f64 < ctx.view_bbox_world_x_max
        && entity_world_y_f64 + 1.0 > ctx.view_bbox_world_y_min
        && entity_world_y_f64 < ctx.view_bbox_world_y_max
}

#[allow(clippy::too_many_arguments)]
fn draw_entity_sprite(
    context: &mut DrawSpriteContext,
    entity_id: EntityId,
    entity_type: Option<EntityType>,
    dest_x: i32,
    dest_y: i32,
) -> Rect {
    let glyph = context.world.get_render_glyph(entity_id);
    let src_rect_in_tileset = context.renderer.tileset.get_rect(glyph);

    let entity_render_size_world = context.world.get_render_size(entity_id);
    let on_screen_pixel_size = (entity_render_size_world
        * context.ctx.world_tile_actual_pixel_size_on_screen)
        .round()
        .max(2.0) as u32;

    let mut dest_rect_on_screen =
        Rect::new(dest_x, dest_y, on_screen_pixel_size, on_screen_pixel_size);
    dest_rect_on_screen.center_on(sdl2::rect::Point::new(dest_x, dest_y));

    if let Some(EntityType::Star) = entity_type {
        if context.viewport.zoom < STAR_MAP_ZOOM_THRESHOLD {
            const STAR_MAP_MODE_PIXEL_SIZE: u32 = 8;
            dest_rect_on_screen.set_width(STAR_MAP_MODE_PIXEL_SIZE);
            dest_rect_on_screen.set_height(STAR_MAP_MODE_PIXEL_SIZE);
        }
    }

    if let Some(color) = context.world.get_entity_color(entity_id) {
        context
            .renderer
            .set_texture_color_mod(color.r, color.g, color.b);
    } else {
        // fallback to a default color if none is set.
        context.renderer.set_texture_color_mod(255, 255, 255); // white
    }

    context
        .canvas
        .copy(
            &context.renderer.texture_ref(),
            Some(src_rect_in_tileset),
            Some(dest_rect_on_screen),
        )
        .unwrap_or_else(|e| error!("failed to copy tile for entity {}: {:?}", entity_id, e));

    dest_rect_on_screen
}

fn draw_selection_outline(canvas: &mut Canvas<Window>, entity_render_rect: &Rect) {
    canvas.set_draw_color(colors::YELLOW);
    let outline_rect = Rect::new(
        entity_render_rect.x() - 2,
        entity_render_rect.y() - 2,
        entity_render_rect.width() + 4,
        entity_render_rect.height() + 4,
    );
    canvas.draw_rect(outline_rect).ok();
}

fn draw_star_label(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    viewport: &Viewport,
    entity_id: EntityId,
    pos: PointF64,
    ctx: &ViewportRenderContext,
) {
    let entity_world_x_f64 = pos.x;
    let entity_world_y_f64 = pos.y;

    if viewport.zoom < STAR_MAP_ZOOM_THRESHOLD {
        if let Some(name) = world.get_entity_name(entity_id) {
            let text = name.to_lowercase();

            const TARGET_FONT_PIXEL_SIZE: f64 = 10.0;
            let font_size_world =
                TARGET_FONT_PIXEL_SIZE / ctx.world_tile_actual_pixel_size_on_screen;
            let text_width_world = text.chars().count() as f64 * font_size_world;

            let system_radius_world = world.get_system_radius(entity_id);

            let label_pos = PointF64 {
                x: entity_world_x_f64 - (text_width_world / 2.0),
                y: entity_world_y_f64 + system_radius_world,
            };

            render_text_in_world(
                canvas,
                renderer,
                &text,
                label_pos,
                font_size_world,
                sdl2::pixels::Color::RGBA(colors::LGRAY.r, colors::LGRAY.g, colors::LGRAY.b, 220),
                ctx,
            );
        }
    } else {
        const STAR_LABEL_MIN_ZOOM: f64 = 0.7;
        if viewport.zoom > STAR_LABEL_MIN_ZOOM {
            if let Some(name) = world.get_entity_name(entity_id) {
                let text = name.to_lowercase();

                const STAR_LABEL_FONT_SIZE_WORLD: f64 = 1.2;
                let char_width_world = STAR_LABEL_FONT_SIZE_WORLD;
                let text_width_world = text.chars().count() as f64 * char_width_world;

                let label_pos = PointF64 {
                    x: entity_world_x_f64 - (text_width_world / 2.0),
                    y: entity_world_y_f64 + 1.1,
                };

                render_text_in_world(
                    canvas,
                    renderer,
                    &text,
                    label_pos,
                    STAR_LABEL_FONT_SIZE_WORLD,
                    sdl2::pixels::Color::RGBA(
                        colors::LGRAY.r,
                        colors::LGRAY.g,
                        colors::LGRAY.b,
                        220,
                    ),
                    ctx,
                );
            }
        }
    }
}

fn draw_entities(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    viewport: &Viewport,
    ctx: &ViewportRenderContext,
    selection: &[EntityId],
) {
    let selection_set: std::collections::HashSet<EntityId> = selection.iter().cloned().collect();

    let mut clusters: Vec<(PointF64, Vec<EntityId>)> = Vec::new();
    let mut processed_entities = std::collections::HashSet::new();

    const CLUSTER_RADIUS_WORLD: f64 = 2.0;

    for entity_id in world.iter_entities() {
        if processed_entities.contains(&entity_id) {
            continue;
        }

        let entity_type = world.get_entity_type(entity_id);
        if !matches!(entity_type, Some(EntityType::Ship)) {
            continue;
        }

        if let Some(pos) = world.get_location_f64(entity_id) {
            let mut current_cluster_center = pos;
            let mut cluster_members = vec![entity_id];
            processed_entities.insert(entity_id);

            for other_id in world.iter_entities() {
                if processed_entities.contains(&other_id) {
                    continue;
                }
                if let Some(other_pos) = world.get_location_f64(other_id) {
                    let distance =
                        ((pos.x - other_pos.x).powi(2) + (pos.y - other_pos.y).powi(2)).sqrt();
                    if distance < CLUSTER_RADIUS_WORLD {
                        cluster_members.push(other_id);
                        processed_entities.insert(other_id);
                        // update center
                        current_cluster_center.x = (current_cluster_center.x
                            * (cluster_members.len() - 1) as f64
                            + other_pos.x)
                            / cluster_members.len() as f64;
                        current_cluster_center.y = (current_cluster_center.y
                            * (cluster_members.len() - 1) as f64
                            + other_pos.y)
                            / cluster_members.len() as f64;
                    }
                }
            }
            clusters.push((current_cluster_center, cluster_members));
        }
    }

    // draw non-clustered entities
    for entity_id in world.iter_entities() {
        let entity_type = world.get_entity_type(entity_id);
        if matches!(entity_type, Some(EntityType::Ship)) {
            continue;
        }

        if viewport.zoom < STAR_MAP_ZOOM_THRESHOLD && !matches!(entity_type, Some(EntityType::Star))
        {
            continue;
        }

        if let Some(pos) = world.get_location_f64(entity_id) {
            if !is_in_view(pos, ctx) {
                continue;
            }

            let entity_world_x_f64 = pos.x;
            let entity_world_y_f64 = pos.y;

            let screen_pixel_x_float = (entity_world_x_f64 - ctx.view_world_origin_x)
                * ctx.world_tile_actual_pixel_size_on_screen;
            let screen_pixel_y_float = (entity_world_y_f64 - ctx.view_world_origin_y)
                * ctx.world_tile_actual_pixel_size_on_screen;

            let dest_x = screen_pixel_x_float.round() as i32;
            let dest_y = screen_pixel_y_float.round() as i32;

            let mut draw_sprite_ctx = DrawSpriteContext {
                canvas,
                renderer,
                world,
                viewport,
                ctx,
            };

            let dest_rect_on_screen =
                draw_entity_sprite(&mut draw_sprite_ctx, entity_id, entity_type, dest_x, dest_y);

            if selection_set.contains(&entity_id) {
                draw_selection_outline(canvas, &dest_rect_on_screen);
            }

            if let Some(EntityType::Star) = entity_type {
                draw_star_label(canvas, renderer, world, viewport, entity_id, pos, ctx);
            }
        }
    }

    // draw clusters
    for (center, members) in clusters {
        if !is_in_view(center, ctx) {
            continue;
        }
        let entity_world_x_f64 = center.x;
        let entity_world_y_f64 = center.y;

        let screen_pixel_x_float = (entity_world_x_f64 - ctx.view_world_origin_x)
            * ctx.world_tile_actual_pixel_size_on_screen;
        let screen_pixel_y_float = (entity_world_y_f64 - ctx.view_world_origin_y)
            * ctx.world_tile_actual_pixel_size_on_screen;

        let dest_x = screen_pixel_x_float.round() as i32;
        let dest_y = screen_pixel_y_float.round() as i32;

        let mut draw_sprite_ctx = DrawSpriteContext {
            canvas,
            renderer,
            world,
            viewport,
            ctx,
        };

        // For now, just render the first ship in the cluster
        let representative_id = members[0];
        let dest_rect_on_screen = draw_entity_sprite(
            &mut draw_sprite_ctx,
            representative_id,
            world.get_entity_type(representative_id),
            dest_x,
            dest_y,
        );

        if members.iter().any(|id| selection_set.contains(id)) {
            draw_selection_outline(canvas, &dest_rect_on_screen);
        }

        if members.len() > 1 {
            let text = format!("{}", members.len());
            let label_pos = PointF64 {
                x: center.x + 0.5,
                y: center.y + 0.5,
            };
            render_text_in_world(
                canvas,
                renderer,
                &text,
                label_pos,
                0.8, // font size in world units
                colors::WHITE,
                ctx,
            );
        }
    }
}

pub fn render_world_in_viewport(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    viewport: &Viewport,
    controls: &ControlState,
    selection: &[EntityId],
) {
    let world_tile_actual_pixel_size_on_screen =
        (TILE_PIXEL_WIDTH as f64 * viewport.zoom).max(0.001);

    let view_world_origin_x = viewport.anchor.x
        - (viewport.screen_pixel_width as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;
    let view_world_origin_y = viewport.anchor.y
        - (viewport.screen_pixel_height as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;

    let visible_world_width_in_tiles_float =
        viewport.screen_pixel_width as f64 / world_tile_actual_pixel_size_on_screen;
    let visible_world_height_in_tiles_float =
        viewport.screen_pixel_height as f64 / world_tile_actual_pixel_size_on_screen;

    let ctx = ViewportRenderContext {
        view_world_origin_x,
        view_world_origin_y,
        world_tile_actual_pixel_size_on_screen,
        view_bbox_world_x_min: view_world_origin_x,
        view_bbox_world_x_max: view_world_origin_x + visible_world_width_in_tiles_float,
        view_bbox_world_y_min: view_world_origin_y,
        view_bbox_world_y_max: view_world_origin_y + visible_world_height_in_tiles_float,
        tile_on_screen_render_w: world_tile_actual_pixel_size_on_screen.round().max(1.0) as u32,
    };

    draw_star_lanes(canvas, world, &ctx);
    draw_orbit_lines(canvas, world, viewport, &ctx);
    draw_entities(canvas, renderer, world, viewport, &ctx, selection);
    draw_move_orders(canvas, world, selection, &ctx);

    if controls.debug_enabled {
        draw_viewport_border(canvas, viewport);
        draw_grid(canvas, viewport, &ctx);
    }
    if let (Some(start), Some(end)) = (controls.selection_box_start, controls.last_mouse_pos) {
        draw_selection_box(canvas, start, end);
    }
}

fn draw_viewport_border(canvas: &mut Canvas<Window>, viewport: &Viewport) {
    canvas.set_draw_color(colors::RED);
    // The border should span the full intended pixel width/height of the viewport,
    let border_rect = Rect::new(
        0,
        0,
        viewport.screen_pixel_width,
        viewport.screen_pixel_height,
    );
    canvas.draw_rect(border_rect).unwrap();
}

fn draw_circle(canvas: &mut Canvas<Window>, center_x: i32, center_y: i32, radius: i32) {
    if radius <= 0 {
        return;
    }
    let diameter = radius * 2;

    let mut x = radius - 1;
    let mut y = 0;
    let mut tx = 1;
    let mut ty = 1;
    let mut error = tx - diameter;

    let mut points = vec![];
    while x >= y {
        points.push(sdl2::rect::Point::new(center_x + x, center_y - y));
        points.push(sdl2::rect::Point::new(center_x + x, center_y + y));
        points.push(sdl2::rect::Point::new(center_x - x, center_y - y));
        points.push(sdl2::rect::Point::new(center_x - x, center_y + y));
        points.push(sdl2::rect::Point::new(center_x + y, center_y - x));
        points.push(sdl2::rect::Point::new(center_x + y, center_y + x));
        points.push(sdl2::rect::Point::new(center_x - y, center_y - x));
        points.push(sdl2::rect::Point::new(center_x - y, center_y + x));

        if error <= 0 {
            y += 1;
            error += ty;
            ty += 2;
        }

        if error > 0 {
            x -= 1;
            tx += 2;
            error += tx - diameter;
        }
    }
    let _ = canvas.draw_points(&points[..]);
}

pub struct Viewport {
    /// Specifies which universe coordinate the center of the viewport is looking at.
    pub anchor: PointF64,
    /// Magnification level. zoom > 1.0 means world tiles appear larger, zoom < 1.0 means they appear smaller.
    pub zoom: f64,
    /// The width of the viewport's rendering area on the screen, in pixels.
    pub screen_pixel_width: u32,
    /// The height of the viewport's rendering area on the screen, in pixels.
    pub screen_pixel_height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            anchor: PointF64 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            screen_pixel_width: INITIAL_WINDOW_WIDTH,
            screen_pixel_height: INITIAL_WINDOW_HEIGHT,
        }
    }
}

impl Viewport {
    pub fn world_tile_pixel_size_on_screen(&self) -> f64 {
        (TILE_PIXEL_WIDTH as f64 * self.zoom).max(0.001)
    }

    pub fn screen_to_world_coords(&self, screen_x: i32, screen_y: i32) -> PointF64 {
        let world_tile_actual_pixel_size_on_screen =
            (TILE_PIXEL_WIDTH as f64 * self.zoom).max(0.001);

        let view_world_origin_x = self.anchor.x
            - (self.screen_pixel_width as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;
        let view_world_origin_y = self.anchor.y
            - (self.screen_pixel_height as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;

        let world_x =
            view_world_origin_x + (screen_x as f64 / world_tile_actual_pixel_size_on_screen);
        let world_y =
            view_world_origin_y + (screen_y as f64 / world_tile_actual_pixel_size_on_screen);

        PointF64 {
            x: world_x,
            y: world_y,
        }
    }

    pub fn world_to_screen_coords(&self, world_pos: crate::location::Point) -> (i32, i32) {
        let world_tile_actual_pixel_size_on_screen = self.world_tile_pixel_size_on_screen();

        let view_world_origin_x = self.anchor.x
            - (self.screen_pixel_width as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;
        let view_world_origin_y = self.anchor.y
            - (self.screen_pixel_height as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;

        let screen_x =
            (world_pos.x as f64 - view_world_origin_x) * world_tile_actual_pixel_size_on_screen;
        let screen_y =
            (world_pos.y as f64 - view_world_origin_y) * world_tile_actual_pixel_size_on_screen;

        (screen_x.round() as i32, screen_y.round() as i32)
    }

    pub fn center_on_entity(&mut self, x: i32, y: i32) {
        self.anchor.x = x as f64;
        self.anchor.y = y as f64;
    }

    pub fn zoom_in(&mut self) {
        self.zoom *= 1.2;
        self.zoom = self.zoom.clamp(0.05, 10.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom /= 1.2;
        self.zoom = self.zoom.clamp(0.05, 10.0);
    }

    pub fn zoom_at(&mut self, zoom_factor: f64, mouse_screen_pos: (i32, i32)) {
        let world_pos_before_zoom =
            self.screen_to_world_coords(mouse_screen_pos.0, mouse_screen_pos.1);

        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(0.05, 10.0);

        let new_world_tile_pixel_size = (TILE_PIXEL_WIDTH as f64 * self.zoom).max(0.001);
        let mouse_offset_from_center_x =
            mouse_screen_pos.0 as f64 - self.screen_pixel_width as f64 / 2.0;
        let mouse_offset_from_center_y =
            mouse_screen_pos.1 as f64 - self.screen_pixel_height as f64 / 2.0;

        self.anchor.x =
            world_pos_before_zoom.x - mouse_offset_from_center_x / new_world_tile_pixel_size;
        self.anchor.y =
            world_pos_before_zoom.y - mouse_offset_from_center_y / new_world_tile_pixel_size;
    }
}

fn render_text_in_world(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    text: &str,
    world_pos: PointF64,  // top-left of the text
    font_size_world: f64, // font size in world units (tile size)
    color: sdl2::pixels::Color,
    ctx: &ViewportRenderContext,
) {
    let char_width_world = font_size_world;

    renderer.set_texture_color_mod(color.r, color.g, color.b);
    let original_alpha = renderer.texture_ref().alpha_mod();
    renderer.texture.borrow_mut().set_alpha_mod(color.a);
    let old_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    for (i, ch) in text.chars().enumerate() {
        // Don't render spaces.
        if ch == ' ' {
            continue;
        }
        let char_world_x = world_pos.x + i as f64 * char_width_world;
        let char_world_y = world_pos.y;

        let src = renderer.tileset.get_rect(ch);

        let on_screen_pixel_size = font_size_world * ctx.world_tile_actual_pixel_size_on_screen;

        // Convert world coordinates to screen coordinates
        let screen_x =
            (char_world_x - ctx.view_world_origin_x) * ctx.world_tile_actual_pixel_size_on_screen;
        let screen_y =
            (char_world_y - ctx.view_world_origin_y) * ctx.world_tile_actual_pixel_size_on_screen;

        let dst = sdl2::rect::Rect::new(
            screen_x.round() as i32,
            screen_y.round() as i32,
            on_screen_pixel_size.round().max(1.0) as u32,
            on_screen_pixel_size.round().max(1.0) as u32,
        );

        canvas
            .copy(&renderer.texture_ref(), Some(src), Some(dst))
            .ok();
    }
    renderer.texture.borrow_mut().set_alpha_mod(original_alpha);
    canvas.set_blend_mode(old_blend_mode);
}

fn draw_selection_box(canvas: &mut Canvas<Window>, start: (i32, i32), end: (i32, i32)) {
    let x = start.0.min(end.0);
    let y = start.1.min(end.1);
    let width = (start.0 - end.0).unsigned_abs();
    let height = (start.1 - end.1).unsigned_abs();

    let rect = Rect::new(x, y, width, height);

    canvas.set_draw_color(colors::WHITE);
    canvas.draw_rect(rect).ok();
}

fn draw_grid(canvas: &mut Canvas<Window>, viewport: &Viewport, ctx: &ViewportRenderContext) {
    let old_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(
        colors::DGRAY.r,
        colors::DGRAY.g,
        colors::DGRAY.b,
        50, // softly visible
    ));

    // vertical lines
    let x_start = ctx.view_bbox_world_x_min.floor() as i32;
    let x_end = ctx.view_bbox_world_x_max.ceil() as i32;
    for x_world in x_start..=x_end {
        let x_screen = ((x_world as f64 - ctx.view_world_origin_x)
            * ctx.world_tile_actual_pixel_size_on_screen)
            .round() as i32;
        canvas
            .draw_line(
                (x_screen, 0),
                (x_screen, viewport.screen_pixel_height as i32),
            )
            .ok();
    }

    // horizontal lines
    let y_start = ctx.view_bbox_world_y_min.floor() as i32;
    let y_end = ctx.view_bbox_world_y_max.ceil() as i32;
    for y_world in y_start..=y_end {
        let y_screen = ((y_world as f64 - ctx.view_world_origin_y)
            * ctx.world_tile_actual_pixel_size_on_screen)
            .round() as i32;
        canvas
            .draw_line(
                (0, y_screen),
                (viewport.screen_pixel_width as i32, y_screen),
            )
            .ok();
    }

    canvas.set_blend_mode(old_blend_mode);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialization::{INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH};

    #[test]
    fn default_viewport() {
        let vp = Viewport::default();
        assert_eq!(vp.anchor, PointF64 { x: 0.0, y: 0.0 });
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.screen_pixel_width, INITIAL_WINDOW_WIDTH);
        assert_eq!(vp.screen_pixel_height, INITIAL_WINDOW_HEIGHT);
    }

    #[test]
    fn test_center_on_entity() {
        let mut vp = Viewport::default();
        vp.center_on_entity(10, 20);
        assert_eq!(vp.anchor, PointF64 { x: 10.0, y: 20.0 });
    }

    #[test]
    fn test_zoom_in_out() {
        let mut vp = Viewport::default();
        let original_zoom = vp.zoom;
        vp.zoom_in();
        assert!(vp.zoom > original_zoom);
        vp.zoom_out();
        // zoom_out should bring it back close to original
        let diff = (vp.zoom - original_zoom).abs();
        assert!(diff < f64::EPSILON);
    }

    #[test]
    fn test_screen_to_world_coords() {
        let mut vp = Viewport {
            anchor: PointF64 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            screen_pixel_width: 800,
            screen_pixel_height: 600,
        };

        // case 1: no zoom, anchor at origin
        vp.zoom = 1.0;
        vp.anchor = PointF64 { x: 0.0, y: 0.0 };
        let coords = vp.screen_to_world_coords(400, 300); // screen center
        assert!((coords.x - 0.0).abs() < 1e-9);
        assert!((coords.y - 0.0).abs() < 1e-9);

        // case 2: zoomed in, anchor at origin
        vp.zoom = 2.0;
        let coords = vp.screen_to_world_coords(400, 300); // screen center
        assert!((coords.x - 0.0).abs() < 1e-9);
        assert!((coords.y - 0.0).abs() < 1e-9);
        // top-left screen should be top-left of smaller world view
        let tile_size = TILE_PIXEL_WIDTH as f64;
        let expected_x = 0.0 - (800.0 / 2.0) / (tile_size * 2.0);
        let expected_y = 0.0 - (600.0 / 2.0) / (tile_size * 2.0);
        let coords_tl = vp.screen_to_world_coords(0, 0);
        assert!((coords_tl.x - expected_x).abs() < 1e-9);
        assert!((coords_tl.y - expected_y).abs() < 1e-9);

        // case 3: zoomed out, anchor offset
        vp.zoom = 0.5;
        vp.anchor = PointF64 { x: 100.0, y: -50.0 };
        let coords_center = vp.screen_to_world_coords(400, 300); // screen center
        assert!((coords_center.x - 100.0).abs() < 1e-9);
        assert!((coords_center.y - -50.0).abs() < 1e-9);
    }
}
