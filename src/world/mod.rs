use std::collections::HashMap;

use crate::location::{LocationSystem, OrbitalInfo, Point};

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

    pub fn iter_orbitals(&self) -> impl Iterator<Item = (EntityId, OrbitalInfo)> + '_ {
        self.locations.iter_orbitals()
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

    /// generate visual star lanes connecting stars.
    /// attempts to connect each star to `target_connections_per_star` closest neighbors,
    /// while ensuring each star has at least `minimum_connections_per_star`.
    pub fn generate_star_lanes(&mut self) {
        use std::collections::{HashMap, HashSet};

        const TARGET_CONNECTIONS_PER_STAR: usize = 4;
        const MINIMUM_CONNECTIONS_PER_STAR: usize = 2;

        // helper to get ordered pair to avoid duplicates
        let add_lane_fn = |a: EntityId,
                           b: EntityId,
                           set: &mut HashSet<(EntityId, EntityId)>,
                           lanes_vec: &mut Vec<(EntityId, EntityId)>| {
            if a == b {
                return false;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            if set.insert(key) {
                lanes_vec.push(key);
                return true;
            }
            false
        };

        let mut lane_set: HashSet<(EntityId, EntityId)> = HashSet::new();
        self.lanes.clear();

        let star_ids: Vec<EntityId> = self
            .render_glyphs
            .iter()
            .filter_map(|(&id, &glyph)| if glyph == '*' { Some(id) } else { None })
            .collect();

        if star_ids.len() < 2 {
            return; // not enough stars to form lanes
        }

        // first pass: connect to target_connections_per_star closest
        for &current_star_id in &star_ids {
            let mut neighbors_by_distance: Vec<(i32, EntityId)> = Vec::new();
            if let Some(p_current) = self.get_location(current_star_id) {
                for &other_star_id in &star_ids {
                    if current_star_id == other_star_id {
                        continue;
                    }
                    if let Some(p_other) = self.get_location(other_star_id) {
                        let dx = p_current.x - p_other.x;
                        let dy = p_current.y - p_other.y;
                        neighbors_by_distance.push((dx * dx + dy * dy, other_star_id));
                    }
                }
            }
            neighbors_by_distance.sort_by_key(|k| k.0); // sort by distance squared

            let mut connections_made_for_current_star = 0;
            for &(_dist_sq, neighbor_id) in &neighbors_by_distance {
                if connections_made_for_current_star >= TARGET_CONNECTIONS_PER_STAR {
                    break;
                }
                if add_lane_fn(current_star_id, neighbor_id, &mut lane_set, &mut self.lanes) {
                    connections_made_for_current_star += 1;
                }
            }
        }

        // second pass: ensure minimum_connections_per_star
        let mut star_connection_counts: HashMap<EntityId, usize> = HashMap::new();
        for &(a, b) in &self.lanes {
            *star_connection_counts.entry(a).or_insert(0) += 1;
            *star_connection_counts.entry(b).or_insert(0) += 1;
        }

        for &current_star_id in &star_ids {
            let mut current_connections = star_connection_counts
                .get(&current_star_id)
                .copied()
                .unwrap_or(0);
            if current_connections < MINIMUM_CONNECTIONS_PER_STAR {
                let mut neighbors_by_distance: Vec<(i32, EntityId)> = Vec::new();
                if let Some(p_current) = self.get_location(current_star_id) {
                    for &other_star_id in &star_ids {
                        if current_star_id == other_star_id {
                            continue;
                        }
                        // only consider connecting if not already connected
                        let key = if current_star_id < other_star_id {
                            (current_star_id, other_star_id)
                        } else {
                            (other_star_id, current_star_id)
                        };
                        if lane_set.contains(&key) {
                            continue;
                        }
                        if let Some(p_other) = self.get_location(other_star_id) {
                            let dx = p_current.x - p_other.x;
                            let dy = p_current.y - p_other.y;
                            neighbors_by_distance.push((dx * dx + dy * dy, other_star_id));
                        }
                    }
                }
                neighbors_by_distance.sort_by_key(|k| k.0);

                for &(_dist_sq, neighbor_id) in &neighbors_by_distance {
                    if current_connections >= MINIMUM_CONNECTIONS_PER_STAR {
                        break;
                    }
                    if add_lane_fn(current_star_id, neighbor_id, &mut lane_set, &mut self.lanes) {
                        current_connections += 1;
                        // also update counts for the other star involved in this new lane
                        *star_connection_counts.entry(neighbor_id).or_insert(0) += 1;
                    }
                }
            }
        }

        // --- Pruning pass for overly similar lanes ---
        // This pass iterates, removing one lane at a time if it's too similar to another
        // from the same star, is the longer of the pair, and its removal is safe.
        const ANGLE_SIMILARITY_THRESHOLD_RADIANS: f64 = 15.0 * std::f64::consts::TAU / 360.0; // 15 degrees

        let mut current_lanes_as_set: HashSet<(EntityId, EntityId)> =
            self.lanes.iter().cloned().collect();

        loop {
            let mut made_change_this_pass = false;

            // Recalculate connection counts based on the current set of lanes for this pass
            let mut temp_star_connection_counts: HashMap<EntityId, usize> = HashMap::new();
            for &(s1, s2) in &current_lanes_as_set {
                *temp_star_connection_counts.entry(s1).or_insert(0) += 1;
                *temp_star_connection_counts.entry(s2).or_insert(0) += 1;
            }

            let mut lane_to_remove_candidate: Option<(EntityId, EntityId)> = None;

            'star_loop: for &current_star_id in &star_ids {
                // star_ids is from earlier in the function
                let p_current = match self.get_location(current_star_id) {
                    Some(p) => p,
                    None => continue, // Should not happen for a valid star_id
                };

                let mut lanes_from_this_star: Vec<(f64, EntityId, i32)> = Vec::new(); // (angle, neighbor_id, dist_sq)

                for &(s1_lane, s2_lane) in &current_lanes_as_set {
                    let neighbor_id_in_lane = if s1_lane == current_star_id {
                        s2_lane
                    } else if s2_lane == current_star_id {
                        s1_lane
                    } else {
                        continue; // This lane is not connected to current_star_id
                    };

                    let p_neighbor = match self.get_location(neighbor_id_in_lane) {
                        Some(p) => p,
                        None => continue, // Should not happen for a star connected by a lane
                    };

                    let dy = (p_neighbor.y - p_current.y) as f64;
                    let dx = (p_neighbor.x - p_current.x) as f64;
                    let angle = dy.atan2(dx); // angle in radians from -PI to PI

                    // Calculate squared distance
                    let dist_sq =
                        (p_neighbor.x - p_current.x).pow(2) + (p_neighbor.y - p_current.y).pow(2);
                    lanes_from_this_star.push((angle, neighbor_id_in_lane, dist_sq));
                }

                if lanes_from_this_star.len() < 2 {
                    // Need at least two lanes from this star to compare
                    continue;
                }

                // Sort lanes by angle to easily find adjacent similar lanes
                lanes_from_this_star
                    .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                for i in 0..lanes_from_this_star.len() {
                    let (angle1, neighbor1_id, dist_sq1) = lanes_from_this_star[i];
                    // Compare with the next lane in the sorted list (with wraparound)
                    let (angle2, neighbor2_id, dist_sq2) =
                        lanes_from_this_star[(i + 1) % lanes_from_this_star.len()];

                    if neighbor1_id == neighbor2_id {
                        continue;
                    } // Should be impossible if lanes are unique

                    let mut delta_angle = (angle1 - angle2).abs();
                    // Adjust for angles spanning across -PI/PI boundary (e.g. -170 deg and 170 deg)
                    if delta_angle > std::f64::consts::PI {
                        delta_angle = std::f64::consts::TAU - delta_angle;
                    }

                    if delta_angle < ANGLE_SIMILARITY_THRESHOLD_RADIANS {
                        // These two lanes are "too similar". Determine which one is longer.
                        let victim_neighbor_id = if dist_sq1 >= dist_sq2 {
                            // Lane to neighbor1 is longer or equal
                            neighbor1_id
                        } else {
                            // Lane to neighbor2 is longer
                            neighbor2_id
                        };

                        // Form the canonical tuple for the lane to potentially remove
                        let potential_lane_to_remove = if current_star_id < victim_neighbor_id {
                            (current_star_id, victim_neighbor_id)
                        } else {
                            (victim_neighbor_id, current_star_id)
                        };

                        // Check if removing this lane is safe for both stars involved
                        let count_current_star = temp_star_connection_counts
                            .get(&current_star_id)
                            .copied()
                            .unwrap_or(0);
                        let count_victim_neighbor = temp_star_connection_counts
                            .get(&victim_neighbor_id)
                            .copied()
                            .unwrap_or(0);

                        // Only remove if both stars will remain above MINIMUM_CONNECTIONS_PER_STAR
                        if count_current_star > MINIMUM_CONNECTIONS_PER_STAR
                            && count_victim_neighbor > MINIMUM_CONNECTIONS_PER_STAR
                        {
                            lane_to_remove_candidate = Some(potential_lane_to_remove);
                            break 'star_loop; // Found a candidate for this pass, restart evaluation
                        }
                    }
                }
            } // End 'star_loop

            if let Some(lane_to_remove) = lane_to_remove_candidate {
                if current_lanes_as_set.remove(&lane_to_remove) {
                    made_change_this_pass = true; // A lane was removed, so we need to re-evaluate
                }
            }

            if !made_change_this_pass {
                break; // No changes in this full pass over all stars, so pruning is stable
            }
        } // End loop

        self.lanes = current_lanes_as_set.into_iter().collect();
    }

    /// iterate over lane pairs
    pub fn iter_lanes(&self) -> impl Iterator<Item = &(EntityId, EntityId)> {
        self.lanes.iter()
    }
}
