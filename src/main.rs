mod colors;
mod debug;
mod entity;
mod event_handling;
mod initialization;
mod load;
mod location;
mod render;
mod world;

use debug::render_debug_overlay;
use location::Point;
use render::Viewport;
use sdl2::image::LoadTexture;
use std::collections::VecDeque;
use std::f64::consts::TAU;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use world::World;

/// Fixed simulation timestep (100Hz)
const SIMULATION_DT: Duration = Duration::from_millis(10);
/// Render interval (10FPS)
const RENDER_DT: Duration = Duration::from_millis(100);

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

    // let (entities, entity_type_map, mut location_map, mut orbital_entities) =
    // entity_setup::initialize_entities();

    let mut world = World::new();
    // sol at center
    let sol_id = world.spawn_star("sol", Point { x: 0, y: 0 });
    // earth: complete one orbit (2π) in 10 seconds → angular_velocity = TAU / 10
    let earth_id = world.spawn_planet("earth", sol_id, 16.0, 0.0, TAU / 60.0);
    // moon: faster orbit around earth, e.g. complete in 5 seconds
    let _moon_id = world.spawn_planet("moon", earth_id, 4.0, 0.0, TAU / 5.0);

    let mut location_viewport = Viewport::default();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut loop_start;
    let mut simulation_load_history = VecDeque::from(vec!['?', '?', '?', '?', '?']);

    let mut last_second_start = Instant::now();
    let mut simulation_units_counter: SimulationUnit = 0;
    let mut simulation_units_per_second: SimulationUnit = 0;
    let mut fps_counter: u32 = 0;
    let mut fps_per_second: u32 = 0;
    // timers for decoupled simulation/render
    let mut last_simulation = Instant::now();
    let mut last_render = Instant::now();

    let one_second_duration = Duration::from_secs(1);

    let mut entity_focus_index = 0;
    let mut debug_enabled = false;

    'running: loop {
        let now = Instant::now();
        loop_start = now;
        // run simulation updates at fixed 100Hz
        while now.duration_since(last_simulation) >= SIMULATION_DT {
            world.update(SIMULATION_DT.as_secs_f64());
            last_simulation += SIMULATION_DT;
            simulation_units_counter += 1;
        }

        // update per-second counters
        if now.duration_since(last_second_start) >= one_second_duration {
            simulation_units_per_second = simulation_units_counter;
            simulation_units_counter = 0;
            fps_per_second = fps_counter;
            fps_counter = 0;
            last_second_start = now;
        }

        let signal = event_handling::handle_events(
            &mut event_pump,
            &mut location_viewport,
            &mut world,
            &mut entity_focus_index,
            &mut debug_enabled,
        );
        match signal {
            event_handling::Signal::Quit => {
                break 'running;
            }
            event_handling::Signal::Continue => {}
        }

        // render at 10 FPS
        if now.duration_since(last_render) >= RENDER_DT {
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
            // debug overlay
            if debug_enabled {
                // update load-history
                let loop_elapsed = loop_start.elapsed();
                simulation_load_history.pop_front();
                let load_indicator = load::get_load_indicator_from_duration(loop_elapsed);
                simulation_load_history.push_back(load_indicator);
                let load_text: String = simulation_load_history.iter().collect();
                // show FPS and SUPS
                render_debug_overlay(
                    &mut canvas,
                    &mut tiles_texture,
                    &load_text,
                    simulation_units_per_second,
                    fps_per_second,
                    location_viewport.zoom,
                );
            }
            // present this frame
            canvas.present();
            last_render = now;
        }

        // tiny sleep to reduce busy-wait CPU usage
        std::thread::sleep(Duration::from_millis(1));
    }
}
