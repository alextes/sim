use crate::colors;
use crate::render::render_status_text;
use crate::render::SpriteSheetRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Renders debug overlay: simulation updates/sec, frames/sec, and zoom.
pub fn render_debug_overlay(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    sups: u64,
    fps: u32,
    zoom: f64,
) {
    // Offset by 1 row to leave the top-most line for sim state
    render_status_text(
        canvas,
        renderer,
        &format!("SUPS {} FPS {}", sups, fps),
        colors::BASE,
        colors::WHITE,
        1,
    );
    // Second line (offset further)
    render_status_text(
        canvas,
        renderer,
        &format!("zoom: {:.2}", zoom),
        colors::BASE,
        colors::WHITE,
        2,
    );
}
