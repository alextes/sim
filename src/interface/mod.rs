use crate::buildings::{BuildingType, EntityBuildings, GROUND_SLOTS, ORBITAL_SLOTS};
use crate::colors;
use crate::render::{tileset, SpriteSheetRenderer};
use crate::world::{EntityId, World};
use sdl2::render::Canvas;
use sdl2::video::Window;
use crate::{ControlState, GameState, SlotType};
use std::sync::{Arc, Mutex};

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
    controls: &ControlState,
    game_state: Arc<Mutex<GameState>>,
) {
    // --- top-left: resources (two lines) ---
    let (energy_rate, metal_rate) = world.resources.calculate_rates(&world.buildings);

    let energy = world.resources.energy;
    let metal = world.resources.metal;

    let energy_text = format!("nrg: {:.1} (+{:.1}/s)", energy, energy_rate);
    let metal_text = format!("mtl: {:.1} (+{:.1}/s)", metal, metal_rate);

    // Determine window dimensions for resources
    let resource_text_max_len = energy_text.len().max(metal_text.len()) as u8;
    let resource_window_width_tiles = resource_text_max_len + 2; // 1 tile padding on each side
    let resource_window_height_tiles = 4; // 2 lines of text + 1 tile padding top/bottom

    // Draw background for resources
    canvas.set_draw_color(colors::BLACK);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            0, // x_tile = 0
            0, // y_tile = 0
            resource_window_width_tiles,
            resource_window_height_tiles,
        ))
        .unwrap();

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

    // --- simulation state overlay (upper-right) ---
    let sim_state_text = if controls.paused {
        "PAUSED".to_string()
    } else {
        format!("{}x", controls.sim_speed)
    };

    // Determine window dimensions for sim speed
    let sim_speed_text_len = sim_state_text.len() as u8;
    let sim_speed_window_width_tiles = sim_speed_text_len + 2; // 1 tile padding on each side
    let sim_speed_window_height_tiles = 3; // 1 line of text + 1 tile padding top/bottom

    // Position: upper-right
    // Assuming viewport_width_tiles can be derived or is fixed for now
    // Let's use a placeholder for viewport_width_tiles and adjust if needed.
    // A common approach is to get it from canvas.output_size()
    let (screen_width_pixels, _) = canvas.output_size().unwrap_or((800,600)); // Default if fails
    let viewport_width_tiles = screen_width_pixels / crate::render::TILE_PIXEL_WIDTH as u32;

    let sim_speed_window_x = viewport_width_tiles.saturating_sub(sim_speed_window_width_tiles as u32) as u8;
    let sim_speed_window_y = 0; // y_tile = 0

    // Draw background for sim speed
    canvas.set_draw_color(colors::BLACK);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            sim_speed_window_x,
            sim_speed_window_y,
            sim_speed_window_width_tiles,
            sim_speed_window_height_tiles,
        ))
        .unwrap();

    render_text_at(
        canvas,
        renderer,
        &sim_state_text,
        colors::BLACK, // Background for text itself (transparent due to fill_rect above)
        colors::WHITE,
        sim_speed_window_x + 1, // x_tile, with padding
        sim_speed_window_y + 1, // y_tile, with padding
    );

    // --- Build Menus (if active) ---
    match &*game_state.lock().unwrap() {
        GameState::BuildMenuSelectingSlotType => {
            build::render_build_slot_type_menu(
                canvas,
                renderer,
            );
        }
        GameState::BuildMenuSelectingBuilding { slot_type } => {
            build::render_build_building_menu(
                canvas,
                renderer,
                *slot_type,
            );
        }
        GameState::BuildMenuError { message } => {
            build::render_build_error_menu(
                canvas,
                renderer,
                message,
            );
        }
        _ => {} // GameState::Playing or other states handled elsewhere or not needing specific menu UI here
    }

    // --- selection panel ---

    // --- bottom-left: selected entity name (if any) ---
    if let Some(id) = selected {
        if let Some(name) = world.get_entity_name(id) {
            // collect all lines we want to show inside the window
            let mut lines: Vec<String> = Vec::new();
            if track_mode {
                lines.push("tracking".to_string());
            }
            lines.push(format!("selected: {}", name));

            // gather building slot information
            if let Some(buildings) = world.buildings.get(&id) {
                for i in 0..ORBITAL_SLOTS {
                    lines.push(format_slot("O", i, buildings.orbital[i]));
                }
                if buildings.has_ground_slots {
                    for i in 0..GROUND_SLOTS {
                        lines.push(format_slot("G", i, buildings.ground[i]));
                    }
                }
            }

            // determine window dimensions in tiles
            let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u8;
            let window_width_tiles = max_line_len + 2; // 1 tile padding on each side
            let window_height_tiles = lines.len() as u8 + 2; // 1 tile padding top + bottom

            // position: lower-left, 1 tile above bottom of screen
            let window_x: u8 = 1;
            let bottom_margin_tiles: u32 = 1;
            let window_y: u8 = viewport_height_tiles
                .saturating_sub(window_height_tiles as u32 + bottom_margin_tiles)
                as u8;

            // draw the window background once
            canvas.set_draw_color(colors::BLACK);
            canvas
                .fill_rect(tileset::make_multi_tile_rect(
                    window_x,
                    window_y,
                    window_width_tiles,
                    window_height_tiles,
                ))
                .unwrap();

            // render each line inside, starting at (window_x + 1, window_y + 1)
            for (i, line) in lines.iter().enumerate() {
                render_text_at(
                    canvas,
                    renderer,
                    line,
                    colors::BLACK,
                    colors::WHITE,
                    window_x + 1,
                    window_y + 1 + i as u8,
                );
            }
        }
    }
}
