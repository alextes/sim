use crate::render::{tileset, SpriteSheetRenderer};
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::{render_text_at, PANEL_BACKGROUND_COLOR, PANEL_BORDER_COLOR, PANEL_TEXT_COLOR};

pub fn render_credits_panel(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    credits: f64,
    top_y_anchor: u8,
) -> u8 {
    let credits_text = format!("credits: {credits:.0}");
    let text_len = credits_text.len() as u8;

    const PANEL_PADDING: u8 = 1;
    let panel_inner_w = text_len;
    let panel_total_w = panel_inner_w + PANEL_PADDING * 2;
    let panel_inner_h: u8 = 1;
    let panel_total_h = panel_inner_h + PANEL_PADDING * 2;

    let panel_x: u8 = 1;
    let panel_y = top_y_anchor;

    let text_x = panel_x + PANEL_PADDING;
    let text_y = panel_y + PANEL_PADDING;

    canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    canvas.set_draw_color(PANEL_BORDER_COLOR);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    render_text_at(
        canvas,
        renderer,
        &credits_text,
        PANEL_BACKGROUND_COLOR,
        PANEL_TEXT_COLOR,
        text_x,
        text_y,
    );

    panel_total_h
}
