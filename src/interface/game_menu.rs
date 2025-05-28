use crate::colors;
use crate::render::SpriteSheetRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// renders the game menu.
pub fn render_game_menu(canvas: &mut Canvas<Window>, renderer: &SpriteSheetRenderer) {
    let lines = vec![
        ("game menu".to_string(), colors::WHITE), // updated title
        ("".to_string(), colors::BLACK),
        ("(esc) close menu".to_string(), colors::WHITE),
        ("(q) quit game".to_string(), colors::WHITE),
    ];
    // calls the draw_centered_window from the parent module (interface::mod.rs)
    super::draw_centered_window(canvas, renderer, &lines);
}
