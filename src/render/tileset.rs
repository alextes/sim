use sdl2::rect::Rect;
use std::collections::HashMap;
use tracing::error;

use super::TILE_PIXEL_WIDTH;

pub fn make_tile_rect(x: u8, y: u8) -> Rect {
    Rect::new(
        x as i32 * TILE_PIXEL_WIDTH as i32,
        y as i32 * TILE_PIXEL_WIDTH as i32,
        TILE_PIXEL_WIDTH as u32,
        TILE_PIXEL_WIDTH as u32,
    )
}

pub fn make_multi_tile_rect(x: u8, y: u8, width: u8, height: u8) -> Rect {
    Rect::new(
        x as i32 * TILE_PIXEL_WIDTH as i32,
        y as i32 * TILE_PIXEL_WIDTH as i32,
        width as u32 * TILE_PIXEL_WIDTH as u32,
        height as u32 * TILE_PIXEL_WIDTH as u32,
    )
}

pub struct Tileset {
    char_rects: HashMap<char, Rect>,
}

impl Tileset {
    pub fn new() -> Self {
        let mut char_rects = HashMap::new();
        // Populate the HashMap with all character to Rect mappings
        // This is a direct port of the old match statement
        char_rects.insert(' ', make_tile_rect(0, 0));
        char_rects.insert('!', make_tile_rect(1, 2));
        char_rects.insert('"', make_tile_rect(2, 2));
        char_rects.insert('#', make_tile_rect(3, 2));
        char_rects.insert('$', make_tile_rect(4, 2));
        char_rects.insert('%', make_tile_rect(5, 2));
        char_rects.insert('&', make_tile_rect(6, 2));
        char_rects.insert('\'', make_tile_rect(7, 2));
        char_rects.insert('(', make_tile_rect(8, 2));
        char_rects.insert(')', make_tile_rect(9, 2));
        char_rects.insert('*', make_tile_rect(10, 2));
        char_rects.insert('+', make_tile_rect(11, 2));
        char_rects.insert(',', make_tile_rect(12, 2));
        char_rects.insert('-', make_tile_rect(13, 2));
        char_rects.insert('.', make_tile_rect(14, 2));
        char_rects.insert('/', make_tile_rect(15, 2));
        char_rects.insert('0', make_tile_rect(0, 3));
        char_rects.insert('1', make_tile_rect(1, 3));
        char_rects.insert('2', make_tile_rect(2, 3));
        char_rects.insert('3', make_tile_rect(3, 3));
        char_rects.insert('4', make_tile_rect(4, 3));
        char_rects.insert('5', make_tile_rect(5, 3));
        char_rects.insert('6', make_tile_rect(6, 3));
        char_rects.insert('7', make_tile_rect(7, 3));
        char_rects.insert('8', make_tile_rect(8, 3));
        char_rects.insert('9', make_tile_rect(9, 3));
        char_rects.insert(':', make_tile_rect(10, 3));
        char_rects.insert(';', make_tile_rect(11, 3));
        char_rects.insert('<', make_tile_rect(12, 3));
        char_rects.insert('=', make_tile_rect(13, 3));
        char_rects.insert('>', make_tile_rect(14, 3));
        char_rects.insert('?', make_tile_rect(15, 3));
        char_rects.insert('A', make_tile_rect(1, 4));
        char_rects.insert('B', make_tile_rect(2, 4));
        char_rects.insert('C', make_tile_rect(3, 4));
        char_rects.insert('D', make_tile_rect(4, 4));
        char_rects.insert('E', make_tile_rect(5, 4));
        char_rects.insert('F', make_tile_rect(6, 4));
        char_rects.insert('G', make_tile_rect(7, 4));
        char_rects.insert('H', make_tile_rect(8, 4));
        char_rects.insert('I', make_tile_rect(9, 4));
        char_rects.insert('J', make_tile_rect(10, 4));
        char_rects.insert('K', make_tile_rect(11, 4));
        char_rects.insert('L', make_tile_rect(12, 4));
        char_rects.insert('M', make_tile_rect(13, 4));
        char_rects.insert('N', make_tile_rect(14, 4));
        char_rects.insert('O', make_tile_rect(15, 4));
        char_rects.insert('P', make_tile_rect(0, 5));
        char_rects.insert('Q', make_tile_rect(1, 5));
        char_rects.insert('R', make_tile_rect(2, 5));
        char_rects.insert('S', make_tile_rect(3, 5));
        char_rects.insert('T', make_tile_rect(4, 5));
        char_rects.insert('U', make_tile_rect(5, 5));
        char_rects.insert('V', make_tile_rect(6, 5));
        char_rects.insert('W', make_tile_rect(7, 5));
        char_rects.insert('X', make_tile_rect(8, 5));
        char_rects.insert('Y', make_tile_rect(9, 5));
        char_rects.insert('Z', make_tile_rect(10, 5));
        char_rects.insert('a', make_tile_rect(1, 6));
        char_rects.insert('b', make_tile_rect(2, 6));
        char_rects.insert('c', make_tile_rect(3, 6));
        char_rects.insert('d', make_tile_rect(4, 6));
        char_rects.insert('e', make_tile_rect(5, 6));
        char_rects.insert('f', make_tile_rect(6, 6));
        char_rects.insert('g', make_tile_rect(7, 6));
        char_rects.insert('h', make_tile_rect(8, 6));
        char_rects.insert('i', make_tile_rect(9, 6));
        char_rects.insert('j', make_tile_rect(10, 6));
        char_rects.insert('k', make_tile_rect(11, 6));
        char_rects.insert('l', make_tile_rect(12, 6));
        char_rects.insert('m', make_tile_rect(13, 6));
        char_rects.insert('n', make_tile_rect(14, 6));
        char_rects.insert('o', make_tile_rect(15, 6));
        char_rects.insert('p', make_tile_rect(0, 7));
        char_rects.insert('q', make_tile_rect(1, 7));
        char_rects.insert('r', make_tile_rect(2, 7));
        char_rects.insert('s', make_tile_rect(3, 7));
        char_rects.insert('t', make_tile_rect(4, 7));
        char_rects.insert('u', make_tile_rect(5, 7));
        char_rects.insert('v', make_tile_rect(6, 7));
        char_rects.insert('w', make_tile_rect(7, 7));
        char_rects.insert('x', make_tile_rect(8, 7));
        char_rects.insert('y', make_tile_rect(9, 7));
        char_rects.insert('z', make_tile_rect(10, 7));

        Tileset { char_rects }
    }

    pub fn get_rect(&self, character: char) -> Rect {
        match self.char_rects.get(&character) {
            Some(rect) => *rect,
            None => {
                error!(
                    "unsupported character encountered: '{}', defaulting to '?'",
                    character
                );
                // '?' is guaranteed to be in char_rects by Tileset::new()
                *self
                    .char_rects
                    .get(&'?')
                    .expect("critical: default character '?' not found in tileset!")
            }
        }
    }
}

impl Default for Tileset {
    fn default() -> Self {
        Self::new()
    }
}
