use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use tracing::error;

use crate::colors;
use crate::location::PointF64;
use crate::world::World;

use super::{SpriteSheetRenderer, TILE_PIXEL_WIDTH};

const PLANET_ORBIT_MIN_ZOOM: f64 = 0.8;
const MOON_ORBIT_MIN_ZOOM: f64 = 2.0;

struct ViewportRenderContext {
    view_world_origin_x: f64,
    view_world_origin_y: f64,
    world_tile_actual_pixel_size_on_screen: f64,
    view_bbox_world_x_min: f64,
    view_bbox_world_x_max: f64,
    view_bbox_world_y_min: f64,
    view_bbox_world_y_max: f64,
    tile_on_screen_render_w: u32,
    tile_on_screen_render_h: u32,
}

fn draw_star_lanes(canvas: &mut Canvas<Window>, world: &World, ctx: &ViewportRenderContext) {
    let old_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(
        colors::LGRAY.r,
        colors::LGRAY.g,
        colors::LGRAY.b,
        30, // subtle alpha
    ));
    for &(a, b) in world.iter_lanes() {
        if let (Some(pos_a), Some(pos_b)) = (world.get_location(a), world.get_location(b)) {
            let ax = (pos_a.x as f64 + 0.5 - ctx.view_world_origin_x)
                * ctx.world_tile_actual_pixel_size_on_screen;
            let ay = (pos_a.y as f64 + 0.5 - ctx.view_world_origin_y)
                * ctx.world_tile_actual_pixel_size_on_screen;
            let bx = (pos_b.x as f64 + 0.5 - ctx.view_world_origin_x)
                * ctx.world_tile_actual_pixel_size_on_screen;
            let by = (pos_b.y as f64 + 0.5 - ctx.view_world_origin_y)
                * ctx.world_tile_actual_pixel_size_on_screen;
            let p1 = sdl2::rect::Point::new(ax.round() as i32, ay.round() as i32);
            let p2 = sdl2::rect::Point::new(bx.round() as i32, by.round() as i32);
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
        let glyph = world.get_render_glyph(entity_id);
        let anchor_pos = match world.get_location(orbital_info.anchor) {
            Some(pos) => pos,
            None => continue,
        };

        let (should_draw, color) = match glyph {
            'p' if viewport.zoom >= PLANET_ORBIT_MIN_ZOOM => (
                true,
                sdl2::pixels::Color::RGBA(colors::LGRAY.r, colors::LGRAY.g, colors::LGRAY.b, 20),
            ),
            'm' if viewport.zoom >= MOON_ORBIT_MIN_ZOOM => (
                true,
                sdl2::pixels::Color::RGBA(colors::DGRAY.r, colors::DGRAY.g, colors::DGRAY.b, 15),
            ),
            _ => (false, sdl2::pixels::Color::BLACK), // don't draw
        };

        if should_draw {
            let center_x = (anchor_pos.x as f64 + 0.5 - ctx.view_world_origin_x)
                * ctx.world_tile_actual_pixel_size_on_screen;
            let center_y = (anchor_pos.y as f64 + 0.5 - ctx.view_world_origin_y)
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

fn draw_entities(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    viewport: &Viewport,
    ctx: &ViewportRenderContext,
) {
    for entity_id in world.iter_entities() {
        if let Some(pos) = world.get_location(entity_id) {
            let entity_world_x_f64 = pos.x as f64;
            let entity_world_y_f64 = pos.y as f64;

            if entity_world_x_f64 + 1.0 > ctx.view_bbox_world_x_min
                && entity_world_x_f64 < ctx.view_bbox_world_x_max
                && entity_world_y_f64 + 1.0 > ctx.view_bbox_world_y_min
                && entity_world_y_f64 < ctx.view_bbox_world_y_max
            {
                let glyph = world.get_render_glyph(entity_id);
                let src_rect_in_tileset = renderer.tileset.get_rect(glyph);

                let screen_pixel_x_float = (entity_world_x_f64 - ctx.view_world_origin_x)
                    * ctx.world_tile_actual_pixel_size_on_screen;
                let screen_pixel_y_float = (entity_world_y_f64 - ctx.view_world_origin_y)
                    * ctx.world_tile_actual_pixel_size_on_screen;

                let dest_x = screen_pixel_x_float.round() as i32;
                let dest_y = screen_pixel_y_float.round() as i32;
                let dest_rect_on_screen = Rect::new(
                    dest_x,
                    dest_y,
                    ctx.tile_on_screen_render_w,
                    ctx.tile_on_screen_render_h,
                );

                if let Some(color) = world.get_entity_color(entity_id) {
                    renderer.set_texture_color_mod(color.r, color.g, color.b);
                } else {
                    // fallback to a default color if none is set.
                    renderer.set_texture_color_mod(255, 255, 255); // white
                }

                canvas
                    .copy(
                        &renderer.texture_ref(),
                        Some(src_rect_in_tileset),
                        Some(dest_rect_on_screen),
                    )
                    .unwrap_or_else(|e| {
                        error!("failed to copy tile for entity {}: {:?}", entity_id, e)
                    });

                const STAR_LABEL_MIN_ZOOM: f64 = 0.7;
                if glyph == '*' && viewport.zoom > STAR_LABEL_MIN_ZOOM {
                    if let Some(name) = world.get_entity_name(entity_id) {
                        let text = name.to_lowercase();

                        const STAR_LABEL_FONT_SIZE_WORLD: f64 = 0.3;
                        let char_width_world = STAR_LABEL_FONT_SIZE_WORLD;
                        let text_width_world = text.chars().count() as f64 * char_width_world;

                        let label_pos = PointF64 {
                            x: (entity_world_x_f64 + 0.5) - (text_width_world / 2.0),
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
    }
}

pub fn render_world_in_viewport(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    viewport: &Viewport,
    debug_enabled: bool,
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
        tile_on_screen_render_h: world_tile_actual_pixel_size_on_screen.round().max(1.0) as u32,
    };

    draw_star_lanes(canvas, world, &ctx);
    draw_orbit_lines(canvas, world, viewport, &ctx);
    draw_entities(canvas, renderer, world, viewport, &ctx);

    if debug_enabled {
        draw_viewport_border(canvas, viewport);
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
        const DEFAULT_TILES_WIDE: u32 = 64;
        const DEFAULT_TILES_HIGH: u32 = 64;
        Self {
            anchor: PointF64 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            screen_pixel_width: DEFAULT_TILES_WIDE * TILE_PIXEL_WIDTH as u32,
            screen_pixel_height: DEFAULT_TILES_HIGH * TILE_PIXEL_WIDTH as u32,
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

    pub fn center_on_entity(&mut self, x: i32, y: i32) {
        self.anchor.x = x as f64;
        self.anchor.y = y as f64;
    }

    pub fn zoom_in(&mut self) {
        self.zoom *= 1.2;
        self.zoom = self.zoom.clamp(0.1, 10.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom /= 1.2;
        self.zoom = self.zoom.clamp(0.1, 10.0);
    }

    pub fn zoom_at(&mut self, zoom_factor: f64, mouse_screen_pos: (i32, i32)) {
        let world_pos_before_zoom =
            self.screen_to_world_coords(mouse_screen_pos.0, mouse_screen_pos.1);

        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(0.1, 10.0);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_viewport() {
        let vp = Viewport::default();
        assert_eq!(vp.anchor, PointF64 { x: 0.0, y: 0.0 });
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.screen_pixel_width, 64 * TILE_PIXEL_WIDTH as u32);
        assert_eq!(vp.screen_pixel_height, 64 * TILE_PIXEL_WIDTH as u32);
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
}
