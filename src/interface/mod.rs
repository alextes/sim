use crate::colors;
use crate::render::{tileset, SpriteSheetRenderer, TILE_PIXEL_WIDTH};
use crate::world::{EntityId, World};
use sdl2::render::Canvas;
use sdl2::video::Window;

pub mod build;
pub mod debug_overlay;
pub mod game_menu;
pub mod intro;
pub mod main_menu;
pub mod selected_object_panel;
pub mod shipyard_menu;
pub mod sim_speed_panel;
pub mod stardate_panel;

/// data required for rendering the debug overlay.
#[derive(Clone, Copy)]
pub struct DebugRenderInfo {
    pub sups: u64,
    pub fps: u32,
    pub zoom: f64,
}

pub struct UIContext<'a> {
    pub world: &'a World,
    pub selection: &'a [EntityId],
    pub viewport_height_tiles: u32,
    pub controls: &'a crate::event_handling::ControlState,
    pub debug_info: Option<DebugRenderInfo>,
    pub total_sim_ticks: u64,
}

/// helper to render text aligned at the given (x,y) tile coordinates.
/// this mirrors the implementation found in `render_status_text` but
/// allows specifying the x position instead of always right-aligning.
pub fn render_text_at(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
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

    renderer.set_texture_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

    for (i, ch) in text.chars().enumerate() {
        let src = renderer.tileset.get_rect(ch);
        let dst = tileset::make_tile_rect(x_tile + i as u8, y_tile);
        canvas
            .copy(&renderer.texture_ref(), Some(src), Some(dst))
            .ok();
    }
}

// constants for panels
pub const PANEL_BORDER_COLOR: sdl2::pixels::Color = colors::DGRAY; // dark gray border
pub const PANEL_BACKGROUND_COLOR: sdl2::pixels::Color = colors::BLACK;
pub const PANEL_TEXT_COLOR: sdl2::pixels::Color = colors::WHITE;
pub const SCREEN_EDGE_MARGIN: u8 = 1; // general margin from the absolute screen edge for right-aligned panels

/// render resource counters (top-left), selection panel (bottom-left) and sim speed (top-right).
/// also renders debug overlay if enabled.
pub fn render_interface(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    ctx: &UIContext,
) {
    // --- left-aligned panels ---
    let (screen_px_w, _) = canvas.output_size().unwrap_or((0, 0));
    let screen_tiles_w = (screen_px_w / TILE_PIXEL_WIDTH as u32) as u8;

    stardate_panel::render_stardate_panel(
        canvas,
        renderer,
        ctx.total_sim_ticks,
        1, // Y-coordinate for the topmost panel
        screen_tiles_w,
    );
    selected_object_panel::render_selected_object_panel(
        canvas,
        renderer,
        ctx.world,
        ctx.selection,
        ctx.controls.track_mode,
        ctx.viewport_height_tiles,
    );

    // --- right-aligned panels (top-right corner) ---
    const TOP_SCREEN_MARGIN: u8 = 1; // Y-coordinate for the topmost panel
    const PANEL_SPACING: u8 = 1; // Vertical spacing between panels

    let mut current_y_offset = TOP_SCREEN_MARGIN;

    // render sim speed panel
    let sim_speed_panel_height = sim_speed_panel::render_sim_speed_panel(
        canvas,
        renderer,
        ctx.controls.sim_speed,
        ctx.controls.paused,
        current_y_offset, // Its top y position
        screen_tiles_w,   // Screen width for its internal right-alignment
    );
    current_y_offset += sim_speed_panel_height + PANEL_SPACING;

    // render debug overlay panel (if enabled and data is present)
    if ctx.controls.debug_enabled {
        if let Some(info) = ctx.debug_info {
            debug_overlay::render_debug_overlay(
                canvas,
                renderer,
                info.sups,
                info.fps,
                info.zoom,
                current_y_offset, // Its top y position, below sim_speed_panel
                screen_tiles_w,   // Screen width for its internal right-alignment
            );
        }
    }
}

// helper function to draw a centered window with a border and text lines.
// made pub(super) to be accessible by submodules like build and pause_menu.
pub(super) fn draw_centered_window(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    lines: &[(String, sdl2::pixels::Color)],
) {
    const PADDING: u8 = 1; // 1 tile padding around text content, inside the panel

    // dimensions of the text content itself
    let text_content_w = lines.iter().map(|(s, _)| s.len()).max().unwrap_or(0) as u8;
    let text_content_h = lines.len() as u8;

    // total dimensions of the panel (text + padding on all sides)
    let panel_w = text_content_w + 2 * PADDING;
    let panel_h = text_content_h + 2 * PADDING;

    // screen size in tiles
    let (px_w, px_h) = canvas.output_size().unwrap();
    let tiles_w = (px_w / TILE_PIXEL_WIDTH as u32) as u8;
    let tiles_h = (px_h / TILE_PIXEL_WIDTH as u32) as u8;

    // top-left corner of the panel
    let panel_x = tiles_w.saturating_sub(panel_w) / 2;
    let panel_y = tiles_h.saturating_sub(panel_h) / 2;

    // 1. draw the background for the entire panel area
    canvas.set_draw_color(PANEL_BACKGROUND_COLOR);
    canvas
        .fill_rect(tileset::make_multi_tile_rect(
            panel_x, panel_y, panel_w, panel_h,
        ))
        .unwrap();

    // 2. draw the border outline on top of the background
    canvas.set_draw_color(PANEL_BORDER_COLOR);
    canvas
        .draw_rect(tileset::make_multi_tile_rect(
            panel_x, panel_y, panel_w, panel_h,
        ))
        .unwrap();

    // 3. render text lines, offset by padding from the panel's edge
    let text_start_y = panel_y + PADDING;

    for (i, (text, fg)) in lines.iter().enumerate() {
        // center each line of text individually
        let text_start_x = panel_x + PADDING + (text_content_w - text.len() as u8) / 2;
        render_text_at(
            canvas,
            renderer,
            text,
            PANEL_BACKGROUND_COLOR, // Text background matches the panel background
            *fg,
            text_start_x,
            text_start_y + i as u8,
        );
    }
}
