use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

use crate::location::{LocationSystem, OrbitalInfo, Point};

use crate::buildings::EntityBuildings;
use crate::command::Command;
use crate::location::PointF64;
use std::collections::VecDeque;

mod resources;
pub mod spawning;
pub mod types;

pub use resources::ResourceSystem;
use types::CelestialBodyData;

// Entity identifiers for all game objects.
pub type EntityId = u32;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ShipInfo {
    pub speed: f64,
}

pub const STAR_COLORS: [Color; 3] = [
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

pub const PLANET_COLORS: [Color; 3] = [
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

pub const MOON_COLORS: [Color; 3] = [
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
    pub(crate) next_entity_id: EntityId,
    /// ordered list of all entity IDs
    pub entities: Vec<EntityId>,
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
    pub fn update(&mut self, dt_seconds: f64, _current_tick: u64) {
        let commands: Vec<Command> = self.command_queue.drain(..).collect();
        for command in commands {
            self.process_command(command);
        }

        self.locations.update(dt_seconds);
        self.resources
            .update(dt_seconds, &self.buildings, &self.celestial_data);

        // ship movement
        let mut completed_moves = Vec::new();
        for (&ship_id, destination) in &self.move_orders {
            if let Some(current_pos) = self.locations.get_location_f64(ship_id) {
                let ship_info = self.ships.get(&ship_id).unwrap(); // should exist if in move_orders

                let vector_to_dest = PointF64 {
                    x: destination.x - current_pos.x,
                    y: destination.y - current_pos.y,
                };
                let distance = (vector_to_dest.x.powi(2) + vector_to_dest.y.powi(2)).sqrt();
                let move_dist = ship_info.speed * dt_seconds;

                if distance < move_dist {
                    // arrived
                    self.locations
                        .set_position_f64(ship_id, *destination)
                        .unwrap();
                    completed_moves.push(ship_id);
                } else {
                    // move towards destination
                    let direction = PointF64 {
                        x: vector_to_dest.x / distance,
                        y: vector_to_dest.y / distance,
                    };
                    let new_pos = PointF64 {
                        x: current_pos.x + direction.x * move_dist,
                        y: current_pos.y + direction.y * move_dist,
                    };
                    self.locations.set_position_f64(ship_id, new_pos).unwrap();
                }
            }
        }

        for ship_id in completed_moves {
            self.move_orders.remove(&ship_id);
        }
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::MoveShip {
                ship_id,
                destination,
            } => {
                self.set_move_order(ship_id, destination);
            }
            Command::BuildShip {
                shipyard_entity_id,
                ship_type,
            } => {
                // for now, we only have one ship type
                let _ = ship_type;
                if let Some(location) = self.locations.get_location(shipyard_entity_id) {
                    let ship_name = format!("frigate-{}", self.ships.len());
                    self.spawn_frigate(ship_name, location);
                }
            }
            Command::BuildBuilding {
                entity_id,
                building_type,
            } => {
                if let Some(buildings) = self.buildings.get_mut(&entity_id) {
                    if let Some(slot) = buildings.find_first_empty_slot() {
                        buildings.build(slot, building_type).ok(); // ok to fail silently
                    }
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;

    #[test]
    fn test_segments_intersect() {
        // simple intersection
        let p1 = Point { x: 0, y: 0 };
        let q1 = Point { x: 10, y: 10 };
        let p2 = Point { x: 0, y: 10 };
        let q2 = Point { x: 10, y: 0 };
        assert!(segments_intersect(p1, q1, p2, q2));

        // no intersection
        let p3 = Point { x: 0, y: 0 };
        let q3 = Point { x: 1, y: 1 };
        let p4 = Point { x: 2, y: 2 };
        let q4 = Point { x: 3, y: 3 };
        assert!(!segments_intersect(p3, q3, p4, q4));

        // collinear and overlapping
        let p5 = Point { x: 0, y: 0 };
        let q5 = Point { x: 10, y: 0 };
        let p6 = Point { x: 5, y: 0 };
        let q6 = Point { x: 15, y: 0 };
        assert!(segments_intersect(p5, q5, p6, q6));

        // collinear but not overlapping
        let p7 = Point { x: 0, y: 0 };
        let q7 = Point { x: 1, y: 0 };
        let p8 = Point { x: 2, y: 0 };
        let q8 = Point { x: 3, y: 0 };
        assert!(!segments_intersect(p7, q7, p8, q8));

        // sharing an endpoint
        let p9 = Point { x: 0, y: 0 };
        let q9 = Point { x: 1, y: 1 };
        let p10 = Point { x: 1, y: 1 };
        let q10 = Point { x: 2, y: 0 };
        assert!(segments_intersect(p9, q9, p10, q10));

        // T-junction
        let p11 = Point { x: 5, y: 0 };
        let q11 = Point { x: 5, y: 10 };
        let p12 = Point { x: 0, y: 5 };
        let q12 = Point { x: 10, y: 5 };
        assert!(segments_intersect(p11, q11, p12, q12));
    }

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
