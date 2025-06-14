pub mod background;
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
use crate::render::background::BackgroundLayer;
use crate::render::tileset::Tileset;
use crate::world::{EntityId, World};
use crate::GameState;

// Re-export key items from the viewport module
pub use viewport::{render_world_in_viewport, Viewport};

pub const TILE_PIXEL_WIDTH: u8 = 9;

pub struct RenderContext<'a, 'tc> {
    pub canvas: &'a mut Canvas<Window>,
    pub sprite_renderer: &'a SpriteSheetRenderer<'tc>,
    pub background_layer: &'a BackgroundLayer,
    pub world: &'a World,
    pub location_viewport: &'a Viewport,
    pub controls: &'a ControlState,
    pub game_state: &'a GameState,
    pub debug_info: Option<DebugRenderInfo>,
    pub intro_progress: Option<f64>,
    pub selected_id: Option<EntityId>,
}

pub struct SpriteSheetRenderer<'tc> {
    pub tileset: Tileset,
    pub texture: RefCell<Texture<'tc>>,
}

impl<'tc> SpriteSheetRenderer<'tc> {
    pub fn new(texture_creator: &'tc TextureCreator<WindowContext>) -> Self {
        let texture = texture_creator
            .load_texture(Path::new("res/taffer_9.png"))
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

fn render_game_scene(ctx: &mut RenderContext) {
    // render the parallax background first
    ctx.background_layer
        .render(ctx.canvas, ctx.sprite_renderer, ctx.location_viewport);

    // render the main game world
    render_world_in_viewport(
        ctx.canvas,
        ctx.sprite_renderer,
        ctx.world,
        ctx.location_viewport,
        ctx.controls.debug_enabled,
        ctx.selected_id,
    );

    // render UI elements (panels, etc.)
    interface::render_interface(
        ctx.canvas,
        ctx.sprite_renderer,
        ctx.world,
        ctx.selected_id,
        ctx.location_viewport.screen_pixel_height / (TILE_PIXEL_WIDTH as u32),
        ctx.controls,
        ctx.debug_info,
    );
}

pub fn render_game_frame(ctx: &mut RenderContext) {
    ctx.canvas.set_draw_color(colors::BASE);
    ctx.canvas.clear();

    match ctx.game_state {
        GameState::Intro => {
            if let Some(progress) = ctx.intro_progress {
                interface::intro::render_intro_screen(ctx.canvas, ctx.sprite_renderer, progress);
            }
        }
        GameState::MainMenu => {
            interface::main_menu::render_main_menu(ctx.canvas, ctx.sprite_renderer);
        }
        GameState::Playing => {
            render_game_scene(ctx);
        }
        GameState::GameMenu => {
            render_game_scene(ctx);
            interface::game_menu::render_game_menu(ctx.canvas, ctx.sprite_renderer);
        }
        GameState::BuildMenu => {
            render_game_scene(ctx);
            interface::build::render_build_menu(ctx.canvas, ctx.sprite_renderer);
        }
        GameState::BuildMenuError { message } => {
            render_game_scene(ctx);
            interface::build::render_build_error_menu(ctx.canvas, ctx.sprite_renderer, message);
        }
        GameState::ShipyardMenu => {
            render_game_scene(ctx);
            interface::shipyard_menu::render_shipyard_menu(ctx.canvas, ctx.sprite_renderer);
        }
        GameState::ShipyardMenuError { message } => {
            render_game_scene(ctx);
            interface::shipyard_menu::render_shipyard_error_menu(
                ctx.canvas,
                ctx.sprite_renderer,
                message,
            );
        }
    }

    ctx.canvas.present();
}
