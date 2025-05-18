use crate::colors;
use crate::render::{tileset, SpriteSheetRenderer};
use crate::world::{EntityId, World};
use sdl2::render::Canvas;
use sdl2::video::Window;

pub mod build;
pub mod resources_panel;
pub mod selected_object_panel;
pub mod sim_speed_panel;

/// Helper to render text aligned at the given (x,y) tile coordinates.
/// This mirrors the implementation found in `render_status_text` but
/// allows specifying the x position instead of always right-aligning.
pub fn render_text_at(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    text: &str,
    background_color: sdl2::pixels::Color,
    foreground_color: sdl2::pixels::Color,
    x_tile: u8,
    y_tile: u8,
) {
    // draw background rectangle behind the text
    canvas.set_draw_color(background_color);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            x_tile,
            y_tile,
            text.len() as u8,
            1,
        ))
        .unwrap();

    renderer
        .texture
        .set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

    for (i, ch) in text.chars().enumerate() {
        let src = renderer.tileset.get_rect(ch);
        let dst = tileset::make_tile_rect(x_tile + i as u8, y_tile);
        canvas.copy(renderer.texture, Some(src), Some(dst)).ok();
    }
}

// Constants for panels
pub const PANEL_BORDER_COLOR: sdl2::pixels::Color = colors::GRAY;
pub const PANEL_BACKGROUND_COLOR: sdl2::pixels::Color = colors::BLACK;
pub const PANEL_TEXT_COLOR: sdl2::pixels::Color = colors::WHITE;

/// Render resource counters (top-left), selection panel (bottom-left) and sim speed (top-right).
///
/// This function should be called once per frame after the world has
/// been updated and the main viewport rendered.
pub fn render_interface(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    world: &World,
    selected: Option<EntityId>,
    viewport_height_tiles: u32,
    controls: &crate::event_handling::ControlState,
) {
    resources_panel::render_resources_panel(canvas, renderer, world);
    selected_object_panel::render_selected_object_panel(
        canvas,
        renderer,
        world,
        selected, // Use the 'selected' parameter directly
        controls.track_mode,
        viewport_height_tiles, // Use the 'viewport_height_tiles' parameter directly
    );
    sim_speed_panel::render_sim_speed_panel(canvas, renderer, controls.sim_speed, controls.paused);
}
