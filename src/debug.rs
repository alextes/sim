use crate::colors;
use crate::render::render_status_text;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

/// Renders debug overlay: simulation updates/sec, frames/sec, and zoom.
pub fn render_debug_overlay(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    sups: u64,
    fps: u32,
    zoom: f64,
) {
    // First line: SUPS and FPS
    render_status_text(
        canvas,
        tiles_texture,
        &format!("SUPS {} FPS {}", sups, fps),
        colors::BASE,
        colors::WHITE,
        0,
    );
    // Second line: zoom
    render_status_text(
        canvas,
        tiles_texture,
        &format!("zoom: {:.2}", zoom),
        colors::BASE,
        colors::WHITE,
        1,
    );
}
