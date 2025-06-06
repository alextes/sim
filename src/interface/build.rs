use crate::colors;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::SpriteSheetRenderer;

/// render the menu prompting the user to select a building.
pub fn render_build_menu(canvas: &mut Canvas<Window>, renderer: &SpriteSheetRenderer) {
    let lines = vec![
        ("build what?".to_string(), colors::WHITE),
        ("".to_string(), colors::BLACK),
        ("(1) solar panel".to_string(), colors::WHITE),
        ("(2) mine".to_string(), colors::WHITE),
        ("".to_string(), colors::BLACK),
        ("(esc) close menu".to_string(), colors::WHITE),
    ];
    super::draw_centered_window(canvas, renderer, &lines);
}

/// render a build error message.
pub fn render_build_error_menu(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    message: &str,
) {
    let lines = vec![
        ("build error:".to_string(), colors::RED),
        ("".to_string(), colors::BLACK),
        (message.to_string(), colors::RED),
        ("".to_string(), colors::BLACK),
        ("(any key) continue".to_string(), colors::WHITE),
    ];
    super::draw_centered_window(canvas, renderer, &lines);
}
