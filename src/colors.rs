//! Colors for coloring tiles.
//!
//! Colors are defined as RGB tuples, and are used in the `render` module.
//!
//! Colors were originally taken from the lee v2 dwarf fortress color set.
#![allow(dead_code)]
use sdl2::pixels::Color;

pub const BASE: Color = Color::RGB(36, 39, 58);
pub const BLUE: Color = Color::RGB(138, 173, 244);
pub const WHITE: Color = Color::RGB(202, 211, 245);
pub const BLACK: Color = Color::RGB(21, 19, 15);
pub const GREEN: Color = Color::RGB(80, 135, 20);
pub const CYAN: Color = Color::RGB(25, 140, 140);
pub const RED: Color = Color::RGB(160, 20, 10);
pub const MAGENTA: Color = Color::RGB(135, 60, 130);
pub const BROWN: Color = Color::RGB(150, 75, 55);
pub const LGRAY: Color = Color::RGB(178, 175, 172);
pub const DGRAY: Color = Color::RGB(116, 110, 113);
pub const LBLUE: Color = Color::RGB(105, 135, 225);
pub const LGREEN: Color = Color::RGB(125, 185, 55);
pub const LCYAN: Color = Color::RGB(60, 205, 190);
pub const LRED: Color = Color::RGB(220, 50, 20);
pub const LMAGENTA: Color = Color::RGB(190, 110, 185);
pub const YELLOW: Color = Color::RGB(230, 170, 30);

pub const GRAY: Color = Color::RGB(128, 128, 128);
pub const ORANGE: Color = Color::RGB(255, 165, 0);
