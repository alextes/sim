use std::collections::{HashMap, HashSet};

use crate::location::{LocationSystem, OrbitalInfo, Point};

use crate::buildings::EntityBuildings;
use crate::command::Command;
use crate::location::PointF64;
use std::collections::VecDeque;

use crate::world::components::{Cargo, CivilianShipAI};
use crate::world::types::{
    CelestialBodyData, Color, EntityType, MOON_COLORS, PLANET_COLORS, STAR_COLORS,
};

mod civ_economy;
mod command_processing;
mod components;
mod lanes;
mod movement;
mod population;
mod resources;
pub mod spawning;
pub mod types;

pub use resources::ResourceSystem;

// Entity identifiers for all game objects.
pub type EntityId = u32;

#[derive(Debug, Clone, Copy)]
pub struct ShipInfo {
    pub speed: f64,
}

#[derive(Debug, Default)]
pub struct World {
    pub(crate) next_entity_id: EntityId,
    /// ordered list of all entity IDs
    pub entities: Vec<EntityId>,
    /// entity types
    pub(crate) entity_types: HashMap<EntityId, EntityType>,
    /// glyphs to use when rendering each entity
    pub(crate) render_glyphs: HashMap<EntityId, char>,
    /// colors to use when rendering each entity
    pub(crate) entity_colors: HashMap<EntityId, Color>,
    /// human-readable names for entities
    pub(crate) entity_names: HashMap<EntityId, String>,
    /// location system managing static and orbital positions
    pub(crate) locations: LocationSystem,
    /// Global resource counters for the player
    pub resources: ResourceSystem,
    /// Data for celestial bodies (population, yields, etc.)
    pub celestial_data: HashMap<EntityId, CelestialBodyData>,
    /// Building slots for entities that support them
    pub buildings: HashMap<EntityId, EntityBuildings>,
    /// visual-only star lanes between entities
    pub lanes: Vec<(EntityId, EntityId)>,
    /// set of entities controlled by the player
    pub player_controlled: HashSet<EntityId>,
    /// Ships
    pub ships: HashMap<EntityId, ShipInfo>,
    /// ship move orders
    pub move_orders: HashMap<EntityId, PointF64>,
    /// command queue
    pub command_queue: VecDeque<Command>,
    /// player/state credits.
    pub player_credits: f64,
    /// cargo holds for entities that can carry resources.
    pub cargo: HashMap<EntityId, Cargo>,
    /// ai state for civilian ships.
    pub civilian_ai: HashMap<EntityId, CivilianShipAI>,
}

impl World {
    /// Create a static entity at a fixed point (e.g. a star).
    pub fn spawn_star(&mut self, name: String, position: Point) -> EntityId {
        spawning::spawn_star(self, name, position)
    }

    /// Create an orbiting entity (e.g. planet or moon) around an existing entity.
    pub fn spawn_planet(
        &mut self,
        name: String,
        anchor: EntityId,
        radius: f64,
        initial_angle: f64,
        angular_velocity: f64,
    ) -> EntityId {
        spawning::spawn_planet(self, name, anchor, radius, initial_angle, angular_velocity)
    }

    pub fn spawn_gas_giant(
        &mut self,
        name: String,
        anchor: EntityId,
        radius: f64,
        initial_angle: f64,
        angular_velocity: f64,
    ) -> EntityId {
        spawning::spawn_gas_giant(self, name, anchor, radius, initial_angle, angular_velocity)
    }

    /// Create an orbiting moon, using the 'm' glyph.
    pub fn spawn_moon(
        &mut self,
        name: String,
        anchor: EntityId,
        radius: f64,
        initial_angle: f64,
        angular_velocity: f64,
    ) -> EntityId {
        spawning::spawn_moon(self, name, anchor, radius, initial_angle, angular_velocity)
    }

    pub fn spawn_mining_ship(
        &mut self,
        name: String,
        position: Point,
        home_base_id: EntityId,
    ) -> EntityId {
        spawning::spawn_mining_ship(self, name, position, home_base_id)
    }

    pub fn spawn_frigate(&mut self, name: String, position: Point) -> EntityId {
        spawning::spawn_frigate(self, name, position)
    }

    /// adds a command to the world's command queue.
    pub fn add_command(&mut self, command: Command) {
        self.command_queue.push_back(command);
    }

    /// sets a move order for a ship.
    pub fn set_move_order(&mut self, ship_id: EntityId, destination: PointF64) {
        if self.ships.contains_key(&ship_id) {
            self.move_orders.insert(ship_id, destination);
        }
    }

    /// advance all orbiters by dt_seconds, updating their stored positions.
    /// also handles periodic resource generation based on simulation ticks.
    pub fn update(&mut self, dt: f64, _current_tick: u64) {
        self.process_commands();
        self.locations.update(dt);
        self.resources.update(
            dt,
            &self.entity_types,
            &self.buildings,
            &mut self.celestial_data,
        );
        self.update_population(dt);
        self.update_ship_movement(dt);
        self.update_civilian_economy(dt);
        self.update_civilian_ships(dt);
    }

    /// calculates the maximum radius of a star system, considering planets and their moons.
    pub fn get_system_radius(&self, star_id: EntityId) -> f64 {
        let mut max_radius = 0.0;

        // find all direct children of the star (planets/gas giants)
        for (orbiter_id, orbital_info) in self.locations.iter_orbitals() {
            if orbital_info.anchor == star_id {
                // this is a planet or gas giant orbiting the star.
                let mut planet_system_radius = orbital_info.radius;

                // now find moons of this planet.
                let mut max_moon_radius = 0.0;
                for (_moon_id, moon_orbital_info) in self.locations.iter_orbitals() {
                    if moon_orbital_info.anchor == orbiter_id {
                        // this is a moon of the current planet.
                        if moon_orbital_info.radius > max_moon_radius {
                            max_moon_radius = moon_orbital_info.radius;
                        }
                    }
                }

                planet_system_radius += max_moon_radius;

                if planet_system_radius > max_radius {
                    max_radius = planet_system_radius;
                }
            }
        }

        // add a small buffer to the radius so lanes don't end exactly on the orbit.
        // if no orbitals, give it a small default radius.
        if max_radius == 0.0 {
            2.0 // default radius for stars with no planets.
        } else {
            max_radius * 1.2 // 20% buffer
        }
    }

    fn find_star_for_entity(&self, entity_id: EntityId) -> Option<EntityId> {
        if let Some(EntityType::Star) = self.get_entity_type(entity_id) {
            return Some(entity_id);
        }

        // build a map of child -> parent anchor to traverse up the hierarchy
        let anchor_map: HashMap<EntityId, EntityId> = self
            .iter_orbitals()
            .map(|(id, info)| (id, info.anchor))
            .collect();

        let mut current_id = entity_id;
        // loop up to 10 times to prevent infinite cycles in case of data errors
        for _ in 0..10 {
            if let Some(&anchor_id) = anchor_map.get(&current_id) {
                if let Some(EntityType::Star) = self.get_entity_type(anchor_id) {
                    return Some(anchor_id);
                }
                current_id = anchor_id;
            } else {
                // not an orbital, so we can't find its star
                return None;
            }
        }
        None
    }

    pub fn iter_orbitals(&self) -> impl Iterator<Item = (EntityId, OrbitalInfo)> + '_ {
        self.locations.iter_orbitals()
    }

    /// return the current universal position of an entity.
    pub fn get_location(&self, entity: EntityId) -> Option<Point> {
        self.locations.get_location(entity)
    }

    /// return the current universal f64 position of an entity.
    pub fn get_location_f64(&self, entity: EntityId) -> Option<PointF64> {
        self.locations.get_location_f64(entity)
    }

    /// iterate over all entity IDs in creation order.
    pub fn iter_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().cloned()
    }

    /// get the human-readable name of an entity, if any.
    pub fn get_entity_name(&self, entity: EntityId) -> Option<String> {
        self.entity_names.get(&entity).cloned()
    }

    /// get the type of an entity.
    pub fn get_entity_type(&self, entity: EntityId) -> Option<EntityType> {
        self.entity_types.get(&entity).copied()
    }

    /// get the glyph used for rendering this entity.
    pub fn get_render_glyph(&self, entity: EntityId) -> char {
        self.render_glyphs.get(&entity).copied().unwrap_or('?')
    }

    /// get the color used for rendering this entity.
    pub fn get_entity_color(&self, entity: EntityId) -> Option<Color> {
        self.entity_colors.get(&entity).copied()
    }

    /// set an entity as player controlled.
    pub fn set_player_controlled(&mut self, entity: EntityId) {
        self.player_controlled.insert(entity);
    }

    /// check if an entity is player controlled.
    pub fn is_player_controlled(&self, entity: EntityId) -> bool {
        self.player_controlled.contains(&entity)
    }

    /// iterate over lane pairs
    pub fn iter_lanes(&self) -> impl Iterator<Item = &(EntityId, EntityId)> {
        self.lanes.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;

    #[test]
    fn test_spawn_frigate() {
        let mut world = World::default();
        let position = Point { x: 1, y: 2 };
        let frigate_id = world.spawn_frigate("test_frigate".to_string(), position);

        // Check that the entity was created
        assert!(world.entities.contains(&frigate_id));
        assert_eq!(
            world.get_entity_name(frigate_id),
            Some("test_frigate".to_string())
        );
        assert_eq!(world.get_render_glyph(frigate_id), 'f');
        assert!(world.get_entity_color(frigate_id).is_some());
        assert_eq!(world.get_location(frigate_id), Some(position));
        assert!(world.is_player_controlled(frigate_id));

        // Check that ship-specific data was added
        assert!(world.ships.contains_key(&frigate_id));
        if let Some(ship_info) = world.ships.get(&frigate_id) {
            assert_eq!(ship_info.speed, 5.0);
        }
    }
}
