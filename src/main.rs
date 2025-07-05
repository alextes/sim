mod buildings;
mod colors;
mod command;
mod event_handling;
mod game_loop;
mod initialization;
mod input;
mod interface;
mod location;
mod map_generation;
mod render;
mod ships;
mod world;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, trace};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::event_handling::ControlState;
use game_loop::GameLoop;
use interface::DebugRenderInfo;
use render::background::BackgroundLayer;
use render::{RenderContext, SpriteSheetRenderer, Viewport};
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Intro,
    MainMenu,
    Playing,
    GameMenu,
    BuildMenu { mode: BuildMenuMode },
    ShipyardMenu,
    ShipyardMenuError { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildMenuMode {
    Main,
    SelectBuilding,
    EnterQuantity {
        building: world::types::BuildingType,
        quantity_string: String,
    },
    ConfirmQuote {
        building: world::types::BuildingType,
        amount: u32,
    },
}

pub fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter) // Use the custom filter
        .init();

    info!("starting sim");

    let (sdl_context, mut canvas, texture_creator) = initialization::setup_sdl();

    let sprite_renderer = SpriteSheetRenderer::new(&texture_creator);

    let mut world = World::default();
    let mut rng = rand::rng();

    map_generation::populate_initial_galaxy(&mut world, &mut rng);

    let background_layer = BackgroundLayer::new(&mut rng);

    let mut location_viewport = Viewport::default();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut game_loop = GameLoop::new(SIMULATION_DT, RENDER_DT);
    // clear any simulation backlog from setup time
    let _ = game_loop.step();

    let mut last_loop_start = Instant::now();
    let mut simulation_units_counter: SimulationUnit = 0;
    let mut total_sim_ticks: u64 = 0;
    let mut simulation_units_per_second: SimulationUnit = 0;
    let mut fps_counter: u32 = 0;
    let mut fps_per_second: u32 = 0;

    let mut controls = ControlState {
        selection: if world.entities.is_empty() {
            vec![]
        } else {
            vec![world.entities[0]]
        },
        debug_enabled: false,
        track_mode: false,
        sim_speed: 1,
        paused: false,
        middle_mouse_dragging: false,
        ctrl_left_mouse_dragging: false,
        ctrl_down: false,
        last_mouse_pos: None,
        selection_box_start: None,
    };

    let game_state = Arc::new(Mutex::new(GameState::Intro));
    let intro_start_time = Instant::now();

    info!("starting main loop");
    'running: loop {
        let now = Instant::now();
        let time_since_last_second_check = now.duration_since(last_loop_start);

        {
            let mut current_game_state = game_state.lock().unwrap();
            if *current_game_state == GameState::Intro
                && intro_start_time.elapsed() >= Duration::from_millis(100)
            {
                *current_game_state = GameState::MainMenu;
            }
        }

        // when paused, prevent simulation backlog from accumulating
        if controls.paused {
            game_loop.last_update = now;
        }

        // handle input events first
        let signal = event_handling::handle_events(
            &mut event_pump,
            &mut location_viewport,
            &mut world,
            &mut controls,
            game_state.clone(),
        );
        match signal {
            event_handling::Signal::Quit => break 'running,
            event_handling::Signal::Continue => {}
        }
        // advance simulation by appropriate number of steps based on time passed since last loop.
        let (steps, should_render) = game_loop.step();
        if *game_state.lock().unwrap() == GameState::Playing {
            for _ in 0..steps {
                if !controls.paused {
                    simulation_units_counter += 1;
                    total_sim_ticks += 1;
                    trace!(tick = total_sim_ticks, "simulating 1 step");
                    // advance the simulation by (dt * speed_multiplier)
                    world.update(
                        SIMULATION_DT.as_secs_f64() * controls.sim_speed as f64,
                        total_sim_ticks,
                    );
                }
            }
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

            let is_playing = *game_state.lock().unwrap() == GameState::Playing;

            // Tracking camera update
            if controls.track_mode && is_playing {
                if let Some(entity_id) = controls.selection.first() {
                    if let Some(loc) = world.get_location(*entity_id) {
                        location_viewport.center_on_entity(loc.x, loc.y);
                    }
                }
            }

            let current_game_state = game_state.lock().unwrap().clone();
            let debug_render_info = if controls.debug_enabled {
                Some(DebugRenderInfo {
                    sups: simulation_units_per_second,
                    fps: fps_per_second,
                    zoom: location_viewport.zoom,
                })
            } else {
                None
            };

            let intro_progress = if current_game_state == GameState::Intro {
                Some((intro_start_time.elapsed().as_secs_f64() / 0.1).min(1.0))
            } else {
                None
            };

            let mut ctx = RenderContext {
                canvas: &mut canvas,
                sprite_renderer: &sprite_renderer,
                background_layer: &background_layer,
                world: &world,
                location_viewport: &location_viewport,
                controls: &controls,
                game_state: &current_game_state,
                debug_info: debug_render_info,
                intro_progress,
                selection: &controls.selection,
                total_sim_ticks,
            };

            render::render_game_frame(&mut ctx);
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
