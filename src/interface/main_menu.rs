use crate::{colors, interface::draw_colored_centered_window, render::SpriteSheetRenderer};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start * (1.0 - t) + end * t
}

fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    let r = lerp(c1.r as f32, c2.r as f32, t).round() as u8;
    let g = lerp(c1.g as f32, c2.g as f32, t).round() as u8;
    let b = lerp(c1.b as f32, c2.b as f32, t).round() as u8;
    Color::RGB(r, g, b)
}

pub fn render_main_menu(canvas: &mut Canvas<Window>, renderer: &SpriteSheetRenderer) {
    canvas.set_draw_color(colors::BLACK);
    canvas.clear();

    let title_art = vec![
        "█████ █████ █   █",
        "█       █   ██ ██",
        "█████   █   █ █ █",
        "    █   █   █   █",
        "█████ █████ █   █",
    ];

    let color1 = colors::BLUE;
    let color2 = colors::MAGENTA;
    let color3 = colors::LRED;

    let mut colored_title: Vec<Vec<(char, Color)>> = Vec::new();
    let width = title_art[0].len() as f32;

    for line in title_art {
        let mut colored_line = Vec::new();
        for (i, char) in line.chars().enumerate() {
            let t = i as f32 / width;
            let color = if t < 0.5 {
                lerp_color(color1, color2, t * 2.0)
            } else {
                lerp_color(color2, color3, (t - 0.5) * 2.0)
            };
            colored_line.push((char, color));
        }
        colored_title.push(colored_line);
    }

    let mut lines_with_colors: Vec<Vec<(char, Color)>> = colored_title;

    // Add empty lines for spacing
    lines_with_colors.push(Vec::new());
    lines_with_colors.push(Vec::new());
    lines_with_colors.push(Vec::new());

    // Menu options
    let play_text: Vec<(char, Color)> = "play".chars().map(|c| (c, colors::WHITE)).collect();
    lines_with_colors.push(play_text);

    draw_colored_centered_window(canvas, renderer, &lines_with_colors);
}
