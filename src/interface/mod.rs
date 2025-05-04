use crate::buildings::{BuildingType, EntityBuildings, GROUND_SLOTS, ORBITAL_SLOTS};
use crate::colors;
use crate::render::tileset;
use crate::world::{EntityId, World};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

pub mod build;

/// Helper to render text aligned at the given (x,y) tile coordinates.
/// This mirrors the implementation found in `render_status_text` but
/// allows specifying the x position instead of always right-aligning.
pub fn render_text_at(
    // Make this public
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

// Helper function to format slot display
fn format_slot(prefix: &str, index: usize, slot: Option<BuildingType>) -> String {
    let building_name = match slot {
        Some(bt) => EntityBuildings::building_name(bt),
        None => "empty",
    };
    format!("{}{}: {}", prefix, index + 1, building_name)
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
    let (energy_rate, metal_rate) = world.resources.calculate_rates(&world.buildings);

    let energy = world.resources.energy;
    let metal = world.resources.metal;

    let energy_text = format!("NRG: {:.1} (+{:.1}/s)", energy, energy_rate);
    let metal_text = format!("MTL: {:.1} (+{:.1}/s)", metal, metal_rate);

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

    // Current line offset for rendering subsequent UI elements
    let mut y_offset = 2; // Start below resources

    // --- Bottom-left: selected entity name (if any) ---
    if let Some(id) = selected {
        if let Some(name) = world.get_entity_name(id) {
            // --- Selection & Tracking --- Start from bottom and go up
            let bottom_y = viewport_height_tiles.saturating_sub(1) as u8;

            let selected_text = format!("selected: {}", name);
            render_text_at(
                canvas,
                tiles_texture,
                &selected_text,
                colors::BASE,
                colors::WHITE,
                0,
                bottom_y,
            );

            // Render tracking status above selection
            if track_mode && bottom_y > 0 {
                let tracking_text = "tracking";
                render_text_at(
                    canvas,
                    tiles_texture,
                    tracking_text,
                    colors::BASE,
                    colors::WHITE,
                    0,
                    bottom_y - 1,
                );
            }

            // --- Building Slots --- Render below resources
            if let Some(buildings) = world.buildings.get(&id) {
                // Orbital Slots
                for i in 0..ORBITAL_SLOTS {
                    let slot_text = format_slot("O", i, buildings.orbital[i]);
                    render_text_at(
                        canvas,
                        tiles_texture,
                        &slot_text,
                        colors::BASE,
                        colors::WHITE,
                        0,
                        y_offset,
                    );
                    y_offset += 1;
                }

                // Ground Slots (if they exist)
                if buildings.has_ground_slots {
                    for i in 0..GROUND_SLOTS {
                        let slot_text = format_slot("G", i, buildings.ground[i]);
                        render_text_at(
                            canvas,
                            tiles_texture,
                            &slot_text,
                            colors::BASE,
                            colors::WHITE,
                            0,
                            y_offset,
                        );
                        y_offset += 1;
                    }
                }
            }
        }
    }
}
