use crate::location::{LocationMap, Point};
use crate::render::Viewport;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

pub fn handle_events(
    event_pump: &mut EventPump,
    location_viewport: &mut Viewport,
    entities: &Vec<u32>,
    location_map: &LocationMap,
    entity_focus_index: &mut usize,
    debug_enabled: &mut bool,
) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return false,
            Event::KeyDown {
                keycode: Some(Keycode::F4),
                ..
            } => {
                *debug_enabled = !*debug_enabled;
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
                *entity_focus_index = (*entity_focus_index + 1) % entities.len();
                let entity_id = entities[*entity_focus_index];
                let Point { x: ex, y: ey } =
                    location_map.get(&entity_id).cloned().unwrap_or_default();
                location_viewport.center_on_entity(ex, ey);
            }
            _ => {}
        }
    }
    true
}
