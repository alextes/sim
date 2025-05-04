use crate::colors;
use crate::render::tileset;
use crate::world::EntityId;
use crate::world::World;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

/// Helper to render text aligned at the given (x,y) tile coordinates.
/// This mirrors the implementation found in `render_status_text` but
/// allows specifying the x position instead of always right-aligning.
fn render_text_at(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    text: &str,
    background_color: sdl2::pixels::Color,
    foreground_color: sdl2::pixels::Color,
    x_tile: u8,
    y_tile: u8,
) {
    // draw background rectangle behind the text
    canvas.set_draw_color(background_color);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            x_tile,
            y_tile,
            text.len() as u8,
            1,
        ))
        .unwrap();

    tiles_texture.set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

    for (i, ch) in text.chars().enumerate() {
        let src = tileset::rect_from_char(ch);
        let dst = tileset::make_tile_rect(x_tile + i as u8, y_tile);
        canvas.copy(tiles_texture, Some(src), Some(dst)).ok();
    }
}

/// Render resource counters (top-left) and current selection (bottom-left).
///
/// This function should be called once per frame after the world has
/// been updated and the main viewport rendered.
pub fn render_interface(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    world: &World,
    selected: Option<EntityId>,
    track_mode: bool,
    viewport_height_tiles: u32,
) {
    // --- Top-left: resources (two lines) ---
    let energy = world.resources.energy;
    let metal = world.resources.metal;

    let energy_text = format!("NRG: {:.0}", energy);
    let metal_text = format!("MTL: {:.0}", metal);

    render_text_at(
        canvas,
        tiles_texture,
        &energy_text,
        colors::BASE,
        colors::YELLOW, // energy color
        0,
        0,
    );

    render_text_at(
        canvas,
        tiles_texture,
        &metal_text,
        colors::BASE,
        colors::LGRAY, // metal color
        0,
        1,
    );

    // --- Bottom-left: selected entity name (if any) ---
    if let Some(id) = selected {
        if let Some(name) = world.get_entity_name(id) {
            let y_selected = viewport_height_tiles.saturating_sub(1) as u8;
            let selected_text = format!("selected: {}", name);
            render_text_at(
                canvas,
                tiles_texture,
                &selected_text,
                colors::BASE,
                colors::WHITE,
                0,
                y_selected,
            );

            if track_mode && y_selected > 0 {
                let tracking_text = "tracking";
                render_text_at(
                    canvas,
                    tiles_texture,
                    tracking_text,
                    colors::BASE,
                    colors::WHITE,
                    0,
                    y_selected - 1,
                );
            }
        }
    }
}
