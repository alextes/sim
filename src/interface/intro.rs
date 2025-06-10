use crate::{
    colors,
    interface::draw_centered_window,
    render::{SpriteSheetRenderer, TILE_PIXEL_WIDTH},
};
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn render_intro_screen(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    progress: f64,
) {
    let title = vec![
        " SSS  III  MMM ".to_string(),
        "S     I   M M M".to_string(),
        " SSS  I   M   M".to_string(),
        "    S I   M   M".to_string(),
        " SSS  III M   M".to_string(),
    ];

    let mut lines_with_colors = title
        .into_iter()
        .map(|s| (s, colors::WHITE))
        .collect::<Vec<_>>();

    // Add empty lines for spacing
    lines_with_colors.push(("".to_string(), colors::BLACK));
    lines_with_colors.push(("".to_string(), colors::BLACK));

    // Loading bar
    let (screen_w, _) = canvas.output_size().unwrap();
    let screen_w_tiles = (screen_w / TILE_PIXEL_WIDTH as u32) as usize;
    let bar_width = screen_w_tiles.saturating_sub(20).max(10);
    let filled_width = (bar_width as f64 * progress).round() as usize;
    let empty_width = bar_width - filled_width;

    let loading_bar = format!(
        "loading... [{}{}]",
        "#".repeat(filled_width),
        "-".repeat(empty_width)
    );
    lines_with_colors.push((loading_bar, colors::LGRAY));

    draw_centered_window(canvas, renderer, &lines_with_colors);
}