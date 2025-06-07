pub mod tileset;
pub mod viewport;

use sdl2::image::LoadTexture;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::cell::{Ref, RefCell};
use std::path::Path;

use crate::colors;
use crate::event_handling::ControlState;
use crate::interface::{self, DebugRenderInfo};
use crate::render::tileset::Tileset;
use crate::world::World;
use crate::GameState;

// Re-export key items from the viewport module
pub use viewport::{render_world_in_viewport, Viewport};

pub const TILE_PIXEL_WIDTH: u8 = 18;

pub struct SpriteSheetRenderer<'tc> {
    pub tileset: Tileset,
    pub texture: RefCell<Texture<'tc>>,
}

impl<'tc> SpriteSheetRenderer<'tc> {
    pub fn new(texture_creator: &'tc TextureCreator<WindowContext>) -> Self {
        let texture = texture_creator
            .load_texture(Path::new("res/taffer_18.png"))
            .unwrap_or_else(|e| {
                panic!("failed to load sprite sheet texture: {}", e);
            });
        tracing::debug!("sprite sheet texture loaded by spritesheetrenderer");

        Self {
            tileset: Tileset::new(),
            texture: RefCell::new(texture),
        }
    }

    /// sets the color modulation for the underlying texture.
    pub fn set_texture_color_mod(&self, r: u8, g: u8, b: u8) {
        self.texture.borrow_mut().set_color_mod(r, g, b);
    }

    /// provides an immutable reference to the underlying texture, wrapped in a Ref guard.
    pub fn texture_ref(&self) -> Ref<Texture<'tc>> {
        self.texture.borrow()
    }
}

pub fn render_game_frame<'tc>(
    canvas: &mut Canvas<Window>,
    sprite_renderer: &SpriteSheetRenderer<'tc>,
    world: &World,
    location_viewport: &Viewport,
    controls: &ControlState,
    game_state: &GameState,
    debug_info: Option<DebugRenderInfo>,
) {
    canvas.set_draw_color(colors::BASE);
    canvas.clear();

    // render the main game world
    render_world_in_viewport(
        canvas,
        sprite_renderer,
        world,
        location_viewport,
        controls.debug_enabled,
    );

    // determine selected entity for the interface
    let selected_entity = if let Some(index) = controls.entity_focus_index {
        if !world.entities.is_empty() && index < world.entities.len() {
            Some(world.entities[index])
        } else {
            None
        }
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
        GameState::BuildMenu => {
            interface::build::render_build_menu(canvas, sprite_renderer);
        }
        GameState::BuildMenuError { message } => {
            interface::build::render_build_error_menu(canvas, sprite_renderer, message);
        }
        GameState::Playing => {}
    }

    canvas.present();
}
