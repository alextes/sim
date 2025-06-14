use crate::buildings::{BuildingType, EntityBuildings};
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
fn format_slot(index: usize, slot: Option<BuildingType>) -> String {
    let building_name = match slot {
        Some(bt) => EntityBuildings::building_name(bt),
        None => "empty",
    };
    format!("slot {}: {}", index + 1, building_name)
}

pub fn render_selected_object_panel(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    selection: &[EntityId],
    track_mode: bool,
    viewport_height_tiles: u32,
) {
    if selection.is_empty() {
        return;
    }

    let mut lines: Vec<(String, sdl2::pixels::Color)> = Vec::new();

    if track_mode {
        lines.push(("tracking".to_string(), PANEL_TEXT_COLOR));
    }

    if selection.len() == 1 {
        let id = selection[0];
        if let Some(name) = world.get_entity_name(id) {
            lines.push((format!("selected: {}", name), PANEL_TEXT_COLOR));

            if let Some(buildings) = world.buildings.get(&id) {
                if !buildings.slots.is_empty() {
                    lines.push(("buildings:".to_string(), PANEL_TEXT_COLOR));
                    for (i, slot) in buildings.slots.iter().enumerate() {
                        lines.push((
                            format_slot(i, *slot),
                            colors::GRAY, // Example color for generic slots
                        ));
                    }
                }
            }
        }
    } else {
        lines.push((
            format!("selected: {} items", selection.len()),
            PANEL_TEXT_COLOR,
        ));
        let ship_count = selection
            .iter()
            .filter(|id| world.ships.contains_key(id))
            .count();
        if ship_count > 0 {
            lines.push((format!("- {} ships", ship_count), colors::GRAY));
        }
    }

    if lines.is_empty() {
        return;
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
