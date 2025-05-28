use crate::buildings::SlotType;
use crate::colors;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::render::SpriteSheetRenderer;

/// render the menu prompting the user to select a slot type (Ground/Orbital).
pub fn render_build_slot_type_menu(canvas: &mut Canvas<Window>, renderer: &SpriteSheetRenderer) {
    let lines = vec![
        ("build where?".to_string(), colors::WHITE),
        ("".to_string(), colors::BLACK),
        ("(g) ground".to_string(), colors::WHITE),
        ("(o) orbital".to_string(), colors::WHITE),
        ("(esc) close menu".to_string(), colors::WHITE),
    ];
    super::draw_centered_window(canvas, renderer, &lines);
}

/// render the menu prompting the user to select a building for the given slot type.
pub fn render_build_building_menu(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    slot_type: SlotType,
) {
    let mut lines: Vec<(String, sdl2::pixels::Color)> = Vec::new();
    lines.push((format!("build what? ({:?})", slot_type), colors::WHITE));
    lines.push(("".to_string(), colors::BLACK));

    if slot_type == SlotType::Orbital {
        lines.push(("(1) solar panel".to_string(), colors::WHITE));
    }
    if slot_type == SlotType::Ground {
        lines.push(("(2) mine".to_string(), colors::WHITE));
    }

    if slot_type == SlotType::Orbital || slot_type == SlotType::Ground {
        lines.push(("".to_string(), colors::BLACK));
    }

    lines.push(("(esc) back".to_string(), colors::WHITE));

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
