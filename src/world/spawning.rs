//! entity spawning logic.
use super::{Color, EntityId, Point, ShipInfo, World, MOON_COLORS, PLANET_COLORS, STAR_COLORS};
use crate::buildings::{EntityBuildings, GAS_GIANT_SLOTS, MOON_SLOTS, PLANET_SLOTS};
use crate::world::types::{
    CelestialBodyData, EntityType, GAS_GIANT_RESOURCES, PLANETARY_RESOURCES,
};
use rand::prelude::*;
use std::collections::HashMap;

/// create a static entity at a fixed point (e.g. a star).
pub fn spawn_star(world: &mut World, name: String, position: Point) -> EntityId {
    let id = world.next_entity_id;
    world.next_entity_id += 1;
    world.entities.push(id);
    world.entity_names.insert(id, name);
    world.entity_types.insert(id, EntityType::Star);
    world.render_glyphs.insert(id, 's');
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

/// create an orbiting entity (e.g. planet or moon) around an existing entity.
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
    world.entity_types.insert(id, EntityType::Planet);
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
    let chosen_resources = PLANETARY_RESOURCES
        .choose_multiple(&mut rng, 3)
        .cloned()
        .collect::<Vec<_>>();
    for resource in chosen_resources {
        yields.insert(resource, rng.random_range(50.0..150.0));
    }

    world.celestial_data.insert(
        id,
        CelestialBodyData {
            yields,
            ..Default::default()
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
    world.entity_types.insert(id, EntityType::Moon);
    world.render_glyphs.insert(id, 'm');
    let mut rng = rand::rng();
    let color = MOON_COLORS.iter().choose(&mut rng).unwrap();
    world.entity_colors.insert(id, *color);
    world
        .locations
        .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
    world.buildings.insert(id, EntityBuildings::new(MOON_SLOTS));

    let mut yields = HashMap::new();
    let chosen_resources = PLANETARY_RESOURCES
        .choose_multiple(&mut rng, 3)
        .cloned()
        .collect::<Vec<_>>();
    for resource in chosen_resources {
        yields.insert(resource, rng.random_range(20.0..80.0));
    }

    world.celestial_data.insert(
        id,
        CelestialBodyData {
            yields,
            ..Default::default()
        },
    );
    id
}

pub fn spawn_frigate(world: &mut World, name: String, position: Point) -> EntityId {
    let id = world.next_entity_id;
    world.next_entity_id += 1;
    world.entities.push(id);
    world.entity_names.insert(id, name);
    world.entity_types.insert(id, EntityType::Ship);
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

pub fn spawn_gas_giant(
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
    world.entity_types.insert(id, EntityType::GasGiant);
    world.render_glyphs.insert(id, 'g');
    let mut rng = rand::rng();
    let color = PLANET_COLORS.iter().choose(&mut rng).unwrap(); // Reuse planet colors for now
    world.entity_colors.insert(id, *color);
    world
        .locations
        .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
    world
        .buildings
        .insert(id, EntityBuildings::new(GAS_GIANT_SLOTS));

    let mut yields = HashMap::new();
    let chosen_resources = GAS_GIANT_RESOURCES
        .choose_multiple(&mut rng, 3)
        .cloned()
        .collect::<Vec<_>>();
    for resource in chosen_resources {
        yields.insert(resource, rng.random_range(80.0..200.0));
    }

    world.celestial_data.insert(
        id,
        CelestialBodyData {
            credits: 0.0,
            population: 0.0, // No population on gas giants
            yields,
            stocks: HashMap::new(),
        },
    );
    id
}
