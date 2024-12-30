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

#[derive(Debug, Default, Clone)]
pub struct Point {
    x: i32,
    y: i32,
}

pub struct Viewport {
    /// Specifies which universe coordinate the top left corner of the viewport is centered on.
    anchor: Point,
    /// Specifies how far we're zoomed in on the universe, and therefore how many tiles are visible.
    zoom: f64,
    width: u32,
    height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            anchor: Point { x: -32, y: -32 },
            zoom: 1.0,
            width: 64,
            height: 64,
        }
    }
}

/// The viewport allows us to observe the universe at a chosen location, zoom level, overlay, etc.
/// TODO: the viewport should be bounded by the universe.
/// TODO: zooming should be capped at maybe 8x8 tiles minimum.
impl Viewport {
    fn min_x(&self) -> i32 {
        self.anchor.x
    }

    fn max_x(&self) -> i32 {
        self.anchor.x + self.width as i32
    }

    fn min_y(&self) -> i32 {
        self.anchor.y
    }

    fn max_y(&self) -> i32 {
        self.anchor.y + self.height as i32
    }

    pub fn center_on_entity(&mut self, x: i32, y: i32) {
        self.anchor.x = x - (self.width as i32 / 2);
        self.anchor.y = y - (self.height as i32 / 2);
    }
}

type SimulationUnit = u32;

trait Orbital {
    fn update_position(&mut self, anchor_x: i32, anchor_y: i32, time_delta: f64);
}

struct OrbitalEntity {
    id: u32,
    anchor_id: u32,
    radius: f64,
    angle: f64,
    angular_velocity: f64, // radians per second
    position: Point,
}

impl Orbital for OrbitalEntity {
    fn update_position(&mut self, anchor_x: i32, anchor_y: i32, time_delta: f64) {
        self.angle += self.angular_velocity * time_delta;
        self.position.x = anchor_x + (self.radius * self.angle.cos()) as i32;
        self.position.y = anchor_y + (self.radius * self.angle.sin()) as i32;
    }
}

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

    let mut entity_focus_index = 0;

    // Initialize orbital entities
    let mut orbital_entities = vec![
        OrbitalEntity {
            id: earth_id,
            anchor_id: sol_id,
            radius: 16.0,
            angle: 0.0,
            angular_velocity: 0.1,
            position: Point { x: 0, y: 0 },
        },
        OrbitalEntity {
            id: moon_id,
            anchor_id: earth_id,
            radius: 2.0,
            angle: 0.0,
            angular_velocity: 0.2,
            position: Point { x: 0, y: 0 },
        },
    ];

    'running: loop {
        // Mark loop start.
        loop_start = Instant::now();

        // Update positions of orbital entities
        for entity in &mut orbital_entities {
            let anchor_position = location_map.get(&entity.anchor_id).unwrap();
            entity.update_position(
                anchor_position.x,
                anchor_position.y,
                SIMULATION_UNIT_DURATION.as_secs_f64(),
            );
            location_map.add_entity(entity.id, entity.position.x, entity.position.y);
        }

        // Handle events.
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    location_viewport.anchor.y -= 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    location_viewport.anchor.y += 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    location_viewport.anchor.x -= 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    location_viewport.anchor.x += 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    ..
                } => {
                    entity_focus_index = (entity_focus_index + 1) % entities.len();
                    let entity_id = entities[entity_focus_index];
                    let Point { x: ex, y: ey } =
                        location_map.get(&entity_id).cloned().unwrap_or_default();
                    location_viewport.center_on_entity(ex, ey);
                }
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
