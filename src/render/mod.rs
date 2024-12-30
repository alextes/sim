mod tileset;

use std::collections::HashMap;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::entity::EntityId;
use crate::location::{LocationMap, Point};
use crate::{colors, EntityType};

pub const TILE_PIXEL_WIDTH: u8 = 9;

pub struct Renderable {
    pub color: Color,
    pub tileset_rect: Rect,
    pub x: u8,
    pub y: u8,
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
        .draw_rect(tileset::make_multi_tile_rect(
            (64 - text.len()) as u8,
            0,
            text.len() as u8,
            1,
        ))
        .unwrap();

    tiles_texture.set_color_mod(foreground_color.r, foreground_color.g, foreground_color.b);

    let chars = text.chars();

    for (i, char) in chars.enumerate() {
        canvas
            .copy(
                tiles_texture,
                Some(tileset::rect_from_char(char)),
                Some(tileset::make_tile_rect(
                    (64 - text.len() + i).try_into().unwrap(),
                    0,
                )),
            )
            .unwrap();
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

pub fn render_viewport(
    canvas: &mut Canvas<Window>,
    tiles_texture: &mut Texture<'_>,
    entity_type_map: &HashMap<EntityId, EntityType>,
    location_map: &LocationMap,
    viewport: &Viewport,
) {
    let visible_entities = location_map.iter().filter(|(_, location)| {
        location.x >= viewport.min_x()
            && location.x <= viewport.max_x()
            && location.y >= viewport.min_y()
            && location.y <= viewport.max_y()
    });

    for (entity_id, point) in visible_entities {
        let translated_location = LocationMap::translate_location(point, viewport);

        let entity_type = entity_type_map
            .get(entity_id)
            .expect("expect entity type to be stored for entity id");

        let renderable = Renderable {
            x: translated_location.x as u8,
            y: translated_location.y as u8,
            tileset_rect: entity_type.into(),
            color: colors::BLUE,
        };

        render_tile(canvas, tiles_texture, &renderable);
    }
}

pub struct Viewport {
    /// Specifies which universe coordinate the top left corner of the viewport is centered on.
    pub anchor: Point,
    /// Specifies how far we're zoomed in on the universe, and therefore how many tiles are visible.
    pub zoom: f64,
    pub width: u32,
    pub height: u32,
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

impl Viewport {
    pub fn min_x(&self) -> i32 {
        self.anchor.x
    }

    pub fn max_x(&self) -> i32 {
        self.anchor.x + self.width as i32
    }

    pub fn min_y(&self) -> i32 {
        self.anchor.y
    }

    pub fn max_y(&self) -> i32 {
        self.anchor.y + self.height as i32
    }

    pub fn center_on_entity(&mut self, x: i32, y: i32) {
        self.anchor.x = x - (self.width as i32 / 2);
        self.anchor.y = y - (self.height as i32 / 2);
    }
}
