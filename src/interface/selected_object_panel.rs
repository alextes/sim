use crate::buildings::{BuildingType, EntityBuildings, GROUND_SLOTS, ORBITAL_SLOTS};
use crate::colors;
use crate::render::{tileset, SpriteSheetRenderer};
use crate::world::{EntityId, World};
use sdl2::render::Canvas;
use sdl2::video::Window;

// Use shared constants and helpers from the parent module
use super::{render_text_at, PANEL_BACKGROUND_COLOR, PANEL_BORDER_COLOR, PANEL_TEXT_COLOR};

// Constants specific to this panel
const PANEL_WIDTH_TILES: u8 = 25; // Fixed width
const PANEL_MAX_HEIGHT_TILES: u8 = 12; // Max height, includes padding and border

// Helper function to format slot display
fn format_slot(prefix: &str, index: usize, slot: Option<BuildingType>) -> String {
    let building_name = match slot {
        Some(bt) => EntityBuildings::building_name(bt),
        None => "empty",
    };
    format!("{}{}: {}", prefix, index + 1, building_name)
}

pub fn render_selected_object_panel(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
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
            let panel_total_height_tiles =
                (panel_inner_height_tiles + 2).min(PANEL_MAX_HEIGHT_TILES); // +2 for padding

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
