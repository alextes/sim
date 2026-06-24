//! maps characters to tiles in the `res/taffer_9.png` atlas.
//!
//! ported from the old sdl `render::tileset`, but returns uv coordinates for
//! the wgpu sprite batch instead of pixel `sdl2::rect::Rect`s. the atlas is a
//! 16x16 grid of 9px tiles (a code page 437 layout).

use std::collections::HashMap;

/// the atlas is 144x144 pixels.
pub const ATLAS_PIXELS: f32 = 144.0;
/// each tile is 9x9 pixels.
pub const TILE_PX: f32 = 9.0;

pub struct Tileset {
    /// character -> (column, row) in the 16x16 tile grid.
    char_tiles: HashMap<char, (u8, u8)>,
}

impl Tileset {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(' ', (0, 0));
        // row 2: ! through / (columns 1..=15)
        for (i, c) in "!\"#$%&'()*+,-./".chars().enumerate() {
            m.insert(c, (1 + i as u8, 2));
        }
        // row 3: 0..9 then : ; < = > ? (columns 0..=15)
        for (i, c) in "0123456789:;<=>?".chars().enumerate() {
            m.insert(c, (i as u8, 3));
        }
        // row 4: A..O (columns 1..=15)
        for (i, c) in "ABCDEFGHIJKLMNO".chars().enumerate() {
            m.insert(c, (1 + i as u8, 4));
        }
        // row 5: P..Z then [ \ ] ^ _ (columns 0..=15)
        for (i, c) in "PQRSTUVWXYZ[\\]^_".chars().enumerate() {
            m.insert(c, (i as u8, 5));
        }
        // row 6: ` then a..o (columns 0..=15)
        for (i, c) in "`abcdefghijklmno".chars().enumerate() {
            m.insert(c, (i as u8, 6));
        }
        // row 7: p..z (columns 0..=10)
        for (i, c) in "pqrstuvwxyz".chars().enumerate() {
            m.insert(c, (i as u8, 7));
        }
        // full block, used for bars/fills
        m.insert('█', (2, 11));

        Tileset { char_tiles: m }
    }

    /// the (column, row) tile for a character, falling back to '?'.
    fn tile(&self, c: char) -> (u8, u8) {
        self.char_tiles
            .get(&c)
            .copied()
            .unwrap_or_else(|| self.char_tiles[&'?'])
    }

    /// the atlas uv rect (min, size) for a character's tile.
    pub fn uv(&self, c: char) -> ([f32; 2], [f32; 2]) {
        let (col, row) = self.tile(c);
        let u = col as f32 * TILE_PX / ATLAS_PIXELS;
        let v = row as f32 * TILE_PX / ATLAS_PIXELS;
        let size = TILE_PX / ATLAS_PIXELS;
        ([u, v], [size, size])
    }
}

impl Default for Tileset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_for_space_is_origin() {
        let ts = Tileset::new();
        let (min, size) = ts.uv(' ');
        assert_eq!(min, [0.0, 0.0]);
        assert_eq!(size, [9.0 / 144.0, 9.0 / 144.0]);
    }

    #[test]
    fn uv_for_capital_a() {
        // 'A' is column 1, row 4.
        let ts = Tileset::new();
        let (min, _) = ts.uv('A');
        assert_eq!(min, [9.0 / 144.0, 36.0 / 144.0]);
    }

    #[test]
    fn unknown_char_falls_back_to_question_mark() {
        let ts = Tileset::new();
        // '€' is not in the atlas; should match '?' (column 15, row 3).
        assert_eq!(ts.uv('€'), ts.uv('?'));
    }
}
