pub mod tileset;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use tracing::error;

use crate::colors;
use crate::location::PointF64;
use crate::world::World;

pub const TILE_PIXEL_WIDTH: u8 = 9;

pub fn render_status_text(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    text: &str,
    background_color: Color,
    foreground_color: Color,
    y_offset: u8,
) {
    canvas.set_draw_color(background_color);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            (64 - text.len()) as u8,
            y_offset,
            text.len() as u8,
            1,
        ))
        .unwrap();

    tiles_texture.set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

    let chars = text.chars();

    for (i, char) in chars.enumerate() {
        let res = canvas.copy(
            tiles_texture,
            Some(tileset::rect_from_char(char)),
            Some(tileset::make_tile_rect(
                (64 - text.len() + i).try_into().unwrap(),
                y_offset,
            )),
        );

        if res.is_err() {
            error!("failed to copy tile: {:?}", res.err());
        }
    }
}

pub fn render_viewport(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    world: &World,
    viewport: &Viewport,
    debug_enabled: bool,
) {
    // Target pixel dimensions of the viewport on screen.
    let target_screen_pixel_width = viewport.screen_pixel_width;
    let target_screen_pixel_height = viewport.screen_pixel_height;

    // Actual pixel size one full world tile would take on screen, as a float.
    // Ensure it's at least a tiny positive value to avoid division by zero.
    let world_tile_actual_pixel_size_on_screen =
        (TILE_PIXEL_WIDTH as f64 * viewport.zoom).max(0.001);

    // World coordinates (floating point, e.g., 10.5, 20.3) that align
    // with the top-left pixel (0,0) of our viewport rendering area.
    // viewport.anchor is the center of the view in world coordinates.
    let view_world_origin_x = viewport.anchor.x
        - (target_screen_pixel_width as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;
    let view_world_origin_y = viewport.anchor.y
        - (target_screen_pixel_height as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;

    // Calculate the width/height of the viewport in terms of world tile units (float)
    let visible_world_width_in_tiles_float =
        target_screen_pixel_width as f64 / world_tile_actual_pixel_size_on_screen;
    let visible_world_height_in_tiles_float =
        target_screen_pixel_height as f64 / world_tile_actual_pixel_size_on_screen;

    // Bounding box of the viewport in world coordinates (float)
    let view_bbox_world_x_min = view_world_origin_x;
    let view_bbox_world_x_max = view_world_origin_x + visible_world_width_in_tiles_float;
    let view_bbox_world_y_min = view_world_origin_y;
    let view_bbox_world_y_max = view_world_origin_y + visible_world_height_in_tiles_float;

    // Pre-calculate the on-screen_render_width/height for each tile for this frame, ensuring it's at least 1 pixel.
    let tile_on_screen_render_w = world_tile_actual_pixel_size_on_screen.round().max(1.0) as u32;
    let tile_on_screen_render_h = world_tile_actual_pixel_size_on_screen.round().max(1.0) as u32;

    // Iterate over entities once
    for entity_id in world.iter_entities() {
        if let Some(pos) = world.get_location(entity_id) {
            // pos is Point {i32, i32}
            let entity_world_x_f64 = pos.x as f64;
            let entity_world_y_f64 = pos.y as f64;

            // Check if the tile occupied by the entity intersects the viewport's world bounding box
            // A tile at (pos.x, pos.y) covers the world area [pos.x, pos.x+1) and [pos.y, pos.y+1)
            if entity_world_x_f64 + 1.0 > view_bbox_world_x_min
                && entity_world_x_f64 < view_bbox_world_x_max
                && entity_world_y_f64 + 1.0 > view_bbox_world_y_min
                && entity_world_y_f64 < view_bbox_world_y_max
            {
                let glyph = world.get_render_glyph(entity_id);
                let src_rect_in_tileset = tileset::rect_from_char(glyph);

                // Calculate the top-left screen pixel position (float) for this tile.
                // This calculation remains the same: based on the entity's integer tile position (pos.x, pos.y)
                // relative to the viewport's float world origin.
                let screen_pixel_x_float = (entity_world_x_f64 - view_world_origin_x)
                    * world_tile_actual_pixel_size_on_screen;
                let screen_pixel_y_float = (entity_world_y_f64 - view_world_origin_y)
                    * world_tile_actual_pixel_size_on_screen;

                // Destination Rect on screen
                let dest_x = screen_pixel_x_float.round() as i32;
                let dest_y = screen_pixel_y_float.round() as i32;
                let dest_rect_on_screen = Rect::new(
                    dest_x,
                    dest_y,
                    tile_on_screen_render_w,
                    tile_on_screen_render_h,
                );

                tiles_texture.set_color_mod(colors::BLUE.r, colors::BLUE.g, colors::BLUE.b); // Assuming a default color for now

                canvas
                    .copy(
                        tiles_texture,
                        Some(src_rect_in_tileset),
                        Some(dest_rect_on_screen),
                    )
                    .unwrap_or_else(|e| {
                        error!("failed to copy tile for entity {}: {:?}", entity_id, e)
                    });
            }
        }
    }

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
        self.zoom *= 1.1;
    }

    pub fn zoom_out(&mut self) {
        self.zoom /= 1.1;
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
