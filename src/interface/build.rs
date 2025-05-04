use super::render_text_at;
use crate::buildings::SlotType;
use crate::colors;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window; // Use super to access function in parent module

/// Renders the menu prompting the user to select a slot type (Ground/Orbital).
pub fn render_build_slot_type_menu(canvas: &mut Canvas<Window>, tiles_texture: &mut Texture<'_>) {
    // Simple menu for now, could be made fancier
    let y_start = 25;
    render_text_at(
        canvas,
        tiles_texture,
        "Build Where?",
        colors::BASE,
        colors::WHITE,
        25,
        y_start,
    );
    render_text_at(
        canvas,
        tiles_texture,
        "(G) Ground",
        colors::BASE,
        colors::WHITE,
        25,
        y_start + 1,
    );
    render_text_at(
        canvas,
        tiles_texture,
        "(O) Orbital",
        colors::BASE,
        colors::WHITE,
        25,
        y_start + 2,
    );
    render_text_at(
        canvas,
        tiles_texture,
        "(Esc) Cancel",
        colors::BASE,
        colors::WHITE,
        25,
        y_start + 4,
    );
}

/// Renders the menu prompting the user to select a building for the given slot type.
pub fn render_build_building_menu(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    slot_type: SlotType,
) {
    let y_start = 25;
    let title = format!("Build What? ({:?})", slot_type);
    render_text_at(
        canvas,
        tiles_texture,
        &title,
        colors::BASE,
        colors::WHITE,
        25,
        y_start,
    );

    // Show only buildings compatible with the selected slot type
    let mut y_current = y_start + 1;
    if slot_type == SlotType::Orbital {
        render_text_at(
            canvas,
            tiles_texture,
            "(1) Solar Panel",
            colors::BASE,
            colors::WHITE,
            25,
            y_current,
        );
        y_current += 1;
    }
    if slot_type == SlotType::Ground {
        render_text_at(
            canvas,
            tiles_texture,
            "(2) Mine",
            colors::BASE,
            colors::WHITE,
            25,
            y_current,
        );
        y_current += 1;
    }

    render_text_at(
        canvas,
        tiles_texture,
        "(Esc) Back",
        colors::BASE,
        colors::WHITE,
        25,
        y_current + 1,
    );
}

/// Renders a build error message.
pub fn render_build_error_menu(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    message: &str,
) {
    let y_start = 25;
    render_text_at(
        canvas,
        tiles_texture,
        "Build Error:",
        colors::BASE,
        colors::RED,
        25,
        y_start,
    );
    render_text_at(
        canvas,
        tiles_texture,
        message,
        colors::BASE,
        colors::RED,
        25,
        y_start + 1,
    );
    render_text_at(
        canvas,
        tiles_texture,
        "(Any key) Continue",
        colors::BASE,
        colors::WHITE,
        25,
        y_start + 3,
    );
}
