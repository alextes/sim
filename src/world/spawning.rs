//! entity spawning logic.
use super::{Color, EntityId, Point, ShipInfo, World, MOON_COLORS, PLANET_COLORS, STAR_COLORS};
use crate::buildings::{EntityBuildings, MOON_SLOTS, PLANET_SLOTS};
use crate::world::types::{CelestialBodyData, ResourceType};
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::HashMap;

/// Create a static entity at a fixed point (e.g. a star).
pub fn spawn_star(world: &mut World, name: String, position: Point) -> EntityId {
    let id = world.next_entity_id;
    world.next_entity_id += 1;
    world.entities.push(id);
    world.entity_names.insert(id, name);
    world.render_glyphs.insert(id, '*');
    let mut rng = rand::rng();
    let color = STAR_COLORS.iter().choose(&mut rng).unwrap();
    world.entity_colors.insert(id, *color);
    world.locations.add_static(id, position);
    world.buildings.insert(id, EntityBuildings::new(0));
    world
        .celestial_data
        .insert(id, CelestialBodyData::default());
    id
}

/// Create an orbiting entity (e.g. planet or moon) around an existing entity.
pub fn spawn_planet(
    world: &mut World,
    name: String,
    anchor: EntityId,
    radius: f64,
    initial_angle: f64,
    angular_velocity: f64,
) -> EntityId {
    let id = world.next_entity_id;
    world.next_entity_id += 1;
    world.entities.push(id);
    world.entity_names.insert(id, name);
    world.render_glyphs.insert(id, 'p');
    let mut rng = rand::rng();
    let color = PLANET_COLORS.iter().choose(&mut rng).unwrap();
    world.entity_colors.insert(id, *color);
    world
        .locations
        .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
    world
        .buildings
        .insert(id, EntityBuildings::new(PLANET_SLOTS));

    let mut yields = HashMap::new();
    yields.insert(ResourceType::Metal, rng.gen_range(0.5..1.5));
    yields.insert(ResourceType::Nobles, rng.gen_range(0.1..0.5));
    yields.insert(ResourceType::Organics, rng.gen_range(0.5..1.5));

    world.celestial_data.insert(
        id,
        CelestialBodyData {
            population: 1.0,
            yields,
        },
    );
    id
}

/// Create an orbiting moon, using the 'm' glyph.
pub fn spawn_moon(
    world: &mut World,
    name: String,
    anchor: EntityId,
    radius: f64,
    initial_angle: f64,
    angular_velocity: f64,
) -> EntityId {
    let id = world.next_entity_id;
    world.next_entity_id += 1;
    world.entities.push(id);
    world.entity_names.insert(id, name);
    world.render_glyphs.insert(id, 'm');
    let mut rng = rand::rng();
    let color = MOON_COLORS.iter().choose(&mut rng).unwrap();
    world.entity_colors.insert(id, *color);
    world
        .locations
        .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
    world.buildings.insert(id, EntityBuildings::new(MOON_SLOTS));

    let mut yields = HashMap::new();
    yields.insert(ResourceType::Metal, rng.gen_range(0.2..0.8));
    yields.insert(ResourceType::Nobles, rng.gen_range(0.3..0.8));
    yields.insert(ResourceType::Organics, rng.gen_range(0.1..0.4));

    world.celestial_data.insert(
        id,
        CelestialBodyData {
            population: 0.2,
            yields,
        },
    );
    id
}

pub fn spawn_frigate(world: &mut World, name: String, position: Point) -> EntityId {
    let id = world.next_entity_id;
    world.next_entity_id += 1;
    world.entities.push(id);
    world.entity_names.insert(id, name);
    world.render_glyphs.insert(id, 'f');
    // use gray for now
    world.entity_colors.insert(
        id,
        Color {
            r: 128,
            g: 128,
            b: 128,
        },
    );
    world
        .locations
        .add_mobile(id, (position.x as f64, position.y as f64).into());
    world.ships.insert(id, ShipInfo { speed: 5.0 }); // Default speed
    world.set_player_controlled(id);
    id
}
