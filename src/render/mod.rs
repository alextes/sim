pub mod tileset;
pub mod viewport;

use sdl2::render::Texture;

use crate::render::tileset::Tileset;

// Re-export key items from the viewport module
pub use viewport::{render_viewport, Viewport};

pub const TILE_PIXEL_WIDTH: u8 = 9;

// Define the new SpriteSheetRenderer struct
pub struct SpriteSheetRenderer<'a, 't> {
    pub tileset: &'a Tileset,
    pub texture: &'a mut Texture<'t>,
}
