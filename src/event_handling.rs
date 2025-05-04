use crate::render::{Viewport, TILE_PIXEL_WIDTH};
use crate::world::World;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

pub enum Signal {
    Quit,
    Continue,
}

pub fn handle_events(
    event_pump: &mut EventPump,
    location_viewport: &mut Viewport,
    world: &mut World,
    entity_focus_index: &mut usize,
    debug_enabled: &mut bool,
    track_mode: &mut bool,
) -> Signal {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return Signal::Quit,
            Event::KeyDown {
                keycode: Some(Keycode::F4),
                ..
            } => {
                *debug_enabled = !*debug_enabled;
            }
            Event::KeyDown {
                keycode: Some(Keycode::F),
                ..
            } => {
                *track_mode = !*track_mode;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                location_viewport.anchor.y -= 1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                location_viewport.anchor.y += 1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                location_viewport.anchor.x -= 1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                location_viewport.anchor.x += 1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Tab),
                ..
            } => {
                *entity_focus_index = (*entity_focus_index + 1) % world.entities.len();
                let entity_id = world.entities[*entity_focus_index];
                let entity_location = world.get_location(entity_id).unwrap();
                location_viewport.center_on_entity(entity_location.x, entity_location.y);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Plus),
                ..
            } => {
                location_viewport.zoom_in();
            }
            Event::KeyDown {
                keycode: Some(Keycode::Equals),
                keymod,
                ..
            } if keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD)
                || keymod.contains(sdl2::keyboard::Mod::RSHIFTMOD) =>
            {
                location_viewport.zoom_in();
            }
            Event::KeyDown {
                keycode: Some(Keycode::Minus),
                ..
            } => {
                location_viewport.zoom_out();
            }
            Event::MouseButtonDown { x, y, .. } => {
                let tile_pixel = (TILE_PIXEL_WIDTH as f64 * location_viewport.zoom) as i32;
                if tile_pixel > 0 {
                    let tile_x = x / tile_pixel;
                    let tile_y = y / tile_pixel;
                    let half_w = location_viewport.width as i32 / 2;
                    let half_h = location_viewport.height as i32 / 2;
                    let world_x = location_viewport.anchor.x - half_w + tile_x;
                    let world_y = location_viewport.anchor.y - half_h + tile_y;

                    if let Some((idx, _)) = world.iter_entities().enumerate().find(|(_, id)| {
                        if let Some(loc) = world.get_location(*id) {
                            loc.x == world_x && loc.y == world_y
                        } else {
                            false
                        }
                    }) {
                        *entity_focus_index = idx;
                    }
                }
            }
            _ => {}
        }
    }
    Signal::Continue
}
