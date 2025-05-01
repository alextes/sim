use crate::colors;
use crate::render::render_status_text;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

/// Renders debug overlay: load history, simulation updates/sec, frames/sec, and zoom.
pub fn render_debug_overlay(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    load_history: &str,
    sups: u32,
    fps: u32,
    zoom: f64,
) {
    // First line: load history, SUPS, and FPS
    render_status_text(
        canvas,
        tiles_texture,
        &format!("LOAD {} SUPS {} FPS {}", load_history, sups, fps),
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
