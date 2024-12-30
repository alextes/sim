mod entity;
mod load;
mod location;
mod render;

use entity::{EntityType, EntityTypeMap};
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use std::{path::Path, time::Duration};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod colors {
    use sdl2::pixels::Color;

    pub const BASE: Color = Color::RGB(36, 39, 58);
    pub const BLUE: Color = Color::RGB(138, 173, 244);
    pub const WHITE: Color = Color::RGB(202, 211, 245);
}

const SIMULATION_UNIT_DURATION: Duration = Duration::from_millis(100);
const SIMULATION_UNIT_BUDGET: Duration = SIMULATION_UNIT_DURATION;

trait Simulate {
    fn get_simulation_interval() -> u32;
    fn progress(&mut self, simulation_units_counter: &u32);
}

#[derive(Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

pub struct Viewport {
    center: Point,
    width: u32,
    height: u32,
}

impl Viewport {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            center: Point { x, y },
            width,
            height,
        }
    }

    pub fn min_x(&self) -> i32 {
        self.center.x - (self.width / 2) as i32
    }

    pub fn max_x(&self) -> i32 {
        self.center.x + (self.width / 2) as i32
    }

    pub fn min_y(&self) -> i32 {
        self.center.y - (self.height / 2) as i32
    }

    pub fn max_y(&self) -> i32 {
        self.center.y + (self.height / 2) as i32
    }
}

type SimulationUnit = u32;

pub fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("starting sim");

    debug!("setting up SDL context");
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG).unwrap();

    debug!("creating SDL window");
    let window = video_subsystem
        .window("sim", 576, 576)
        .position_centered()
        .build()
        .unwrap();

    debug!("creating SDL canvas");
    let mut canvas = window.into_canvas().software().build().unwrap();

    debug!("loading tiles texture");
    let texture_creator = canvas.texture_creator();
    let mut tiles_texture = texture_creator
        .load_texture(Path::new("res/taffer.png"))
        .unwrap();

    let mut entities = vec![];
    let mut entity_type_map: EntityTypeMap = HashMap::new();
    let mut location_map = location::LocationMap::new();
    let mut location_viewport = Viewport::default();

    // Add Sol
    let sol_id = 0;
    entities.push(sol_id);
    entity_type_map.insert(sol_id, EntityType::Star);
    location_map.add_entity(sol_id, 0, 0);

    // Add Earth
    let earth_id = 1;
    entities.push(earth_id);
    entity_type_map.insert(earth_id, EntityType::Planet);
    location_map.add_entity(earth_id, -16, 0);

    // Add Moon
    let moon_id = 2;
    entities.push(moon_id);
    entity_type_map.insert(moon_id, EntityType::Moon);
    location_map.add_entity(moon_id, -16, 2);

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Tracks how much time has passed since we started counting up to one second.
    let mut loop_start;
    let mut simulation_load_history = VecDeque::from(vec!['?', '?', '?', '?', '?']);

    // Tracks how many simulation units (loops) were completed.
    let mut last_second_start = Instant::now();
    let mut simulation_units_counter: SimulationUnit = 0;
    let mut simulation_units_per_second: SimulationUnit = 0;

    let one_second_duration = Duration::from_secs(1);

    'running: loop {
        // Mark loop start.
        loop_start = Instant::now();

        // Handle events.
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.clear();

        // Render our tiles.
        render::render_viewport(
            &mut canvas,
            &mut tiles_texture,
            &entity_type_map,
            &location_map,
            &location_viewport,
        );

        // Calculate how long we took to complete the loop, and report the simulation speed.

        // First we print a load indicator. This is a simple measure of how much time was left out
        // of the time budget a single Simulation Unit has, namely 100ms. 0 indicates low load, 9
        // high.
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

        // We update an indication of how many Simulation Units we're completing per second. Ideally this is
        // 10.
        match last_second_start.elapsed().cmp(&one_second_duration) {
            Ordering::Less => (),
            Ordering::Equal | Ordering::Greater => {
                simulation_units_per_second = simulation_units_counter;
                simulation_units_counter = 0;
                last_second_start = Instant::now();
            }
        }

        canvas.present();

        // Sleep the rest of our budget.
        let simulation_unit_budget_left =
            SIMULATION_UNIT_BUDGET.as_millis() as i64 - loop_elapsed.as_millis() as i64;
        let duration_to_sleep = Duration::from_millis(simulation_unit_budget_left.max(0) as u64);
        std::thread::sleep(duration_to_sleep);
    }
}
