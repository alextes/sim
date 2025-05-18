pub mod tileset;
pub mod viewport;

use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use tracing::error;

use crate::render::tileset::Tileset;

// Re-export key items from the viewport module
pub use viewport::{render_viewport, Viewport};

pub const TILE_PIXEL_WIDTH: u8 = 9;

// Define the new SpriteSheetRenderer struct
pub struct SpriteSheetRenderer<'a, 't> {
    pub tileset: &'a Tileset,
    pub texture: &'a mut Texture<'t>,
}

pub fn render_status_text(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
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

    renderer
        .texture
        .set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

    let chars = text.chars();

    for (i, char) in chars.enumerate() {
        let res = canvas.copy(
            renderer.texture,
            Some(renderer.tileset.get_rect(char)),
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
