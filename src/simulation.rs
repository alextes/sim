use crate::entity::{Orbital, OrbitalEntity};
use crate::location::LocationMap;
use std::time::Duration;

pub const SIMULATION_UNIT_DURATION: Duration = Duration::from_millis(100);

pub fn update_orbital_entities(
    orbital_entities: &mut Vec<OrbitalEntity>,
    location_map: &mut LocationMap,
) {
    for entity in orbital_entities {
        let anchor_position = location_map.get(&entity.anchor_id).unwrap();
        entity.update_position(
            anchor_position.x,
            anchor_position.y,
            SIMULATION_UNIT_DURATION.as_secs_f64(),
        );
        location_map.add_entity(entity.id, entity.position.x, entity.position.y);
    }
}
