pub mod tileset;
pub mod viewport;

use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::colors;
use crate::event_handling::ControlState;
use crate::interface::{self, DebugRenderInfo};
use crate::render::tileset::Tileset;
use crate::world::World;
use crate::GameState;

// Re-export key items from the viewport module
pub use viewport::{render_viewport, Viewport};

pub const TILE_PIXEL_WIDTH: u8 = 9;

pub struct SpriteSheetRenderer<'a, 't> {
    pub tileset: &'a Tileset,
    pub texture: &'a mut Texture<'t>,
}

pub fn render_game_frame<'t>(
    canvas: &mut Canvas<Window>,
    sprite_renderer: &mut SpriteSheetRenderer<'_, 't>,
    world: &World,
    location_viewport: &Viewport,
    controls: &ControlState,
    game_state: &GameState,
    debug_info: Option<DebugRenderInfo>,
) {
    canvas.set_draw_color(colors::BASE);
    canvas.clear();

    // render the main game viewport
    render_viewport(
        canvas,
        sprite_renderer,
        world,
        location_viewport,
        controls.debug_enabled,
    );

    // determine selected entity for the interface
    let selected_entity =
        if !world.entities.is_empty() && controls.entity_focus_index < world.entities.len() {
            Some(world.entities[controls.entity_focus_index])
        } else {
            None
        };

    // render UI elements (panels, etc.)
    interface::render_interface(
        canvas,
        sprite_renderer,
        world,
        selected_entity,
        location_viewport.screen_pixel_height / (TILE_PIXEL_WIDTH as u32),
        controls,
        debug_info,
    );

    // render context-specific menus based on game state
    match game_state {
        GameState::GameMenu => {
            interface::game_menu::render_game_menu(canvas, sprite_renderer);
        }
        GameState::BuildMenuSelectingSlotType => {
            interface::build::render_build_slot_type_menu(canvas, sprite_renderer);
        }
        GameState::BuildMenuSelectingBuilding { slot_type } => {
            interface::build::render_build_building_menu(canvas, sprite_renderer, *slot_type);
        }
        GameState::BuildMenuError { message } => {
            interface::build::render_build_error_menu(canvas, sprite_renderer, message);
        }
        GameState::Playing => {}
    }

    canvas.present();
}
