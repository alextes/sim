use crate::{colors, interface::draw_centered_window, render::SpriteSheetRenderer};
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn render_main_menu(canvas: &mut Canvas<Window>, renderer: &SpriteSheetRenderer) {
    let title = vec![
        " SSS  III  MMM ".to_string(),
        "S      I  M M M".to_string(),
        " SSS   I  M   M".to_string(),
        "    S  I  M   M".to_string(),
        " SSS  III M   M".to_string(),
    ];

    let mut lines_with_colors = title
        .into_iter()
        .map(|s| (s, colors::WHITE))
        .collect::<Vec<_>>();

    // Add empty lines for spacing
    lines_with_colors.push(("".to_string(), colors::BLACK));
    lines_with_colors.push(("".to_string(), colors::BLACK));
    lines_with_colors.push(("".to_string(), colors::BLACK));

    // Menu options
    lines_with_colors.push(("play".to_string(), colors::WHITE));

    draw_centered_window(canvas, renderer, &lines_with_colors);
}
