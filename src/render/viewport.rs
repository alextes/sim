use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use tracing::error;

use crate::colors;
use crate::location::PointF64;
use crate::world::World;

use super::{SpriteSheetRenderer, TILE_PIXEL_WIDTH};

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

fn draw_entities(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
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

                renderer.set_texture_color_mod(colors::BLUE.r, colors::BLUE.g, colors::BLUE.b);

                canvas
                    .copy(
                        &renderer.texture_ref(),
                        Some(src_rect_in_tileset),
                        Some(dest_rect_on_screen),
                    )
                    .unwrap_or_else(|e| {
                        error!("failed to copy tile for entity {}: {:?}", entity_id, e)
                    });
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
    draw_entities(canvas, renderer, world, &ctx);

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
    pub fn center_on_entity(&mut self, x: i32, y: i32) {
        self.anchor.x = x as f64;
        self.anchor.y = y as f64;
    }

    pub fn zoom_in(&mut self) {
        self.zoom *= 1.2;
    }

    pub fn zoom_out(&mut self) {
        self.zoom /= 1.2;
    }
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
