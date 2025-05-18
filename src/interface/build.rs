use super::render_text_at;
use crate::buildings::SlotType;
use crate::colors;
use crate::render::SpriteSheetRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window; // Use super to access function in parent module

/// Renders the menu prompting the user to select a slot type (Ground/Orbital).
pub fn render_build_slot_type_menu(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
) {
    // Simple menu for now, could be made fancier
    let x_start = 1;
    let y_start = 10;
    render_text_at(
        canvas,
        renderer,
        "build where?",
        colors::BASE,
        colors::WHITE,
        x_start,
        y_start,
    );
    render_text_at(
        canvas,
        renderer,
        "(g) ground",
        colors::BASE,
        colors::WHITE,
        x_start,
        y_start + 1,
    );
    render_text_at(
        canvas,
        renderer,
        "(o) orbital",
        colors::BASE,
        colors::WHITE,
        x_start,
        y_start + 2,
    );
    render_text_at(
        canvas,
        renderer,
        "(esc) cancel",
        colors::BASE,
        colors::WHITE,
        x_start,
        y_start + 4,
    );
}

/// Renders the menu prompting the user to select a building for the given slot type.
pub fn render_build_building_menu(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    slot_type: SlotType,
) {
    let x_start = 1;
    let y_start = 10;
    let title = format!("build what? ({:?})", slot_type);
    render_text_at(
        canvas,
        renderer,
        &title,
        colors::BASE,
        colors::WHITE,
        x_start,
        y_start,
    );

    // Show only buildings compatible with the selected slot type
    let mut y_current = y_start + 1;
    if slot_type == SlotType::Orbital {
        render_text_at(
            canvas,
            renderer,
            "(1) solar panel",
            colors::BASE,
            colors::WHITE,
            x_start,
            y_current,
        );
        y_current += 1;
    }
    if slot_type == SlotType::Ground {
        render_text_at(
            canvas,
            renderer,
            "(2) mine",
            colors::BASE,
            colors::WHITE,
            x_start,
            y_current,
        );
        y_current += 1;
    }

    render_text_at(
        canvas,
        renderer,
        "(esc) back",
        colors::BASE,
        colors::WHITE,
        x_start,
        y_current + 1,
    );
}

/// Renders a build error message.
pub fn render_build_error_menu(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    message: &str,
) {
    let x_start = 1;
    let y_start = 10;
    render_text_at(
        canvas,
        renderer,
        "build error:",
        colors::BASE,
        colors::RED,
        x_start,
        y_start,
    );
    render_text_at(
        canvas,
        renderer,
        message,
        colors::BASE,
        colors::RED,
        x_start,
        y_start + 1,
    );
    render_text_at(
        canvas,
        renderer,
        "(any key) continue",
        colors::BASE,
        colors::WHITE,
        x_start,
        y_start + 3,
    );
}
