mod tileset;

use std::collections::HashMap;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::entity::EntityId;
use crate::location::LocationMap;
use crate::{colors, EntityType, Point, Viewport};

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
            .get(entity_id)
            .expect("expect entity type to be stored for entity id");

        let renderable = Renderable {
            x: x as u8,
            y: y as u8,
            tileset_rect: entity_type.into(),
            color: colors::BLUE,
        };

        render_tile(canvas, tiles_texture, &renderable);
    }
}
