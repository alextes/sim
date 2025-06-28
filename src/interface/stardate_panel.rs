use crate::render::{tileset, SpriteSheetRenderer};
use sdl2::render::Canvas;
use sdl2::video::Window;

// use shared constants and helpers from the parent module
use super::{render_text_at, PANEL_BACKGROUND_COLOR, PANEL_BORDER_COLOR, PANEL_TEXT_COLOR};

pub fn render_stardate_panel(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    total_sim_ticks: u64,
    top_y_anchor: u8,
    _screen_tiles_w: u8,
) -> u8 {
    // returns total panel height in tiles
    let stardate = total_sim_ticks as f64 / 100.0; // 100 ticks per second
    let date_text = format!("DATE: {stardate:.2}");
    let text_len = date_text.len() as u8;

    // panel dimensions
    const PANEL_PADDING: u8 = 1;
    let panel_inner_w = text_len;
    let panel_total_w = panel_inner_w + PANEL_PADDING * 2;
    let panel_inner_h: u8 = 1;
    let panel_total_h = panel_inner_h + PANEL_PADDING * 2;

    // panel position (top-left, using provided y_anchor and screen_width)
    let panel_x: u8 = 1;
    let panel_y = top_y_anchor;

    // text position (inside panel with padding)
    let text_x = panel_x + PANEL_PADDING;
    let text_y = panel_y + PANEL_PADDING;

    // draw panel background
    canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // draw panel border
    canvas.set_draw_color(PANEL_BORDER_COLOR);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // render text
    render_text_at(
        canvas,
        renderer,
        &date_text,
        PANEL_BACKGROUND_COLOR,
        PANEL_TEXT_COLOR,
        text_x,
        text_y,
    );

    panel_total_h // return the height of this panel
}
