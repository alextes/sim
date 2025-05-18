use crate::buildings::{BuildingType, EntityBuildings, GROUND_SLOTS, ORBITAL_SLOTS};
use crate::colors;
use crate::render::{tileset, SpriteSheetRenderer, TILE_PIXEL_WIDTH};
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

    // Panel dimensions and position (top-left)
    let panel_padding: u8 = 1;
    let line_1_len = energy_text.len() as u8;
    let line_2_len = metal_text.len() as u8;
    let panel_inner_w = line_1_len.max(line_2_len);
    let panel_total_w = panel_inner_w + panel_padding * 2;
    let panel_inner_h: u8 = 2; // Two lines of text
    let panel_total_h = panel_inner_h + panel_padding * 2;

    let panel_x: u8 = 1; // 1 tile from left
    let panel_y: u8 = 1; // 1 tile from top

    // Draw panel background
    canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // Draw panel border
    canvas.set_draw_color(PANEL_BORDER_COLOR);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // Text positions (inside panel with padding)
    let text_x = panel_x + panel_padding;
    let text_y_line1 = panel_y + panel_padding;
    let text_y_line2 = panel_y + panel_padding + 1;

    render_text_at(
        canvas,
        renderer,
        &energy_text,
        PANEL_BACKGROUND_COLOR, // Text background (same as panel)
        colors::YELLOW,         // energy color
        text_x,
        text_y_line1,
    );

    render_text_at(
        canvas,
        renderer,
        &metal_text,
        PANEL_BACKGROUND_COLOR, // Text background (same as panel)
        colors::LGRAY,          // metal color
        text_x,
        text_y_line2,
    );
}

fn render_sim_speed_panel(
    canvas: &mut Canvas<Window>,
    renderer: &mut SpriteSheetRenderer,
    sim_speed: u32,
    paused: bool,
) {
    let speed_text = if paused {
        "SPEED: PAUSED".to_string()
    } else {
        format!("SPEED: {}x", sim_speed)
    };
    let text_len = speed_text.len() as u8;

    // Position in top-right
    // Calculate screen width in tiles
    let (screen_px_w, _) = canvas.output_size().unwrap_or((0, 0));
    let screen_tiles_w = (screen_px_w / TILE_PIXEL_WIDTH as u32) as u8;

    // Panel dimensions
    let panel_padding: u8 = 1;
    let panel_inner_w = text_len;
    let panel_total_w = panel_inner_w + panel_padding * 2;
    let panel_inner_h: u8 = 1;
    let panel_total_h = panel_inner_h + panel_padding * 2;

    // Panel position (top-right)
    let panel_x = screen_tiles_w.saturating_sub(panel_total_w + 1); // +1 for margin from edge
    let panel_y: u8 = 1; // 1 tile from top

    // Text position (inside panel with padding)
    let text_x = panel_x + panel_padding;
    let text_y = panel_y + panel_padding;

    // Draw panel background
    canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // Draw panel border
    canvas.set_draw_color(PANEL_BORDER_COLOR);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            panel_x,
            panel_y,
            panel_total_w,
            panel_total_h,
        ))
        .unwrap();

    // Render text
    render_text_at(
        canvas,
        renderer,
        &speed_text,
        PANEL_BACKGROUND_COLOR, // Text background (same as panel)
        PANEL_TEXT_COLOR,       // Text foreground
        text_x,
        text_y,
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
    viewport_height_tiles: u32,
    controls: &crate::event_handling::ControlState,
) {
    render_resources_panel(canvas, renderer, world);
    render_selected_object_panel(
        canvas,
        renderer,
        world,
        selected,
        controls.track_mode,
        viewport_height_tiles,
    );
    render_sim_speed_panel(canvas, renderer, controls.sim_speed, controls.paused);
}
