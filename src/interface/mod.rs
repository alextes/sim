use crate::buildings::{BuildingType, EntityBuildings, GROUND_SLOTS, ORBITAL_SLOTS};
use crate::colors;
use crate::render::{tileset, SpriteSheetRenderer};
use crate::world::{EntityId, World};
use sdl2::render::Canvas;
use sdl2::video::Window;

pub mod build;

/// Helper to render text aligned at the given (x,y) tile coordinates.
/// This mirrors the implementation found in `render_status_text` but
/// allows specifying the x position instead of always right-aligning.
pub fn render_text_at(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
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

    renderer
        .texture
        .set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

    for (i, ch) in text.chars().enumerate() {
        let src = renderer.tileset.get_rect(ch);
        let dst = tileset::make_tile_rect(x_tile + i as u8, y_tile);
        canvas.copy(renderer.texture, Some(src), Some(dst)).ok();
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
    renderer: &mut SpriteSheetRenderer,
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
        renderer,
        &energy_text,
        colors::BASE,
        colors::YELLOW, // energy color
        1,              // x_tile = 1
        1,              // y_tile = 1
    );

    render_text_at(
        canvas,
        renderer,
        &metal_text,
        colors::BASE,
        colors::LGRAY, // metal color
        1,             // x_tile = 1
        2,             // y_tile = 2
    );

    // Current line offset for rendering subsequent UI elements
    let mut y_offset = 3; // Start below resources + margin

    // --- Bottom-left: selected entity name (if any) ---
    if let Some(id) = selected {
        if let Some(name) = world.get_entity_name(id) {
            // --- Build the information lines for the info window ---
            let mut info_lines: Vec<String> = Vec::new();

            // First line: selected entity name
            info_lines.push(format!("selected: {}", name));

            // Second line (optional): tracking status
            if track_mode {
                info_lines.push("tracking".to_string());
            }

            // Determine the size of the window (in tiles)
            let max_line_len = info_lines
                .iter()
                .map(|s| s.len())
                .max()
                .unwrap_or(0);

            // Add 2 tiles of horizontal padding (1 left, 1 right) and vertical padding (1 top, 1 bottom)
            let window_width_tiles: u8 = (max_line_len as u8).saturating_add(2);
            let window_height_tiles: u8 = (info_lines.len() as u8).saturating_add(2);

            // Position: bottom-left, but fully on-screen
            let x_tile: u8 = 1;
            let window_top_y: u8 = viewport_height_tiles
                .saturating_sub(window_height_tiles as u32 + 1) as u8;

            // Draw the window background (filled rectangle)
            canvas.set_draw_color(colors::BLACK);
            canvas
                .fill_rect(tileset::make_multi_tile_rect(
                    x_tile,
                    window_top_y,
                    window_width_tiles,
                    window_height_tiles,
                ))
                .unwrap();

            // Render each line of text inside the window
            for (i, line) in info_lines.iter().enumerate() {
                render_text_at(
                    canvas,
                    renderer,
                    line,
                    colors::BLACK,
                    colors::WHITE,
                    x_tile + 1,                      // inside left padding
                    window_top_y + 1 + i as u8,      // inside top padding + line offset
                );
            }

            // --- Building Slots --- Render below resources
            if let Some(buildings) = world.buildings.get(&id) {
                // Orbital Slots
                for i in 0..ORBITAL_SLOTS {
                    let slot_text = format_slot("O", i, buildings.orbital[i]);
                    render_text_at(
                        canvas,
                        renderer,
                        &slot_text,
                        colors::BASE,
                        colors::WHITE,
                        1, // x_tile = 1
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
                            renderer,
                            &slot_text,
                            colors::BASE,
                            colors::WHITE,
                            1, // x_tile = 1
                            y_offset,
                        );
                        y_offset += 1;
                    }
                }
            }
        }
    }
}
