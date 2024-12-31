mod colors;
mod entity;
mod entity_setup;
mod event_handling;
mod initialization;
mod load;
mod location;
mod render;
mod simulation;

use entity::{EntityTypeMap, OrbitalEntity};
use location::LocationMap;
use render::Viewport;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

const SIMULATION_UNIT_DURATION: Duration = Duration::from_millis(100);
const SIMULATION_UNIT_BUDGET: Duration = SIMULATION_UNIT_DURATION;

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

    let (entities, entity_type_map, mut location_map, mut orbital_entities) =
        entity_setup::initialize_entities();

    let mut location_viewport = Viewport::default();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut loop_start;
    let mut simulation_load_history = VecDeque::from(vec!['?', '?', '?', '?', '?']);

    let mut last_second_start = Instant::now();
    let mut simulation_units_counter: SimulationUnit = 0;
    let mut simulation_units_per_second: SimulationUnit = 0;

    let one_second_duration = Duration::from_secs(1);

    let mut entity_focus_index = 0;

    'running: loop {
        loop_start = Instant::now();

        simulation::update_orbital_entities(&mut orbital_entities, &mut location_map);

        if !event_handling::handle_events(
            &mut event_pump,
            &mut location_viewport,
            &entities,
            &location_map,
            &mut entity_focus_index,
        ) {
            break 'running;
        }

        canvas.clear();

        render::render_viewport(
            &mut canvas,
            &mut tiles_texture,
            &entity_type_map,
            &location_map,
            &location_viewport,
        );

        simulation_load_history.pop_front();
        let loop_elapsed = loop_start.elapsed();
        let load_indicator = load::get_load_indicator_from_duration(loop_elapsed);
        simulation_load_history.push_back(load_indicator);
        let simulation_load_history_text: String = simulation_load_history.iter().collect();

        simulation_units_counter += 1;

        render::render_status_text(
            &mut canvas,
            &mut tiles_texture,
            &format!(
                "LOAD {} SUPS {}",
                simulation_load_history_text, simulation_units_per_second
            ),
            colors::BASE,
            colors::WHITE,
        );

        match last_second_start.elapsed().cmp(&one_second_duration) {
            Ordering::Less => (),
            Ordering::Equal | Ordering::Greater => {
                simulation_units_per_second = simulation_units_counter;
                simulation_units_counter = 0;
                last_second_start = Instant::now();
            }
        }

        canvas.present();

        let simulation_unit_budget_left =
            SIMULATION_UNIT_BUDGET.as_millis() as i64 - loop_elapsed.as_millis() as i64;
        let duration_to_sleep = Duration::from_millis(simulation_unit_budget_left.max(0) as u64);
        std::thread::sleep(duration_to_sleep);
    }
}
