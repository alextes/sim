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

// Constants for the selected object panel
const PANEL_BORDER_COLOR: sdl2::pixels::Color = colors::GRAY;
const PANEL_BACKGROUND_COLOR: sdl2::pixels::Color = colors::BLACK;
const PANEL_TEXT_COLOR: sdl2::pixels::Color = colors::WHITE;
const PANEL_WIDTH_TILES: u8 = 25; // Fixed width
const PANEL_MAX_HEIGHT_TILES: u8 = 12; // Max height, includes padding and border

fn render_selected_object_panel(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    world: &World,
    selected: Option<EntityId>,
    track_mode: bool,
    viewport_height_tiles: u32,
) {
    if let Some(id) = selected {
        if let Some(name) = world.get_entity_name(id) {
            let mut lines: Vec<(String, sdl2::pixels::Color)> = Vec::new();
            if track_mode {
                lines.push(("tracking".to_string(), PANEL_TEXT_COLOR));
            }
            lines.push((format!("selected: {}", name), PANEL_TEXT_COLOR));

            if let Some(buildings) = world.buildings.get(&id) {
                for i in 0..ORBITAL_SLOTS {
                    lines.push((
                        format_slot("Orb", i, buildings.orbital[i]),
                        colors::CYAN, // Example color for orbital slots
                    ));
                }
                if buildings.has_ground_slots {
                    for i in 0..GROUND_SLOTS {
                        lines.push((
                            format_slot("Gnd", i, buildings.ground[i]),
                            colors::ORANGE, // Example color for ground slots
                        ));
                    }
                }
            }

            // Calculate actual panel height based on content, but capped
            let content_height_tiles = lines.len() as u8;
            let panel_inner_height_tiles = content_height_tiles;
            let panel_total_height_tiles = (panel_inner_height_tiles + 2).min(PANEL_MAX_HEIGHT_TILES); // +2 for padding

            // Ensure panel_width_tiles accommodates the longest line + padding
            let max_line_len = lines.iter().map(|(l, _)| l.len()).max().unwrap_or(0) as u8;
            let required_width_tiles = max_line_len + 2; // +2 for padding
            let panel_total_width_tiles = PANEL_WIDTH_TILES.max(required_width_tiles);


            let window_x: u8 = 1;
            let bottom_margin_tiles: u32 = 1;
            let window_y: u8 = viewport_height_tiles
                .saturating_sub(panel_total_height_tiles as u32 + bottom_margin_tiles)
                as u8;

            // Draw background
            canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
            canvas
                .fill_rect(tileset::make_multi_tile_rect(
                    window_x,
                    window_y,
                    panel_total_width_tiles,
                    panel_total_height_tiles,
                ))
                .unwrap();

            // Draw border
            canvas.set_draw_color(PANEL_BORDER_COLOR);
            canvas
                .draw_rect(tileset::make_multi_tile_rect(
                    window_x,
                    window_y,
                    panel_total_width_tiles,
                    panel_total_height_tiles,
                ))
                .unwrap();

            // Render each line inside, starting at (window_x + 1, window_y + 1)
            // And only render as many lines as can fit
            let max_lines_to_render = (panel_total_height_tiles - 2) as usize; // -2 for padding
            for (i, (line, color)) in lines.iter().take(max_lines_to_render).enumerate() {
                render_text_at(
                    canvas,
                    renderer,
                    line,
                    PANEL_BACKGROUND_COLOR, // Text background is panel background
                    *color,                 // Text foreground color
                    window_x + 1,
                    window_y + 1 + i as u8,
                );
            }
        }
    }
}

fn render_resources_panel(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    world: &World,
) {
    // --- top-left: resources (two lines) ---
    let (energy_rate, metal_rate) = world.resources.calculate_rates(&world.buildings);

    let energy = world.resources.energy;
    let metal = world.resources.metal;

    let energy_text = format!("nrg: {:.1} (+{:.1}/s)", energy, energy_rate);
    let metal_text = format!("mtl: {:.1} (+{:.1}/s)", metal, metal_rate);

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
    render_resources_panel(canvas, renderer, world);
    render_selected_object_panel(
        canvas,
        renderer,
        world,
        selected,
        track_mode,
        viewport_height_tiles,
    );
}
