mod colors;
mod debug;
mod entity;
mod event_handling;
mod game_loop;
mod initialization;
mod location;
mod render;
mod world;

use debug::render_debug_overlay;
use game_loop::GameLoop;
use location::Point;
use render::Viewport;
use sdl2::image::LoadTexture;
use std::f64::consts::TAU;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use tracing::trace;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use world::World;

/// Fixed simulation timestep (100Hz)
const SIMULATION_DT: Duration = Duration::from_millis(10);
/// Render interval (10Hz)
const RENDER_DT: Duration = Duration::from_millis(100);
/// One second duration constant
const ONE_SECOND_DURATION: Duration = Duration::from_secs(1);

type SimulationUnit = u32;

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

    let mut world = World::new();
    // sol at center
    let sol_id = world.spawn_star("sol", Point { x: 0, y: 0 });
    // earth: complete one orbit (2π) in 10 seconds → angular_velocity = TAU / 10
    let earth_id = world.spawn_planet("earth", sol_id, 16.0, 0.0, TAU / 60.0);
    // moon: faster orbit around earth, e.g. complete in 5 seconds
    let _moon_id = world.spawn_moon("moon", earth_id, 4.0, 0.0, TAU / 5.0);

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

    info!("starting main loop");
    'running: loop {
        let now = Instant::now();
        // handle input events first
        let signal = event_handling::handle_events(
            &mut event_pump,
            &mut location_viewport,
            &mut world,
            &mut entity_focus_index,
            &mut debug_enabled,
        );
        match signal {
            event_handling::Signal::Quit => break 'running,
            event_handling::Signal::Continue => {}
        }
        // advance simulation step if it's time
        let (steps, should_render) = game_loop.step();
        for _ in 0..steps {
            trace!("simulating {} steps", steps);
            world.update(SIMULATION_DT.as_secs_f64());
            simulation_units_counter += 1;
        }

        // update per-second counters
        if now.duration_since(last_loop_start) >= ONE_SECOND_DURATION {
            simulation_units_per_second = simulation_units_counter;
            simulation_units_counter = 0;
            fps_per_second = fps_counter;
            fps_counter = 0;
            last_loop_start = now;
        }

        // render if it's time
        if should_render {
            trace!("rendering 1 frame");
            fps_counter += 1;
            canvas.set_draw_color(colors::BASE);
            canvas.clear();
            render::render_viewport(
                &mut canvas,
                &mut tiles_texture,
                &world,
                &location_viewport,
                debug_enabled,
            );
            if debug_enabled {
                render_debug_overlay(
                    &mut canvas,
                    &mut tiles_texture,
                    simulation_units_per_second,
                    fps_per_second,
                    location_viewport.zoom,
                );
            }
            canvas.present();
        }
        // tiny sleep to reduce busy-wait CPU usage
        let next_sim = game_loop.last_update + SIMULATION_DT;
        let next_rdr = game_loop.last_render + RENDER_DT;
        let wake_at = next_sim.min(next_rdr);
        if let Some(dur) = wake_at.checked_duration_since(now) {
            trace!("sleeping for {:?}", dur);
            std::thread::sleep(dur);
        }
    }
}
