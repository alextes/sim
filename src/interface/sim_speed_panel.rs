use crate::render::{tileset, SpriteSheetRenderer /*, TILE_PIXEL_WIDTH*/};
use sdl2::render::Canvas;
use sdl2::video::Window;

// Use shared constants and helpers from the parent module
use super::{
    render_text_at, PANEL_BACKGROUND_COLOR, PANEL_BORDER_COLOR, PANEL_TEXT_COLOR,
    SCREEN_EDGE_MARGIN,
};

pub fn render_sim_speed_panel(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    sim_speed: u32,
    paused: bool,
    top_y_anchor: u8,
    screen_tiles_w: u8,
) -> u8 {
    // Returns total panel height in tiles
    let speed_text = if paused {
        "SPEED: PAUSED".to_string()
    } else {
        format!("SPEED: {}x", sim_speed)
    };
    let text_len = speed_text.len() as u8;

    // Panel dimensions
    const PANEL_PADDING: u8 = 1;
    let panel_inner_w = text_len;
    let panel_total_w = panel_inner_w + PANEL_PADDING * 2;
    let panel_inner_h: u8 = 1; // Sim speed is a single line
    let panel_total_h = panel_inner_h + PANEL_PADDING * 2;

    // Panel position (top-right, using provided y_anchor and screen_width)
    let panel_x = screen_tiles_w.saturating_sub(panel_total_w + SCREEN_EDGE_MARGIN);
    let panel_y = top_y_anchor;

    // Text position (inside panel with padding)
    let text_x = panel_x + PANEL_PADDING;
    let text_y = panel_y + PANEL_PADDING;

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

    panel_total_h // Return the height of this panel
}
