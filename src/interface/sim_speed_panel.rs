use crate::render::{tileset, SpriteSheetRenderer, TILE_PIXEL_WIDTH};
use sdl2::render::Canvas;
use sdl2::video::Window;

// Use shared constants and helpers from the parent module
use super::{render_text_at, PANEL_BACKGROUND_COLOR, PANEL_BORDER_COLOR, PANEL_TEXT_COLOR};

pub fn render_sim_speed_panel(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    sim_speed: u32,
    paused: bool,
) {
    let speed_text = if paused {
        "SPEED: PAUSED".to_string()
    } else {
        format!("SPEED: {}x", sim_speed)
    };
    let text_len = speed_text.len() as u8;

    // Position in top-right
    // Calculate screen width in tiles
    let (screen_px_w, _) = canvas.output_size().unwrap_or((0, 0));
    let screen_tiles_w = (screen_px_w / TILE_PIXEL_WIDTH as u32) as u8;

    // Panel dimensions
    let panel_padding: u8 = 1;
    let panel_inner_w = text_len;
    let panel_total_w = panel_inner_w + panel_padding * 2;
    let panel_inner_h: u8 = 1;
    let panel_total_h = panel_inner_h + panel_padding * 2;

    // Panel position (top-right)
    let panel_x = screen_tiles_w.saturating_sub(panel_total_w + 1); // +1 for margin from edge
    let panel_y: u8 = 1; // 1 tile from top

    // Text position (inside panel with padding)
    let text_x = panel_x + panel_padding;
    let text_y = panel_y + panel_padding;

    // Draw panel background
    canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // Draw panel border
    canvas.set_draw_color(PANEL_BORDER_COLOR);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // Render text
    render_text_at(
        canvas,
        renderer,
        &speed_text,
        PANEL_BACKGROUND_COLOR, // Text background (same as panel)
        PANEL_TEXT_COLOR,       // Text foreground
        text_x,
        text_y,
    );
}
