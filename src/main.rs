use render::render_tiles;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::time::Instant;
use std::{path::Path, time::Duration};
use text::render_status_text;
use tiles::fill_tiles_with_earth;

mod render {
    use sdl2::rect::Rect;
    use sdl2::render::{Canvas, Texture};
    use sdl2::video::Window;

    use crate::tiles::{make_tileset_rect, Entity, Tile, Tiles, TILE_PIXEL_WIDTH};

    fn source_rect_from_entity(entity: &Entity) -> Rect {
        match entity {
            // Entity::Dude => make_tile_rect(1, 0),
            // Entity::Grass => make_tile_rect(13, 9),
            // Entity::ThickGrass => make_tile_rect(13, 9),
            Entity::Earth => make_tileset_rect(15, 3),
        }
    }

    fn render_tile(canvas: &mut Canvas<Window>, tiles_texture: &mut Texture<'_>, tile: &Tile) {
        tiles_texture.set_color_mod(tile.color.r, tile.color.g, tile.color.b);

        canvas
            .copy(
                tiles_texture,
                Some(source_rect_from_entity(&tile.entity)),
                Some(Rect::new(
                    tile.x as i32 * TILE_PIXEL_WIDTH as i32,
                    tile.y as i32 * TILE_PIXEL_WIDTH as i32,
                    TILE_PIXEL_WIDTH as u32,
                    TILE_PIXEL_WIDTH as u32,
                )),
            )
            .unwrap();
    }

    pub fn render_tiles(
        canvas: &mut Canvas<Window>,
        tiles_texture: &mut Texture<'_>,
        tiles: &mut Tiles,
    ) {
        for tile in tiles {
            render_tile(canvas, tiles_texture, tile)
        }
    }
}

mod colors {
    use sdl2::pixels::Color;

    pub const BLACK: Color = Color::RGB(21, 19, 15);
    // pub const BLUE: Color = Color::RGB(0, 0, 255);
    pub const BROWN: Color = Color::RGB(150, 75, 55);
    // pub const GREEN: Color = Color::RGB(0, 255, 0);
    // pub const RED: Color = Color::RGB(255, 0, 0);
    pub const WHITE: Color = Color::RGB(255, 255, 255);
}

mod tiles {
    use sdl2::pixels::Color;
    use sdl2::rect::Rect;

    use crate::colors;

    // 64x64 plane.
    pub type Tiles = Vec<Tile>;

    pub const TILES_WIDTH: u8 = 64;
    pub const TILE_PIXEL_WIDTH: u8 = 9;

    pub enum Entity {
        // Dude,
        // Grass,
        // ThickGrass,
        Earth,
    }

    pub struct Tile {
        pub x: u8,
        pub y: u8,
        pub entity: Entity,
        pub color: Color,
    }

    pub fn fill_tiles_with_earth(tiles: &mut Tiles) {
        for x in 0..TILES_WIDTH {
            for y in 0..TILES_WIDTH {
                let tile = Tile {
                    x,
                    y,
                    entity: Entity::Earth,
                    color: colors::BROWN,
                };
                tiles.push(tile);
            }
        }
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
        make_tile_rect(x - 1, y - 1)
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
            ' ' => make_tileset_rect(1, 1),
            '!' => make_tileset_rect(2, 3),
            '0' => make_tileset_rect(1, 4),
            '1' => make_tileset_rect(2, 4),
            '2' => make_tileset_rect(3, 4),
            '3' => make_tileset_rect(4, 4),
            '4' => make_tileset_rect(5, 4),
            '5' => make_tileset_rect(6, 4),
            '6' => make_tileset_rect(7, 4),
            '7' => make_tileset_rect(8, 4),
            '8' => make_tileset_rect(9, 4),
            '9' => make_tileset_rect(10, 4),
            '?' => make_tileset_rect(16, 4),
            'A' => make_tileset_rect(2, 5),
            'B' => make_tileset_rect(3, 5),
            'C' => make_tileset_rect(4, 5),
            'D' => make_tileset_rect(5, 5),
            'E' => make_tileset_rect(6, 5),
            'F' => make_tileset_rect(7, 5),
            'G' => make_tileset_rect(8, 5),
            'H' => make_tileset_rect(9, 5),
            'I' => make_tileset_rect(10, 5),
            'J' => make_tileset_rect(11, 5),
            'K' => make_tileset_rect(12, 5),
            'L' => make_tileset_rect(13, 5),
            'M' => make_tileset_rect(14, 5),
            'N' => make_tileset_rect(15, 5),
            'O' => make_tileset_rect(16, 5),
            'P' => make_tileset_rect(1, 6),
            'Q' => make_tileset_rect(2, 6),
            'R' => make_tileset_rect(3, 6),
            'S' => make_tileset_rect(4, 6),
            'T' => make_tileset_rect(5, 6),
            'U' => make_tileset_rect(6, 6),
            'V' => make_tileset_rect(7, 6),
            'W' => make_tileset_rect(8, 6),
            'X' => make_tileset_rect(9, 6),
            'Y' => make_tileset_rect(10, 6),
            'Z' => make_tileset_rect(11, 6),
            'a' => make_tileset_rect(2, 7),
            'b' => make_tileset_rect(3, 7),
            'c' => make_tileset_rect(4, 7),
            'd' => make_tileset_rect(5, 7),
            'e' => make_tileset_rect(6, 7),
            'f' => make_tileset_rect(7, 7),
            'g' => make_tileset_rect(8, 7),
            'h' => make_tileset_rect(9, 7),
            'i' => make_tileset_rect(10, 7),
            'j' => make_tileset_rect(11, 7),
            'k' => make_tileset_rect(12, 7),
            'l' => make_tileset_rect(13, 7),
            'm' => make_tileset_rect(14, 7),
            'n' => make_tileset_rect(15, 7),
            'o' => make_tileset_rect(16, 7),
            'p' => make_tileset_rect(1, 8),
            'q' => make_tileset_rect(2, 8),
            'r' => make_tileset_rect(3, 8),
            's' => make_tileset_rect(4, 8),
            't' => make_tileset_rect(5, 8),
            'u' => make_tileset_rect(6, 8),
            'v' => make_tileset_rect(7, 8),
            'w' => make_tileset_rect(8, 8),
            'x' => make_tileset_rect(9, 8),
            'y' => make_tileset_rect(10, 8),
            'z' => make_tileset_rect(11, 8),
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

    let mut tiles = Vec::with_capacity(4096);
    fill_tiles_with_earth(&mut tiles);

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Tracks how much time has passed since we started counting up to one second.
    let mut loop_start;
    let mut simulation_load_history = VecDeque::from(vec!['?', '?', '?', '?', '?']);

    // Tracks how many simulation units (loops) were completed.
    let mut last_second_start = Instant::now();
    let mut simulation_units_counter = 0;
    let mut simulation_units_per_second = 0;

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
        render_tiles(&mut canvas, &mut tiles_texture, &mut tiles);

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
            colors::BLACK,
            colors::WHITE,
        );

        // We update an indication of how many Simulation Units we're completing per second. Ideally this is
        // 10.
        match last_second_start.elapsed().cmp(&Duration::from_secs(1)) {
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
