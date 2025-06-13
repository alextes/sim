use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

use rand::seq::IteratorRandom;

use crate::location::{LocationSystem, OrbitalInfo, Point};

use crate::buildings::{EntityBuildings, MOON_SLOTS, PLANET_SLOTS};

mod resources;

pub use resources::ResourceSystem;

// Entity identifiers for all game objects.
pub type EntityId = u32;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

const STAR_COLORS: [Color; 3] = [
    Color {
        r: 255,
        g: 255,
        b: 255,
    }, // white
    Color {
        r: 255,
        g: 255,
        b: 224,
    }, // yellow-white
    Color {
        r: 173,
        g: 216,
        b: 230,
    }, // pale blue
];

const PLANET_COLORS: [Color; 3] = [
    Color {
        r: 60,
        g: 179,
        b: 113,
    }, // blue-green
    Color {
        r: 183,
        g: 65,
        b: 14,
    }, // rusty red
    Color {
        r: 244,
        g: 164,
        b: 96,
    }, // sandy brown
];

const MOON_COLORS: [Color; 3] = [
    Color {
        r: 211,
        g: 211,
        b: 211,
    }, // light gray
    Color {
        r: 128,
        g: 128,
        b: 128,
    }, // gray
    Color {
        r: 169,
        g: 169,
        b: 169,
    }, // dark gray
];

// --- star lane intersection helpers ---

// helper to check if point q lies on segment pr
fn on_segment(p: Point, q: Point, r: Point) -> bool {
    q.x <= max(p.x, r.x) && q.x >= min(p.x, r.x) && q.y <= max(p.y, r.y) && q.y >= min(p.y, r.y)
}

// helper to find orientation of ordered triplet (p, q, r).
// 0 --> p, q and r are collinear
// 1 --> clockwise
// 2 --> counterclockwise
fn orientation(p: Point, q: Point, r: Point) -> i32 {
    let val = (q.y as i64 - p.y as i64) * (r.x as i64 - q.x as i64)
        - (q.x as i64 - p.x as i64) * (r.y as i64 - q.y as i64);

    if val == 0 {
        return 0; // collinear
    }
    if val > 0 {
        1 // clockwise
    } else {
        2 // counterclockwise
    }
}

/// checks if line segment 'p1q1' and 'p2q2' intersect.
/// important: this function will report an intersection if segments share an endpoint.
/// the calling logic must handle cases where lanes share a star.
fn segments_intersect(p1: Point, q1: Point, p2: Point, q2: Point) -> bool {
    // find the four orientations needed for general and special cases
    let o1 = orientation(p1, q1, p2);
    let o2 = orientation(p1, q1, q2);
    let o3 = orientation(p2, q2, p1);
    let o4 = orientation(p2, q2, q1);

    // general case: segments cross each other
    if o1 != o2 && o3 != o4 {
        return true;
    }

    // special cases for collinear points
    // p1, q1 and p2 are collinear and p2 lies on segment p1q1
    if o1 == 0 && on_segment(p1, p2, q1) {
        return true;
    }

    // p1, q1 and q2 are collinear and q2 lies on segment p1q1
    if o2 == 0 && on_segment(p1, q2, q1) {
        return true;
    }

    // p2, q2 and p1 are collinear and p1 lies on segment p2q2
    if o3 == 0 && on_segment(p2, p1, q2) {
        return true;
    }

    // p2, q2 and q1 are collinear and q1 lies on segment p2q2
    if o4 == 0 && on_segment(p2, q1, q2) {
        return true;
    }

    false // doesn't fall in any of the above cases
}

#[derive(Debug, Default)]
pub struct World {
    next_entity_id: EntityId,
    /// ordered list of all entity IDs
    pub entities: Vec<EntityId>,
    /// glyphs to use when rendering each entity
    render_glyphs: HashMap<EntityId, char>,
    /// colors to use when rendering each entity
    entity_colors: HashMap<EntityId, Color>,
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
    /// set of entities controlled by the player
    pub player_controlled: HashSet<EntityId>,
}

impl World {
    /// Create a static entity at a fixed point (e.g. a star).
    pub fn spawn_star(&mut self, name: String, position: Point) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(id);
        self.entity_names.insert(id, name);
        self.render_glyphs.insert(id, '*');
        let mut rng = rand::rng();
        let color = STAR_COLORS.iter().choose(&mut rng).unwrap();
        self.entity_colors.insert(id, *color);
        self.locations.add_static(id, position);
        self.buildings.insert(id, EntityBuildings::new(0));
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
        let mut rng = rand::rng();
        let color = PLANET_COLORS.iter().choose(&mut rng).unwrap();
        self.entity_colors.insert(id, *color);
        self.locations
            .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
        self.buildings
            .insert(id, EntityBuildings::new(PLANET_SLOTS));
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
        let mut rng = rand::rng();
        let color = MOON_COLORS.iter().choose(&mut rng).unwrap();
        self.entity_colors.insert(id, *color);
        self.locations
            .add_orbital(id, anchor, radius, initial_angle, angular_velocity);
        self.buildings.insert(id, EntityBuildings::new(MOON_SLOTS));
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

    /// generate visual star lanes connecting stars.
    /// attempts to connect each star to `target_connections_per_star` closest neighbors,
    /// while ensuring each star has at least `minimum_connections_per_star`.
    pub fn generate_star_lanes(&mut self) {
        use std::collections::{HashMap, HashSet};

        const TARGET_CONNECTIONS_PER_STAR: usize = 4;
        const MINIMUM_CONNECTIONS_PER_STAR: usize = 2;

        let mut lanes: Vec<(EntityId, EntityId)> = Vec::new();
        let mut lane_set: HashSet<(EntityId, EntityId)> = HashSet::new();

        // helper to get ordered pair to avoid duplicates
        let add_lane_fn =
            |a: EntityId,
             b: EntityId,
             current_set: &mut HashSet<(EntityId, EntityId)>,
             current_lanes: &mut Vec<(EntityId, EntityId)>| {
                if a == b {
                    return false;
                }

                // --- intersection check ---
                // unwrap is safe here because we only deal with star_ids that are in the world.
                let p_a = self.get_location(a).unwrap();
                let p_b = self.get_location(b).unwrap();
                for &(other_a, other_b) in current_lanes.iter() {
                    // if they share a star, they don't cross, they connect. skip check.
                    if a == other_a || a == other_b || b == other_a || b == other_b {
                        continue;
                    }
                    let p_other_a = self.get_location(other_a).unwrap();
                    let p_other_b = self.get_location(other_b).unwrap();
                    if segments_intersect(p_a, p_b, p_other_a, p_other_b) {
                        return false; // intersects, can't add this lane.
                    }
                }

                let key = if a < b { (a, b) } else { (b, a) };
                if current_set.insert(key) {
                    current_lanes.push(key);
                    return true;
                }
                false
            };

        let star_ids: Vec<EntityId> = self
            .render_glyphs
            .iter()
            .filter_map(|(&id, &glyph)| if glyph == '*' { Some(id) } else { None })
            .collect();

        if star_ids.len() < 2 {
            self.lanes = lanes;
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
                if add_lane_fn(current_star_id, neighbor_id, &mut lane_set, &mut lanes) {
                    connections_made_for_current_star += 1;
                }
            }
        }

        // second pass: ensure minimum_connections_per_star
        let mut star_connection_counts: HashMap<EntityId, usize> = HashMap::new();
        for &(a, b) in &lanes {
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
                    if add_lane_fn(current_star_id, neighbor_id, &mut lane_set, &mut lanes) {
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
            lanes.iter().cloned().collect();

        loop {
            let mut made_change_this_pass = false;

            // recalculate connection counts based on the current set of lanes for this pass
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
