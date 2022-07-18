use render::render_tiles;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::{path::Path, time::Duration};
use tile::fill_tiles_with_earth;

mod render {
    use sdl2::rect::Rect;
    use sdl2::render::{Canvas, Texture};
    use sdl2::video::Window;

    use crate::tiles::{make_tileset_rect, Entity, Tile, Tiles, TILE_PIXEL_WIDTH};

    fn source_rect_from_tile(entity: &Entity) -> Rect {
        match entity {
            Entity::Dude => rect_from_pos(1, 0),
            Entity::Grass => rect_from_pos(13, 9),
            Entity::ThickGrass => rect_from_pos(13, 9),
            Entity::Earth => rect_from_pos(14, 2),
        }
    }

    fn render_tile(canvas: &mut Canvas<Window>, tiles_texture: &mut Texture<'_>, tile: &Tile) {
        tiles_texture.set_color_mod(tile.color.r, tile.color.g, tile.color.b);

        canvas
            .copy(
                tiles_texture,
                Some(source_rect_from_tile(&tile.entity)),
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
        canvas.present();
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

    #[derive(Debug)]
    pub enum Entity {
        Dude,
        Grass,
        ThickGrass,
        Earth,
    }

    #[derive(Debug)]
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

mod text {}

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
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
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

        render_tiles(&mut canvas, &mut tiles_texture, &mut tiles);

        // Sleep so we don't loop crazy fast.
        // Replace this with an adjustable simulation rate.
        std::thread::sleep(Duration::from_secs(1 / 60));
    }
}
