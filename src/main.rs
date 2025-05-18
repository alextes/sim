mod buildings;
mod colors;
mod debug;
mod event_handling;
mod game_loop;
mod initialization;
mod input;
mod interface;
mod location;
mod render;
mod world;

use crate::buildings::{BuildingType, SlotType};
use debug::render_debug_overlay;
use game_loop::GameLoop;
use interface::render_interface;
use location::Point;
use render::{tileset::Tileset, SpriteSheetRenderer, Viewport};
use sdl2::image::LoadTexture;
use std::f64::consts::TAU;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::trace;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use world::World;

/// Fixed simulation timestep (100Hz)
pub const SIMULATION_DT: Duration = Duration::from_millis(10);
/// Render interval (10Hz)
const RENDER_DT: Duration = Duration::from_millis(100);
/// One second duration constant
const ONE_SECOND: Duration = Duration::from_secs(1);

/// Use u64 for tick counter to avoid potential overflow and match World::update
type SimulationUnit = u64;

/// Represents the different interaction modes the game can be in.
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Playing,
    BuildMenuSelectingSlotType,
    BuildMenuSelectingBuilding { slot_type: SlotType },
    BuildMenuError { message: String },
}

pub fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("starting sim");

    let (sdl_context, mut canvas, texture_creator) = initialization::setup_sdl();

    let mut tiles_texture = texture_creator
        .load_texture(Path::new("res/taffer.png"))
        .unwrap();

    debug!("tiles texture loaded");

    let tileset = Tileset::new();

    let mut world = World::default();
    // sol at center
    let sol_id = world.spawn_star("sol", Point { x: 0, y: 0 });
    // earth: complete one orbit (2π) in 10 seconds → angular_velocity = TAU / 10
    let earth_id = world.spawn_planet("earth", sol_id, 16.0, 0.0, TAU / 60.0);
    // moon: faster orbit around earth, e.g. complete in 5 seconds
    let _moon_id = world.spawn_moon("moon", earth_id, 4.0, 0.0, TAU / 5.0);

    // Pre-build on Earth
    if let Some(earth_buildings) = world.buildings.get_mut(&earth_id) {
        // Add a mine to the first available ground slot
        if let Some(ground_slot) = earth_buildings.find_first_empty_slot(SlotType::Ground) {
            earth_buildings
                .build(SlotType::Ground, ground_slot, BuildingType::Mine)
                .expect("Failed to build initial mine");
        }
        // Add a solar panel to the first available orbital slot
        if let Some(orbital_slot) = earth_buildings.find_first_empty_slot(SlotType::Orbital) {
            earth_buildings
                .build(SlotType::Orbital, orbital_slot, BuildingType::SolarPanel)
                .expect("Failed to build initial solar panel");
        }
    }

    let mut location_viewport = Viewport::default();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut game_loop = GameLoop::new(SIMULATION_DT, RENDER_DT);
    // Clear any simulation backlog from setup time
    let _ = game_loop.step();

    let mut last_loop_start = Instant::now();
    let mut simulation_units_counter: SimulationUnit = 0;
    let mut simulation_units_per_second: SimulationUnit = 0;
    let mut fps_counter: u32 = 0;
    let mut fps_per_second: u32 = 0;

    let mut entity_focus_index = 0;
    let mut debug_enabled = false;
    let mut track_mode = false;
    let game_state = Arc::new(Mutex::new(GameState::Playing));

    info!("starting main loop");
    'running: loop {
        let now = Instant::now();
        let time_since_last_second_check = now.duration_since(last_loop_start);

        let mut sprite_renderer = SpriteSheetRenderer {
            tileset: &tileset,
            texture: &mut tiles_texture,
        };

        // handle input events first
        let signal = event_handling::handle_events(
            &mut event_pump,
            &mut location_viewport,
            &mut world,
            &mut entity_focus_index,
            &mut debug_enabled,
            &mut track_mode,
            game_state.clone(),
        );
        match signal {
            event_handling::Signal::Quit => break 'running,
            event_handling::Signal::Continue => {}
        }
        // advance simulation by appropriate number of steps based on time passed since last loop.
        let (steps, should_render) = game_loop.step();
        for _ in 0..steps {
            simulation_units_counter += 1;
            trace!(tick = simulation_units_counter, "simulating 1 step");
            world.update(SIMULATION_DT.as_secs_f64(), simulation_units_counter);
        }

        if time_since_last_second_check >= ONE_SECOND {
            simulation_units_per_second = simulation_units_counter;
            simulation_units_counter = 0;
            fps_per_second = fps_counter;
            fps_counter = 0;

            last_loop_start = now;
        }

        if should_render {
            trace!("rendering 1 frame");
            fps_counter += 1;

            // clear the canvas *before* deciding what to render
            canvas.set_draw_color(colors::BASE);
            canvas.clear();

            // always render the world viewport first
            render::render_viewport(
                &mut canvas,
                &mut sprite_renderer,
                &world,
                &location_viewport,
                debug_enabled,
            );

            if debug_enabled {
                render_debug_overlay(
                    &mut canvas,
                    &mut sprite_renderer,
                    simulation_units_per_second,
                    fps_per_second,
                    location_viewport.zoom,
                );
            }

            // tracking camera update (only affects viewport positioning, so we compute before interface)
            if track_mode && !world.entities.is_empty() {
                let entity_id = world.entities[entity_focus_index];
                if let Some(loc) = world.get_location(entity_id) {
                    location_viewport.center_on_entity(loc.x, loc.y);
                }
            }

            // selection panel bottom-left
            let selected_entity = if !world.entities.is_empty() {
                Some(world.entities[entity_focus_index])
            } else {
                None
            };
            render_interface(
                &mut canvas,
                &mut sprite_renderer,
                &world,
                selected_entity,
                track_mode,
                location_viewport.screen_pixel_height / (render::TILE_PIXEL_WIDTH as u32),
            );

            // overlay build menus if not in playing state
            match &*game_state.lock().unwrap() {
                GameState::BuildMenuSelectingSlotType => {
                    interface::build::render_build_slot_type_menu(
                        &mut canvas,
                        &mut sprite_renderer,
                    );
                }
                GameState::BuildMenuSelectingBuilding { slot_type } => {
                    interface::build::render_build_building_menu(
                        &mut canvas,
                        &mut sprite_renderer,
                        *slot_type,
                    );
                }
                GameState::BuildMenuError { message } => {
                    interface::build::render_build_error_menu(
                        &mut canvas,
                        &mut sprite_renderer,
                        message,
                    );
                }
                _ => {} // GameState::Playing already handled
            }

            canvas.present();
        }
        let next_sim = game_loop.last_update + SIMULATION_DT;
        let next_rdr = game_loop.last_render + RENDER_DT;
        let wake_at = next_sim.min(next_rdr);
        if let Some(dur) = wake_at.checked_duration_since(now) {
            trace!("sleeping for {:?}", dur);
            std::thread::sleep(dur);
        }
    }
}
