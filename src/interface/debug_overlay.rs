// use crate::colors; // Unused if PANEL_TEXT_COLOR is sufficient from super
use crate::render::{/*render_status_text,*/ tileset, SpriteSheetRenderer, /*, TILE_PIXEL_WIDTH*/}; // tileset is from here
use sdl2::render::Canvas;
use sdl2::video::Window;

// Use shared constants and helpers from the parent module (interface/mod.rs)
use super::{
    render_text_at, PANEL_BACKGROUND_COLOR, PANEL_BORDER_COLOR, PANEL_TEXT_COLOR,
    SCREEN_EDGE_MARGIN,
};

/// Renders debug overlay in a panel: simulation updates/sec, frames/sec, and zoom.
pub fn render_debug_overlay(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    sups: u64,
    fps: u32,
    zoom: f64,
    top_y_anchor: u8,
    screen_tiles_w: u8,
) -> u8 {
    // Returns total panel height in tiles
    let line1 = format!("SUPS {sups} FPS {fps}");
    let line2 = format!("zoom: {zoom:.2}");
    let lines_content: [&str; 2] = [&line1, &line2];

    let max_text_len = lines_content.iter().map(|s| s.len()).max().unwrap_or(0) as u8;
    let num_text_lines = lines_content.len() as u8;

    // Panel dimensions
    const PANEL_PADDING: u8 = 1;
    let panel_inner_w = max_text_len;
    let panel_total_w = panel_inner_w + PANEL_PADDING * 2;
    let panel_inner_h = num_text_lines;
    let panel_total_h = panel_inner_h + PANEL_PADDING * 2;

    // Panel position (top-right, using provided y_anchor and screen_width)
    let panel_x = screen_tiles_w.saturating_sub(panel_total_w + SCREEN_EDGE_MARGIN);
    let panel_y = top_y_anchor;

    // Text start position (inside panel with padding)
    let text_start_x = panel_x + PANEL_PADDING;
    let text_start_y = panel_y + PANEL_PADDING;

    // 1. Draw panel background
    canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // 2. Draw panel border
    canvas.set_draw_color(PANEL_BORDER_COLOR);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // 3. Render text lines
    render_text_at(
        canvas,
        renderer,
        &line1,
        PANEL_BACKGROUND_COLOR, // Text background (same as panel)
        PANEL_TEXT_COLOR,       // Text foreground
        text_start_x,
        text_start_y, // First line
    );
    render_text_at(
        canvas,
        renderer,
        &line2,
        PANEL_BACKGROUND_COLOR,
        PANEL_TEXT_COLOR,
        text_start_x,
        text_start_y + 1, // Second line
    );

    panel_total_h // Return the height of this panel
}
