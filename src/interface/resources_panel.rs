use crate::colors;
use crate::render::{tileset, SpriteSheetRenderer};
use crate::world::World;
use sdl2::render::Canvas;
use sdl2::video::Window;

// Use shared constants and helpers from the parent module
use super::{render_text_at, PANEL_BACKGROUND_COLOR, PANEL_BORDER_COLOR};

pub fn render_resources_panel(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
) {
    let (energy_rate, metal_rate, nobles_rate, organics_rate) = world
        .resources
        .calculate_rates(&world.buildings, &world.celestial_data);

    let energy = world.resources.energy;
    let metal = world.resources.metal;
    let nobles = world.resources.nobles;
    let organics = world.resources.organics;

    let energy_text = format!("nrg: {:.1} (+{:.1}/s)", energy, energy_rate);
    let metal_text = format!("mtl: {:.1} (+{:.1}/s)", metal, metal_rate);
    let nobles_text = format!("nbl: {:.1} (+{:.1}/s)", nobles, nobles_rate);
    let organics_text = format!("org: {:.1} (+{:.1}/s)", organics, organics_rate);

    // Panel dimensions and position (top-left)
    let panel_padding: u8 = 1;
    let line_lens = [
        energy_text.len() as u8,
        metal_text.len() as u8,
        nobles_text.len() as u8,
        organics_text.len() as u8,
    ];
    let panel_inner_w = *line_lens.iter().max().unwrap_or(&0);
    let panel_total_w = panel_inner_w + panel_padding * 2;
    let panel_inner_h: u8 = 4; // Four lines of text
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
    let text_y_line3 = panel_y + panel_padding + 2;
    let text_y_line4 = panel_y + panel_padding + 3;

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

    render_text_at(
        canvas,
        renderer,
        &nobles_text,
        PANEL_BACKGROUND_COLOR, // Text background (same as panel)
        colors::LBLUE,          // nobles color
        text_x,
        text_y_line3,
    );

    render_text_at(
        canvas,
        renderer,
        &organics_text,
        PANEL_BACKGROUND_COLOR, // Text background (same as panel)
        colors::LGREEN,         // organics color
        text_x,
        text_y_line4,
    );
}
