use crate::colors;
use crate::render::SpriteSheetRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SIM_ASCII_ART: &str = r#"
   ____  _  __  __
  / __/ (_)/ / / /
 / _/  / / / /_/ /
/___/ /_/  \____/

"#;

/// renders the game menu.
pub fn render_game_menu(canvas: &mut Canvas<Window>, renderer: &SpriteSheetRenderer) {
    let mut lines = Vec::new();

    for line in SIM_ASCII_ART.lines() {
        lines.push((line.to_string(), colors::WHITE));
    }

    lines.push(("".to_string(), colors::BLACK));
    lines.push(("(p) play".to_string(), colors::WHITE));
    lines.push(("(q) quit".to_string(), colors::WHITE));

    // calls the draw_centered_window from the parent module (interface::mod.rs)
    super::draw_centered_window(canvas, renderer, &lines);
}
