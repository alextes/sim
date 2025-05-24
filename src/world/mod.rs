use std::collections::HashMap;

use crate::location::{LocationSystem, Point};

use crate::buildings::EntityBuildings;

mod resources;

pub use resources::ResourceSystem;

// Entity identifiers for all game objects.
pub type EntityId = u32;

#[derive(Debug, Default)]
pub struct World {
    next_entity_id: EntityId,
    /// ordered list of all entity IDs
    pub entities: Vec<EntityId>,
    /// glyphs to use when rendering each entity
    render_glyphs: HashMap<EntityId, char>,
    /// human-readable names for entities
    entity_names: HashMap<EntityId, String>,
    /// location system managing static and orbital positions
    locations: LocationSystem,
    /// Global resource counters for the player
    pub resources: ResourceSystem,
    /// Building slots for entities that support them
    pub buildings: HashMap<EntityId, EntityBuildings>,
    /// visual-only star lanes between entities
    pub lanes: Vec<(EntityId, EntityId)>,
}

impl World {
    /// Create a static entity at a fixed point (e.g. a star).
    pub fn spawn_star(&mut self, name: String, position: Point) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(id);
        self.entity_names.insert(id, name);
        self.render_glyphs.insert(id, '*');
        self.locations.add_static(id, position);
        self.buildings.insert(id, EntityBuildings::new(false));
        id
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
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(id);
        self.entity_names.insert(id, name);
        self.render_glyphs.insert(id, 'p');
        self.locations
            .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
        self.buildings.insert(id, EntityBuildings::new(true));
        id
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
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(id);
        self.entity_names.insert(id, name);
        self.render_glyphs.insert(id, 'm');
        self.locations
            .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
        self.buildings.insert(id, EntityBuildings::new(true));
        id
    }

    /// advance all orbiters by dt_seconds, updating their stored positions.
    /// also handles periodic resource generation based on simulation ticks.
    pub fn update(&mut self, dt_seconds: f64, _current_tick: u64) {
        self.locations.update(dt_seconds);

        // delegate resource updates to the ResourceSystem
        self.resources.update(dt_seconds, &self.buildings);
    }

    /// return the current universal position of an entity.
    pub fn get_location(&self, entity: EntityId) -> Option<Point> {
        self.locations.get_location(entity)
    }

    /// iterate over all entity IDs in creation order.
    pub fn iter_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().cloned()
    }

    /// get the human-readable name of an entity, if any.
    pub fn get_entity_name(&self, entity: EntityId) -> Option<String> {
        self.entity_names.get(&entity).cloned()
    }

    /// get the glyph used for rendering this entity.
    pub fn get_render_glyph(&self, entity: EntityId) -> char {
        self.render_glyphs.get(&entity).copied().unwrap_or('?')
    }

    /// generate visual star lanes connecting stars within a given maximum distance.
    /// ensures each star has at least one connection (to its nearest neighbor) if none are within the threshold.
    pub fn generate_star_lanes(&mut self, max_distance: i32) {
        use std::collections::HashSet;

        // helper to get ordered pair to avoid duplicates
        let add_lane = |a: EntityId,
                        b: EntityId,
                        set: &mut HashSet<(EntityId, EntityId)>,
                        lanes: &mut Vec<(EntityId, EntityId)>| {
            if a == b {
                return;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            if set.insert(key) {
                lanes.push(key);
            }
        };

        let mut lane_set: HashSet<(EntityId, EntityId)> = HashSet::new();
        self.lanes.clear();

        // collect star ids (we assume every spawn_star entity is a star; others planets/moons etc.)
        // heuristic: glyph '*' for star used in spawn_star
        let star_ids: Vec<EntityId> = self
            .render_glyphs
            .iter()
            .filter_map(|(&id, &glyph)| if glyph == '*' { Some(id) } else { None })
            .collect();

        // generate lanes within threshold
        for i in 0..star_ids.len() {
            for j in i + 1..star_ids.len() {
                let a = star_ids[i];
                let b = star_ids[j];
                if let (Some(pa), Some(pb)) = (self.get_location(a), self.get_location(b)) {
                    let dx = pa.x - pb.x;
                    let dy = pa.y - pb.y;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= max_distance * max_distance {
                        add_lane(a, b, &mut lane_set, &mut self.lanes);
                    }
                }
            }
        }

        // ensure each star has at least one lane
        for &star in &star_ids {
            let has_lane = self.lanes.iter().any(|&(a, b)| a == star || b == star);
            if !has_lane {
                // find nearest other star
                let mut nearest: Option<(EntityId, i32)> = None; // (id, dist_sq)
                for &other in &star_ids {
                    if other == star {
                        continue;
                    }
                    if let (Some(pa), Some(pb)) =
                        (self.get_location(star), self.get_location(other))
                    {
                        let dx = pa.x - pb.x;
                        let dy = pa.y - pb.y;
                        let dist_sq = dx * dx + dy * dy;
                        if nearest.is_none() || dist_sq < nearest.unwrap().1 {
                            nearest = Some((other, dist_sq));
                        }
                    }
                }
                if let Some((other, _)) = nearest {
                    add_lane(star, other, &mut lane_set, &mut self.lanes);
                }
            }
        }
    }

    /// iterate over lane pairs
    pub fn iter_lanes(&self) -> impl Iterator<Item = &(EntityId, EntityId)> {
        self.lanes.iter()
    }
}
