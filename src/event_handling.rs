use crate::render::Viewport;
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
            _ => {}
        }
    }
    Signal::Continue
}
