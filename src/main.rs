use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use std::{path::Path, time::Duration};
use text::render_status_text;

mod render {
    use std::collections::HashMap;

    use lazy_static::lazy_static;
    use sdl2::rect::Rect;
    use sdl2::render::{Canvas, Texture};
    use sdl2::video::Window;

    use crate::location::LocationMap;
    use crate::tiles::{make_tileset_rect, Renderable, TILE_PIXEL_WIDTH};
    use crate::{colors, EntityId, EntityType, Point, Viewport};

    lazy_static! {
        static ref EMPTY_TILE: Rect = make_tileset_rect(0, 0);
        static ref LOWER_P_TILE: Rect = make_tileset_rect(0, 7);
    }

    fn source_rect_from_entity(entity: &EntityType) -> Rect {
        use EntityType::*;
        match entity {
            Space => *EMPTY_TILE,
            Planet => *LOWER_P_TILE,
        }
    }

    fn render_tile(
        canvas: &mut Canvas<Window>,
        tiles_texture: &mut Texture<'_>,
        renderable: &Renderable,
    ) {
        tiles_texture.set_color_mod(renderable.color.r, renderable.color.g, renderable.color.b);

        canvas
            .copy(
                tiles_texture,
                Some(renderable.tileset_rect),
                Some(Rect::new(
                    renderable.x as i32 * TILE_PIXEL_WIDTH as i32,
                    renderable.y as i32 * TILE_PIXEL_WIDTH as i32,
                    TILE_PIXEL_WIDTH as u32,
                    TILE_PIXEL_WIDTH as u32,
                )),
            )
            .unwrap();
    }

    pub fn render_tiles(
        canvas: &mut Canvas<Window>,
        tiles_texture: &mut Texture<'_>,
        entity_type_map: &HashMap<EntityId, EntityType>,
        location_map: &LocationMap,
        viewport: &Viewport,
    ) {
        let visible_translated_location_map =
            location_map.iter().filter_map(|(entity_id, location)| {
                if location.x >= viewport.min_x()
                    && location.x <= viewport.max_x()
                    && location.y >= viewport.min_y()
                    && location.y <= viewport.max_y()
                {
                    Some((
                        entity_id,
                        LocationMap::translate_location(location, viewport),
                    ))
                } else {
                    None
                }
            });

        for (entity_id, Point { x, y }) in visible_translated_location_map {
            let entity_type = entity_type_map
                .get(&entity_id)
                .expect("expect entity type to be stored for entity id");
            let renderable = Renderable {
                x: x as u8,
                y: y as u8,
                tileset_rect: source_rect_from_entity(entity_type),
                color: colors::BLUE,
            };
            render_tile(canvas, tiles_texture, &renderable);
        }
    }
}

mod colors {
    use sdl2::pixels::Color;

    pub const BASE: Color = Color::RGB(36, 39, 58);
    pub const BLUE: Color = Color::RGB(138, 173, 244);
    pub const WHITE: Color = Color::RGB(202, 211, 245);
}

mod tiles {
    use sdl2::pixels::Color;
    use sdl2::rect::Rect;

    // Starting with a small limited amount of space, to be expanded to galaxy size.
    pub const TILE_PIXEL_WIDTH: u8 = 9;

    pub struct Renderable {
        pub color: Color,
        pub tileset_rect: Rect,
        pub x: u8,
        pub y: u8,
    }

    pub fn make_tile_rect(x: u8, y: u8) -> Rect {
        Rect::new(
            x as i32 * TILE_PIXEL_WIDTH as i32,
            y as i32 * TILE_PIXEL_WIDTH as i32,
            TILE_PIXEL_WIDTH as u32,
            TILE_PIXEL_WIDTH as u32,
        )
    }

    pub fn make_tileset_rect(x: u8, y: u8) -> Rect {
        make_tile_rect(x, y)
    }

    pub fn make_multi_tile_rect(x: u8, y: u8, width: u8, height: u8) -> Rect {
        Rect::new(
            x as i32 * TILE_PIXEL_WIDTH as i32,
            y as i32 * TILE_PIXEL_WIDTH as i32,
            width as u32 * TILE_PIXEL_WIDTH as u32,
            height as u32 * TILE_PIXEL_WIDTH as u32,
        )
    }
}

mod text {
    use sdl2::pixels::Color;
    use sdl2::rect::Rect;
    use sdl2::render::{Canvas, Texture};
    use sdl2::video::Window;

    use crate::tiles::{make_multi_tile_rect, make_tile_rect, make_tileset_rect};

    fn rect_from_char(char: char) -> Rect {
        match char {
            ' ' => make_tileset_rect(0, 0),
            '!' => make_tileset_rect(1, 2),
            '0' => make_tileset_rect(0, 3),
            '1' => make_tileset_rect(1, 3),
            '2' => make_tileset_rect(2, 3),
            '3' => make_tileset_rect(3, 3),
            '4' => make_tileset_rect(4, 3),
            '5' => make_tileset_rect(5, 3),
            '6' => make_tileset_rect(6, 3),
            '7' => make_tileset_rect(7, 3),
            '8' => make_tileset_rect(8, 3),
            '9' => make_tileset_rect(9, 3),
            '?' => make_tileset_rect(15, 3),
            'A' => make_tileset_rect(1, 4),
            'B' => make_tileset_rect(2, 4),
            'C' => make_tileset_rect(3, 4),
            'D' => make_tileset_rect(4, 4),
            'E' => make_tileset_rect(5, 4),
            'F' => make_tileset_rect(6, 4),
            'G' => make_tileset_rect(7, 4),
            'H' => make_tileset_rect(8, 4),
            'I' => make_tileset_rect(9, 4),
            'J' => make_tileset_rect(10, 4),
            'K' => make_tileset_rect(11, 4),
            'L' => make_tileset_rect(12, 4),
            'M' => make_tileset_rect(13, 4),
            'N' => make_tileset_rect(14, 4),
            'O' => make_tileset_rect(15, 4),
            'P' => make_tileset_rect(0, 5),
            'Q' => make_tileset_rect(1, 5),
            'R' => make_tileset_rect(2, 5),
            'S' => make_tileset_rect(3, 5),
            'T' => make_tileset_rect(4, 5),
            'U' => make_tileset_rect(5, 5),
            'V' => make_tileset_rect(6, 5),
            'W' => make_tileset_rect(7, 5),
            'X' => make_tileset_rect(8, 5),
            'Y' => make_tileset_rect(9, 5),
            'Z' => make_tileset_rect(10, 5),
            'a' => make_tileset_rect(1, 6),
            'b' => make_tileset_rect(2, 6),
            'c' => make_tileset_rect(3, 6),
            'd' => make_tileset_rect(4, 6),
            'e' => make_tileset_rect(5, 6),
            'f' => make_tileset_rect(6, 6),
            'g' => make_tileset_rect(7, 6),
            'h' => make_tileset_rect(8, 6),
            'i' => make_tileset_rect(9, 6),
            'j' => make_tileset_rect(10, 6),
            'k' => make_tileset_rect(11, 6),
            'l' => make_tileset_rect(12, 6),
            'm' => make_tileset_rect(13, 6),
            'n' => make_tileset_rect(14, 6),
            'o' => make_tileset_rect(15, 6),
            'p' => make_tileset_rect(0, 7),
            'q' => make_tileset_rect(1, 7),
            'r' => make_tileset_rect(2, 7),
            's' => make_tileset_rect(3, 7),
            't' => make_tileset_rect(4, 7),
            'u' => make_tileset_rect(5, 7),
            'v' => make_tileset_rect(6, 7),
            'w' => make_tileset_rect(7, 7),
            'x' => make_tileset_rect(8, 7),
            'y' => make_tileset_rect(9, 7),
            'z' => make_tileset_rect(10, 7),
            char => panic!("tried to get rect for unsupported character: '{char}'"),
        }
    }

    pub fn render_status_text(
        canvas: &mut Canvas<Window>,
        tiles_texture: &mut Texture<'_>,
        text: &str,
        background_color: Color,
        foreground_color: Color,
    ) {
        canvas.set_draw_color(background_color);
        canvas
            .draw_rect(make_multi_tile_rect(
                (64 - text.len()).try_into().unwrap(),
                0,
                text.len().try_into().unwrap(),
                1,
            ))
            .unwrap();

        tiles_texture.set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

        let chars = text.chars();

        for (i, char) in chars.enumerate() {
            canvas
                .copy(
                    tiles_texture,
                    Some(rect_from_char(char)),
                    Some(make_tile_rect((64 - text.len() + i).try_into().unwrap(), 0)),
                )
                .unwrap();
        }
    }
}

const SIMULATION_UNIT_DURATION: Duration = Duration::from_millis(100);
const SIMULATION_UNIT_BUDGET: Duration = SIMULATION_UNIT_DURATION;

fn get_load_indicator_from_duration(duration: Duration) -> char {
    match duration {
        num if num <= Duration::from_millis(10) => '0',
        num if num > Duration::from_millis(10) && num <= Duration::from_millis(20) => '1',
        num if num > Duration::from_millis(20) && num <= Duration::from_millis(30) => '2',
        num if num > Duration::from_millis(30) && num <= Duration::from_millis(40) => '3',
        num if num > Duration::from_millis(40) && num <= Duration::from_millis(50) => '4',
        num if num > Duration::from_millis(50) && num <= Duration::from_millis(60) => '5',
        num if num > Duration::from_millis(60) && num <= Duration::from_millis(70) => '6',
        num if num > Duration::from_millis(70) && num <= Duration::from_millis(80) => '7',
        num if num > Duration::from_millis(80) && num <= Duration::from_millis(90) => '8',
        num if num > Duration::from_millis(90) && num <= Duration::from_millis(100) => '9',
        _ => '?',
    }
}

trait Simulate {
    fn get_simulation_interval() -> u32;
    fn progress(&mut self, simulation_units_counter: &u32);
}

type EntityId = u32;

pub enum EntityType {
    // Dude,
    // Grass,
    // ThickGrass,
    Planet,
    Space,
}

type EntityTypeMap = HashMap<EntityId, EntityType>;

mod location {
    use std::{
        collections::HashMap,
        ops::{Deref, DerefMut},
    };

    use crate::{EntityId, Point, Viewport};

    #[derive(Debug)]
    pub struct LocationMap(HashMap<EntityId, Point>);

    impl LocationMap {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn add_entity(&mut self, entity_id: EntityId, x: i32, y: i32) {
            self.0.insert(entity_id, Point { x, y });
        }

        pub fn translate_location(point: &Point, viewport: &Viewport) -> Point {
            Point {
                x: point.x - viewport.center.x,
                y: point.y - viewport.center.y,
            }
        }
    }

    impl Deref for LocationMap {
        type Target = HashMap<EntityId, Point>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for LocationMap {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
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
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG).unwrap();

    let window = video_subsystem
        .window("sim", 576, 576)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut tiles_texture = texture_creator
        .load_texture(Path::new("taffer.png"))
        .unwrap();

    let mut entities = vec![];
    let mut entity_type_map: EntityTypeMap = HashMap::new();
    let mut location_map = location::LocationMap::new();
    let location_viewport = Viewport {
        center: Point { x: 0, y: 0 },
        width: 64,
        height: 64,
    };

    entities.push(0);
    entity_type_map.insert(0, EntityType::Planet);
    location_map.add_entity(0, 32, 32);

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Tracks how much time has passed since we started counting up to one second.
    let mut loop_start;
    let mut simulation_load_history = VecDeque::from(vec!['?', '?', '?', '?', '?']);

    // Tracks how many simulation units (loops) were completed.
    let mut last_second_start = Instant::now();
    let mut simulation_units_counter = 0;
    let mut simulation_units_per_second = 0;

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
        render::render_tiles(
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
        let load_indicator = get_load_indicator_from_duration(loop_elapsed);
        simulation_load_history.push_back(load_indicator);
        let simulation_load_history_text: String = simulation_load_history.iter().collect();

        simulation_units_counter = simulation_units_counter + 1;

        render_status_text(
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
