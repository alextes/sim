extern crate sdl2;

use colors::Color;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::{path::Path, time::Duration};

mod colors {
    pub struct Color(pub u8, pub u8, pub u8);
    pub const RED: Color = Color(255, 0, 0);
    pub const GREEN: Color = Color(0, 255, 0);
    pub const BLUE: Color = Color(0, 0, 255);
    pub const WHITE: Color = Color(255, 255, 255);
}

enum Tile {
    Dude,
    Grass,
}

fn source_rect_from_tile(tile: &Tile) -> Rect {
    match tile {
        Tile::Dude => Rect::new(1 * 9, 0 * 9, 9, 9),
        Tile::Grass => Rect::new(13 * 9, 3 * 9, 9, 9),
    }
}

fn draw_tile(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    tile: &Tile,
    color: &Color,
) {
    tiles_texture.set_color_mod(color.0, color.1, color.2);

    canvas
        .copy(
            tiles_texture,
            Some(source_rect_from_tile(tile)),
            Some(Rect::new(200, 200, 9, 9)),
        )
        .unwrap();
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG).unwrap();

    let window = video_subsystem
        .window("sim", 400, 400)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut tiles_texture = texture_creator
        .load_texture(Path::new("taffer.png"))
        .unwrap();

    draw_tile(&mut canvas, &mut tiles_texture, &Tile::Dude, &colors::WHITE);
    canvas.present();

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
        // The rest of the game loop goes here...
        std::thread::sleep(Duration::from_secs(1 / 60));
    }
}
