use crate::colors;
use crate::world::{EntityId, World};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::SpriteSheetRenderer;

/// render the menu prompting the user to select a building.
pub fn render_build_menu(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    selected_id: Option<EntityId>,
) {
    let mut lines = vec![("build what?".to_string(), colors::WHITE)];
    lines.push(("".to_string(), colors::BLACK));

    let has_empty_slot = selected_id
        .and_then(|id| world.buildings.get(&id))
        .and_then(|b| b.find_first_empty_slot())
        .is_some();

    if has_empty_slot {
        lines.push(("(1) solar panel".to_string(), colors::WHITE));
        lines.push(("(2) mine".to_string(), colors::WHITE));
    } else {
        lines.push(("(no empty slots)".to_string(), colors::LGRAY));
    }

    lines.push(("".to_string(), colors::BLACK));
    lines.push(("(esc) close menu".to_string(), colors::WHITE));

    super::draw_centered_window(canvas, renderer, &lines);
}
