use crate::colors;
use crate::render::SpriteSheetRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Renders the pause menu.
pub fn render_pause_menu(canvas: &mut Canvas<Window>, renderer: &mut SpriteSheetRenderer) {
    let lines = vec![
        ("paused".to_string(), colors::WHITE),
        ("".to_string(), colors::BLACK),
        ("(esc) close menu".to_string(), colors::WHITE),
        ("(q) quit game".to_string(), colors::WHITE),
    ];
    // Calls the draw_centered_window from the parent module (interface::mod.rs)
    super::draw_centered_window(canvas, renderer, &lines);
}
