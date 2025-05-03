mod tileset;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use tracing::error;

use crate::colors;
use crate::location::Point;
use crate::world::World;

pub const TILE_PIXEL_WIDTH: u8 = 9;

pub struct Renderable {
    pub color: Color,
    pub tileset_rect: Rect,
    pub x: u8,
    pub y: u8,
}

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
    let tile_pixel_width = (TILE_PIXEL_WIDTH as f64 * viewport.zoom) as u32;
    let half_w = viewport.width as i32 / 2;
    let half_h = viewport.height as i32 / 2;
    for entity_id in world.iter_entities() {
        if let Some(pos) = world.get_location(entity_id) {
            if pos.x >= viewport.anchor.x - half_w
                && pos.x < viewport.anchor.x + half_w
                && pos.y >= viewport.anchor.y - half_h
                && pos.y < viewport.anchor.y + half_h
            {
                let screen_x = (pos.x - (viewport.anchor.x - half_w)) as u8;
                let screen_y = (pos.y - (viewport.anchor.y - half_h)) as u8;
                if let Some(glyph) = world.get_render_glyph(entity_id) {
                    let renderable = Renderable {
                        x: screen_x,
                        y: screen_y,
                        tileset_rect: tileset::rect_from_char(glyph),
                        color: colors::BLUE,
                    };
                    render_tile_with_zoom(canvas, tiles_texture, &renderable, tile_pixel_width);
                }
            }
        }
    }

    if debug_enabled {
        draw_viewport_border(canvas, viewport);
    }
}

fn draw_viewport_border(canvas: &mut Canvas<Window>, viewport: &Viewport) {
    canvas.set_draw_color(colors::RED);
    let tile_w = (TILE_PIXEL_WIDTH as f64 * viewport.zoom) as u32;
    let border_rect = Rect::new(0, 0, viewport.width * tile_w, viewport.height * tile_w);
    canvas.draw_rect(border_rect).unwrap();
}

fn render_tile_with_zoom(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    renderable: &Renderable,
    tile_pixel_width: u32,
) {
    tiles_texture.set_color_mod(renderable.color.r, renderable.color.g, renderable.color.b);

    canvas
        .copy(
            tiles_texture,
            Some(renderable.tileset_rect),
            Some(Rect::new(
                renderable.x as i32 * tile_pixel_width as i32,
                renderable.y as i32 * tile_pixel_width as i32,
                tile_pixel_width,
                tile_pixel_width,
            )),
        )
        .unwrap();
}

pub struct Viewport {
    /// Specifies which universe coordinate the center of the viewport is looking at.
    pub anchor: Point,
    /// Specifies how far we're zoomed in on the universe, and therefore how many tiles are visible.
    pub zoom: f64,
    pub width: u32,
    pub height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            anchor: Point { x: 0, y: 0 },
            zoom: 1.0,
            width: 64,
            height: 64,
        }
    }
}

impl Viewport {
    pub fn center_on_entity(&mut self, x: i32, y: i32) {
        self.anchor.x = x;
        self.anchor.y = y;
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
    use crate::location::Point;

    #[test]
    fn default_viewport() {
        let vp = Viewport::default();
        assert_eq!(vp.anchor, Point { x: 0, y: 0 });
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.width, 64);
        assert_eq!(vp.height, 64);
    }

    #[test]
    fn test_center_on_entity() {
        let mut vp = Viewport::default();
        vp.center_on_entity(10, 20);
        assert_eq!(vp.anchor, Point { x: 10, y: 20 });
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
