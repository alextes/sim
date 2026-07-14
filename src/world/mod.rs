use std::collections::{HashMap, HashSet};

use crate::location::OrbitalParameters;
use crate::location::{LocationSystem, OrbitalInfo, Point};

use crate::command::Command;
use crate::infrastructure::EntityInfrastructure;
use crate::location::PointF64;
use std::collections::VecDeque;

use crate::ships::ShipType;
use crate::world::components::{Cargo, CivilianShipAI, MiningRoute};
use crate::world::types::{
    BodyProfile, CelestialBodyData, Color, EntityType, MOON_COLORS, PLANET_COLORS, STAR_COLORS,
};

mod civ_economy;
mod command_processing;
pub mod components;
mod lanes;
mod movement;
mod population;
mod resources;
pub mod spawning;
pub mod types;

use rand::rngs::StdRng;
use rand::SeedableRng;
pub use resources::ResourceSystem;
use tracing::debug;

// Entity identifiers for all game objects.
pub type EntityId = u32;

#[derive(Debug, Clone, Copy)]
pub struct ShipInfo {
    pub speed: f64,
    pub ship_type: ShipType,
}

/// the world's owned random number generator. wrapping `StdRng` lets `World`
/// keep deriving `Default` (StdRng has no `Default`); the default seeds from os
/// entropy for interactive play. reproducible runs reseed via `World::seed_rng`.
#[derive(Debug)]
pub struct WorldRng(pub StdRng);

impl Default for WorldRng {
    fn default() -> Self {
        WorldRng(StdRng::from_os_rng())
    }
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
    /// static character and development capacity for planet-like bodies.
    pub body_profiles: HashMap<EntityId, BodyProfile>,
    /// infrastructure for entities that support it.
    pub infrastructure: HashMap<EntityId, EntityInfrastructure>,
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
    /// optional user-defined mining routes per ship
    pub mining_routes: HashMap<EntityId, MiningRoute>,
    /// master switch for autonomous civilian ai behavior
    pub enable_civilian_ai: bool,
    /// world-owned rng driving generation, spawning, and civilian ai. seed it
    /// for reproducible runs (see `seed_rng`).
    pub(crate) rng: WorldRng,
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

    /// reseed the world's rng for reproducible runs. call before
    /// `populate_initial_galaxy` (and before any `update`) to get an identical
    /// world and simulation from the same seed.
    pub fn seed_rng(&mut self, seed: u64) {
        self.rng = WorldRng(StdRng::seed_from_u64(seed));
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
            &self.infrastructure,
            &mut self.celestial_data,
        );
        self.update_population(dt);
        self.update_ship_movement(dt);
        if self.enable_civilian_ai {
            self.update_civilian_economy(dt);
            self.update_civilian_ships(dt);
        } else {
            // even when ai is disabled, allow manual mining routes to operate using the same mechanics
            self.update_civilian_ships(dt);
        }
        self.process_ship_mining(dt);
        self.process_construction(dt);
        let sales_info = self.process_ship_sales();
        for (ship_id, total_value, home_base, cargo) in sales_info {
            let cargo_desc = cargo
                .iter()
                .map(|(s, a)| format!("{a:.2} {s}"))
                .collect::<Vec<String>>()
                .join(", ");
            debug!(
                "ship {} sold {:.2} credits worth of resources to {}: {}",
                self.get_entity_name(ship_id)
                    .unwrap_or_else(|| "unknown".to_string()),
                total_value,
                self.get_entity_name(home_base)
                    .unwrap_or_else(|| "unknown".to_string()),
                cargo_desc,
            );
        }
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

    fn process_construction(&mut self, dt: f64) {
        for infrastructure in self.infrastructure.values_mut() {
            infrastructure.process_construction(dt as f32);
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

    /// get the orbital parameters of an entity, if it has any.
    pub fn get_orbital_parameters(&self, entity: EntityId) -> Option<OrbitalParameters> {
        self.locations.get_orbital_parameters(entity)
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

    /// get the size of an entity for rendering purposes.
    pub fn get_render_size(&self, entity: EntityId) -> f64 {
        match self.get_entity_type(entity) {
            Some(EntityType::Star) => 3.0,
            Some(EntityType::GasGiant) => 1.5,
            Some(EntityType::Planet) => 1.0,
            Some(EntityType::Moon) => 0.5,
            Some(EntityType::Ship) => 0.4,
            None => 1.0,
        }
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

    /// player-controlled bodies shown in the planet overview, in stable order.
    pub fn owned_body_overview_entities(&self) -> Vec<EntityId> {
        let mut bodies: Vec<_> = self
            .entities
            .iter()
            .copied()
            .filter(|&entity| self.is_owned_overview_body(entity))
            .collect();

        bodies.sort_by_key(|&entity| {
            (
                self.find_star_for_entity(entity).unwrap_or(u32::MAX),
                overview_body_type_priority(self.get_entity_type(entity)),
                self.entity_names.get(&entity).cloned().unwrap_or_default(),
                entity,
            )
        });
        bodies
    }

    fn is_owned_overview_body(&self, entity: EntityId) -> bool {
        self.is_player_controlled(entity)
            && self.celestial_data.contains_key(&entity)
            && self.infrastructure.contains_key(&entity)
            && matches!(
                self.get_entity_type(entity),
                Some(EntityType::GasGiant | EntityType::Planet | EntityType::Moon)
            )
    }

    /// iterate over lane pairs (consumed by the star-lane overlay).
    pub fn iter_lanes(&self) -> impl Iterator<Item = &(EntityId, EntityId)> {
        self.lanes.iter()
    }

    /// set or clear a mining route for a ship. also updates the ship's ai home_base to the sell body when set.
    pub fn set_mining_route(&mut self, ship_id: EntityId, route: Option<MiningRoute>) {
        match route {
            Some(r) => {
                self.mining_routes.insert(ship_id, r);
                if let Some(ai) = self.civilian_ai.get_mut(&ship_id) {
                    ai.home_base = r.sell_body;
                    ai.state = crate::world::components::CivilianShipState::Idle;
                }
            }
            None => {
                self.mining_routes.remove(&ship_id);
            }
        }
    }

    /// compute a naive most-profitable mining route by scanning all bodies with yields and all bodies as buyers.
    pub fn compute_best_mining_route(&self) -> Option<MiningRoute> {
        let mut best: Option<(f64, MiningRoute)> = None;
        for (&source_id, data) in &self.celestial_data {
            if data.yields.is_empty() {
                continue;
            }
            for &sell_id in self.celestial_data.keys() {
                if sell_id == source_id {
                    continue;
                }
                for (&raw, &grade) in &data.yields {
                    let price = crate::world::resources::get_local_price(
                        self,
                        sell_id,
                        crate::world::types::Storable::Raw(raw),
                    );
                    let score = (grade as f64) * price;
                    let candidate = MiningRoute {
                        target_body: source_id,
                        resource: raw,
                        sell_body: sell_id,
                    };
                    match best {
                        Some((best_score, _)) if score <= best_score => {}
                        _ => best = Some((score, candidate)),
                    }
                }
            }
        }
        best.map(|(_, r)| r)
    }
}

fn overview_body_type_priority(entity_type: Option<EntityType>) -> u8 {
    match entity_type {
        Some(EntityType::GasGiant) => 0,
        Some(EntityType::Planet) => 1,
        Some(EntityType::Moon) => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;
    use crate::world::types::BodySize;

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

    #[test]
    fn body_size_capacity_uses_v1_defaults() {
        assert_eq!(BodySize::Tiny.capacity(), 4);
        assert_eq!(BodySize::Small.capacity(), 8);
        assert_eq!(BodySize::Medium.capacity(), 16);
        assert_eq!(BodySize::Large.capacity(), 28);
        assert_eq!(BodySize::Giant.capacity(), 48);
    }

    #[test]
    fn owned_body_overview_entities_filters_to_owned_celestial_bodies() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let planet_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 1.0);
        let moon_id = world.spawn_moon("moon".to_string(), planet_id, 2.0, 0.0, 1.0);
        let gas_id = world.spawn_gas_giant("jupiter".to_string(), star_id, 20.0, 0.0, 1.0);
        let _ship_id = world.spawn_frigate("frigate".to_string(), Point { x: 1, y: 1 });
        let _unowned_id = world.spawn_planet("mars".to_string(), star_id, 15.0, 0.0, 1.0);
        let missing_infrastructure_id =
            world.spawn_planet("venus".to_string(), star_id, 12.0, 0.0, 1.0);
        let missing_data_id = world.spawn_planet("mercury".to_string(), star_id, 8.0, 0.0, 1.0);

        for entity in [
            planet_id,
            moon_id,
            gas_id,
            missing_infrastructure_id,
            missing_data_id,
        ] {
            world.set_player_controlled(entity);
        }
        world.infrastructure.remove(&missing_infrastructure_id);
        world.celestial_data.remove(&missing_data_id);

        let bodies = world.owned_body_overview_entities();

        assert_eq!(bodies, vec![gas_id, planet_id, moon_id]);
    }

    #[test]
    fn owned_body_overview_entities_sorts_deterministically() {
        let mut world = World::default();
        let first_star = world.spawn_star("a-star".to_string(), Point { x: 0, y: 0 });
        let second_star = world.spawn_star("b-star".to_string(), Point { x: 100, y: 0 });
        let first_planet_z = world.spawn_planet("zeta".to_string(), first_star, 10.0, 0.0, 1.0);
        let first_planet_a = world.spawn_planet("alpha".to_string(), first_star, 12.0, 0.0, 1.0);
        let first_moon = world.spawn_moon("moon".to_string(), first_planet_z, 2.0, 0.0, 1.0);
        let first_gas = world.spawn_gas_giant("giant".to_string(), first_star, 20.0, 0.0, 1.0);
        let second_planet = world.spawn_planet("other".to_string(), second_star, 10.0, 0.0, 1.0);

        for entity in [
            first_planet_z,
            first_planet_a,
            first_moon,
            first_gas,
            second_planet,
        ] {
            world.set_player_controlled(entity);
        }

        let bodies = world.owned_body_overview_entities();

        assert_eq!(
            bodies,
            vec![
                first_gas,
                first_planet_a,
                first_planet_z,
                first_moon,
                second_planet
            ]
        );
    }

    /// full-sim reproducibility: with the world-owned rng seeded, generation
    /// plus many ticks of the civilian economy (which selects mining targets
    /// and places built ships randomly) must produce bit-identical results.
    #[test]
    fn same_seed_produces_identical_simulation_run() {
        fn run(seed: u64) -> Vec<String> {
            let mut world = World::default();
            world.seed_rng(seed);
            crate::map_generation::populate_initial_galaxy(&mut world);
            world.enable_civilian_ai = true;

            // give the civilian economy something to do: mining ships homed at earth.
            let earth = world
                .iter_entities()
                .find(|&id| world.get_entity_name(id).as_deref() == Some("earth"))
                .expect("earth exists after generation");
            for i in 0..3 {
                world.spawn_mining_ship(format!("miner-{i}"), Point { x: 5 + i, y: 5 }, earth);
            }

            for _ in 0..300 {
                world.update(0.1, 0);
            }

            // fingerprint the parts the rng drives: ship states/positions and credits.
            let mut lines: Vec<String> = world
                .civilian_ai
                .keys()
                .map(|&id| {
                    let pos = world
                        .get_location_f64(id)
                        .unwrap_or(crate::location::PointF64 { x: 0.0, y: 0.0 });
                    format!(
                        "{id}:{:?}:{:.4},{:.4}",
                        world.civilian_ai[&id].state, pos.x, pos.y
                    )
                })
                .collect();
            lines.sort();
            lines.push(format!("credits:{:.4}", world.player_credits));
            lines
        }

        assert_eq!(run(777), run(777));
    }
}
