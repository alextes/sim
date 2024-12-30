use crate::entity::{EntityType, EntityTypeMap, OrbitalEntity};
use crate::location::{LocationMap, Point};
use std::collections::HashMap;

pub fn initialize_entities() -> (Vec<u32>, EntityTypeMap, LocationMap, Vec<OrbitalEntity>) {
    let mut entities = vec![];
    let mut entity_type_map: EntityTypeMap = HashMap::new();
    let mut location_map = LocationMap::new();

    // Add Sol
    let sol_id = 0;
    entities.push(sol_id);
    entity_type_map.insert(sol_id, EntityType::Star);
    location_map.add_entity(sol_id, 0, 0);

    // Add Earth
    let earth_id = 1;
    entities.push(earth_id);
    entity_type_map.insert(earth_id, EntityType::Planet);
    location_map.add_entity(earth_id, -16, 0);

    // Add Moon
    let moon_id = 2;
    entities.push(moon_id);
    entity_type_map.insert(moon_id, EntityType::Moon);
    location_map.add_entity(moon_id, -16, 2);

    let orbital_entities = vec![
        OrbitalEntity {
            id: earth_id,
            anchor_id: sol_id,
            radius: 16.0,
            angle: 0.0,
            angular_velocity: 0.1,
            position: Point { x: 0, y: 0 },
        },
        OrbitalEntity {
            id: moon_id,
            anchor_id: earth_id,
            radius: 2.0,
            angle: 0.0,
            angular_velocity: 0.2,
            position: Point { x: 0, y: 0 },
        },
    ];

    (entities, entity_type_map, location_map, orbital_entities)
}
