use lazy_static::lazy_static;
use sdl2::rect::Rect;

use crate::entity::EntityType;

use super::TILE_PIXEL_WIDTH;

lazy_static! {
    static ref EMPTY_TILE: Rect = make_tile_rect(0, 0);
    static ref EXCLAMATION_POINT_TILE: Rect = make_tile_rect(1, 2);
    static ref LOWER_M_TILE: Rect = make_tile_rect(13, 6);
    static ref LOWER_P_TILE: Rect = make_tile_rect(0, 7);
    static ref LOWER_S_TILE: Rect = make_tile_rect(3, 7);
}

impl From<&EntityType> for Rect {
    fn from(entity: &EntityType) -> Self {
        use EntityType::*;
        match entity {
            Moon => *LOWER_M_TILE,
            Planet => *LOWER_P_TILE,
            Space => *EMPTY_TILE,
            Star => *LOWER_S_TILE,
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

pub fn make_multi_tile_rect(x: u8, y: u8, width: u8, height: u8) -> Rect {
    Rect::new(
        x as i32 * TILE_PIXEL_WIDTH as i32,
        y as i32 * TILE_PIXEL_WIDTH as i32,
        width as u32 * TILE_PIXEL_WIDTH as u32,
        height as u32 * TILE_PIXEL_WIDTH as u32,
    )
}

pub fn rect_from_char(character: char) -> Rect {
    match character {
        ' ' => *EMPTY_TILE,
        '!' => *EXCLAMATION_POINT_TILE,
        '0' => make_tile_rect(0, 3),
        '1' => make_tile_rect(1, 3),
        '2' => make_tile_rect(2, 3),
        '3' => make_tile_rect(3, 3),
        '4' => make_tile_rect(4, 3),
        '5' => make_tile_rect(5, 3),
        '6' => make_tile_rect(6, 3),
        '7' => make_tile_rect(7, 3),
        '8' => make_tile_rect(8, 3),
        '9' => make_tile_rect(9, 3),
        '?' => make_tile_rect(15, 3),
        'A' => make_tile_rect(1, 4),
        'B' => make_tile_rect(2, 4),
        'C' => make_tile_rect(3, 4),
        'D' => make_tile_rect(4, 4),
        'E' => make_tile_rect(5, 4),
        'F' => make_tile_rect(6, 4),
        'G' => make_tile_rect(7, 4),
        'H' => make_tile_rect(8, 4),
        'I' => make_tile_rect(9, 4),
        'J' => make_tile_rect(10, 4),
        'K' => make_tile_rect(11, 4),
        'L' => make_tile_rect(12, 4),
        'M' => make_tile_rect(13, 4),
        'N' => make_tile_rect(14, 4),
        'O' => make_tile_rect(15, 4),
        'P' => make_tile_rect(0, 5),
        'Q' => make_tile_rect(1, 5),
        'R' => make_tile_rect(2, 5),
        'S' => make_tile_rect(3, 5),
        'T' => make_tile_rect(4, 5),
        'U' => make_tile_rect(5, 5),
        'V' => make_tile_rect(6, 5),
        'W' => make_tile_rect(7, 5),
        'X' => make_tile_rect(8, 5),
        'Y' => make_tile_rect(9, 5),
        'Z' => make_tile_rect(10, 5),
        'a' => make_tile_rect(1, 6),
        'b' => make_tile_rect(2, 6),
        'c' => make_tile_rect(3, 6),
        'd' => make_tile_rect(4, 6),
        'e' => make_tile_rect(5, 6),
        'f' => make_tile_rect(6, 6),
        'g' => make_tile_rect(7, 6),
        'h' => make_tile_rect(8, 6),
        'i' => make_tile_rect(9, 6),
        'j' => make_tile_rect(10, 6),
        'k' => make_tile_rect(11, 6),
        'l' => make_tile_rect(12, 6),
        'm' => make_tile_rect(13, 6),
        'n' => make_tile_rect(14, 6),
        'o' => make_tile_rect(15, 6),
        'p' => *LOWER_P_TILE,
        'q' => make_tile_rect(1, 7),
        'r' => make_tile_rect(2, 7),
        's' => make_tile_rect(3, 7),
        't' => make_tile_rect(4, 7),
        'u' => make_tile_rect(5, 7),
        'v' => make_tile_rect(6, 7),
        'w' => make_tile_rect(7, 7),
        'x' => make_tile_rect(8, 7),
        'y' => make_tile_rect(9, 7),
        'z' => make_tile_rect(10, 7),
        character => panic!("tried to get rect for unsupported character: '{character}'"),
    }
}
