use super::render_text_at;
use crate::buildings::SlotType;
use crate::colors;
use crate::render::tileset; // for make_multi_tile_rect
use crate::render::{SpriteSheetRenderer, TILE_PIXEL_WIDTH};
use sdl2::render::Canvas;
use sdl2::video::Window; // Use super to access function in parent module

/// Renders the pause menu.
pub fn render_pause_menu(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
) {
    let lines = vec![
        ("paused".to_string(), colors::WHITE),
        ("(esc) close menu".to_string(), colors::WHITE),
        ("(q) quit game".to_string(), colors::WHITE),
    ];
    draw_centered_window(canvas, renderer, &lines);
}

/// Renders the menu prompting the user to select a slot type (Ground/Orbital).
pub fn render_build_slot_type_menu(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
) {
    let lines = vec![
        ("build where?".to_string(), colors::WHITE),
        ("(g) ground".to_string(), colors::WHITE),
        ("(o) orbital".to_string(), colors::WHITE),
        ("(esc) close menu".to_string(), colors::WHITE),
    ];
    draw_centered_window(canvas, renderer, &lines);
}

/// Renders the menu prompting the user to select a building for the given slot type.
pub fn render_build_building_menu(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    slot_type: SlotType,
) {
    let mut lines: Vec<(String, sdl2::pixels::Color)> = Vec::new();
    lines.push((format!("build what? ({:?})", slot_type), colors::WHITE));

    if slot_type == SlotType::Orbital {
        lines.push(("(1) solar panel".to_string(), colors::WHITE));
    }
    if slot_type == SlotType::Ground {
        lines.push(("(2) mine".to_string(), colors::WHITE));
    }

    lines.push(("(esc) back".to_string(), colors::WHITE));

    draw_centered_window(canvas, renderer, &lines);
}

/// Renders a build error message.
pub fn render_build_error_menu(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    message: &str,
) {
    let lines = vec![
        ("build error:".to_string(), colors::RED),
        (message.to_string(), colors::RED),
        ("(any key) continue".to_string(), colors::WHITE),
    ];
    draw_centered_window(canvas, renderer, &lines);
}

// draw a centered window with given text/color lines
fn draw_centered_window(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    lines: &[(String, sdl2::pixels::Color)],
) {
    // compute window size in tiles
    let max_len = lines.iter().map(|(s, _)| s.len()).max().unwrap_or(0) as u8;
    let content_w = max_len;
    let content_h = lines.len() as u8;

    // window includes 1 tile padding around content, plus 1 tile for border
    let window_w = content_w + 4; // +2 for padding, +2 for border
    let window_h = content_h + 4; // +2 for padding, +2 for border

    // screen size in tiles
    let (px_w, px_h) = canvas.output_size().unwrap();
    let tiles_w = (px_w / TILE_PIXEL_WIDTH as u32) as u8;
    let tiles_h = (px_h / TILE_PIXEL_WIDTH as u32) as u8;

    let window_x = tiles_w.saturating_sub(window_w) / 2;
    let window_y = tiles_h.saturating_sub(window_h) / 2;

    // 1. Draw border rectangle (slightly larger)
    canvas.set_draw_color(colors::DGRAY); // Dark gray for border
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            window_x,
            window_y,
            window_w,
            window_h,
        ))
        .unwrap();

    // 2. Draw main background rectangle (slightly smaller, inside the border)
    let bg_x = window_x + 1;
    let bg_y = window_y + 1;
    let bg_w = window_w - 2;
    let bg_h = window_h - 2;
    canvas.set_draw_color(colors::BASE); // Use BASE color for window background
    canvas
        .fill_rect(tileset::make_multi_tile_rect(bg_x, bg_y, bg_w, bg_h))
        .unwrap();

    // render lines with padding
    // text starts 1 tile inside the background rect (so, window_x + 2, window_y + 2)
    let text_start_x = bg_x + 1;
    let text_start_y = bg_y + 1;

    for (i, (text, fg)) in lines.iter().enumerate() {
        render_text_at(
            canvas,
            renderer,
            text,
            colors::BASE, // Use BASE color for text background (transparent over window bg)
            *fg,
            text_start_x,
            text_start_y + i as u8,
        );
    }
}
