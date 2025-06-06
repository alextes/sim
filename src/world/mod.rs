use std::collections::HashMap;

use crate::location::{LocationSystem, Point};

use crate::buildings::EntityBuildings;

mod resources;

pub use resources::ResourceSystem;

// Entity identifiers for all game objects.
pub type EntityId = u32;

// maximum number of star lanes per star
const MAX_LANES_PER_STAR: usize = 5;

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
        use std::collections::{HashMap, HashSet};

        // helper to get ordered pair to avoid duplicates
        let add_lane = |a: EntityId,
                        b: EntityId,
                        set: &mut HashSet<(EntityId, EntityId)>,
                        lanes: &mut Vec<(EntityId, EntityId)>,
                        lane_count: &mut HashMap<EntityId, usize>| -> bool {
            if a == b {
                return false;
            }
            // check if either star already has max lanes
            let a_count = lane_count.get(&a).copied().unwrap_or(0);
            let b_count = lane_count.get(&b).copied().unwrap_or(0);
            if a_count >= MAX_LANES_PER_STAR || b_count >= MAX_LANES_PER_STAR {
                return false;
            }
            
            let key = if a < b { (a, b) } else { (b, a) };
            if set.insert(key) {
                lanes.push(key);
                *lane_count.entry(a).or_insert(0) += 1;
                *lane_count.entry(b).or_insert(0) += 1;
                return true;
            }
            false
        };

        // helper to check if two lanes are nearly parallel
        let are_lanes_nearly_parallel = |p1: Point, p2: Point, p3: Point, p4: Point| -> bool {
            // vector from p1 to p2
            let v1 = (p2.x - p1.x, p2.y - p1.y);
            // vector from p3 to p4
            let v2 = (p4.x - p3.x, p4.y - p3.y);
            
            // calculate dot product and magnitudes
            let dot = (v1.0 * v2.0 + v1.1 * v2.1) as f64;
            let mag1 = ((v1.0 * v1.0 + v1.1 * v1.1) as f64).sqrt();
            let mag2 = ((v2.0 * v2.0 + v2.1 * v2.1) as f64).sqrt();
            
            if mag1 == 0.0 || mag2 == 0.0 {
                return false;
            }
            
            // cosine of angle between vectors
            let cos_angle = dot / (mag1 * mag2);
            // consider nearly parallel if angle < 15 degrees (cos > 0.966)
            cos_angle.abs() > 0.966
        };

        let mut lane_set: HashSet<(EntityId, EntityId)> = HashSet::new();
        let mut lane_count: HashMap<EntityId, usize> = HashMap::new();
        self.lanes.clear();

        // collect star ids (we assume every spawn_star entity is a star; others planets/moons etc.)
        // heuristic: glyph '*' for star used in spawn_star
        let star_ids: Vec<EntityId> = self
            .render_glyphs
            .iter()
            .filter_map(|(&id, &glyph)| if glyph == '*' { Some(id) } else { None })
            .collect();

        // collect star positions for efficiency
        let mut star_positions: HashMap<EntityId, Point> = HashMap::new();
        for &star in &star_ids {
            if let Some(pos) = self.get_location(star) {
                star_positions.insert(star, pos);
            }
        }

        // generate lanes within threshold
        let mut potential_lanes: Vec<(EntityId, EntityId, i32)> = Vec::new();
        for i in 0..star_ids.len() {
            for j in i + 1..star_ids.len() {
                let a = star_ids[i];
                let b = star_ids[j];
                if let (Some(&pa), Some(&pb)) = (star_positions.get(&a), star_positions.get(&b)) {
                    let dx = pa.x - pb.x;
                    let dy = pa.y - pb.y;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= max_distance * max_distance {
                        potential_lanes.push((a, b, dist_sq));
                    }
                }
            }
        }
        
        // sort by distance to prefer shorter lanes
        potential_lanes.sort_by_key(|&(_, _, dist)| dist);
        
        // add lanes, checking for near-parallel conflicts
        for (a, b, _) in potential_lanes {
            if let (Some(&pa), Some(&pb)) = (star_positions.get(&a), star_positions.get(&b)) {
                // check if this lane would be nearly parallel to existing lanes
                let mut is_valid = true;
                for &(existing_a, existing_b) in &self.lanes {
                    if let (Some(&epa), Some(&epb)) = (star_positions.get(&existing_a), star_positions.get(&existing_b)) {
                        // check if new lane (a,b) is nearly parallel to existing lane
                        if are_lanes_nearly_parallel(pa, pb, epa, epb) {
                            // also check if they share a common point (avoid A-B and B-C being parallel)
                            if a == existing_a || a == existing_b || b == existing_a || b == existing_b {
                                is_valid = false;
                                break;
                            }
                        }
                    }
                }
                
                if is_valid {
                    add_lane(a, b, &mut lane_set, &mut self.lanes, &mut lane_count);
                }
            }
        }

        // ensure each star has at least one lane
        for &star in &star_ids {
            let current_count = lane_count.get(&star).copied().unwrap_or(0);
            if current_count == 0 {
                // find nearest other star
                let mut nearest: Option<(EntityId, i32)> = None; // (id, dist_sq)
                for &other in &star_ids {
                    if other == star {
                        continue;
                    }
                    if let (Some(&pa), Some(&pb)) =
                        (star_positions.get(&star), star_positions.get(&other))
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
                    add_lane(star, other, &mut lane_set, &mut self.lanes, &mut lane_count);
                }
            }
        }
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
    fn test_generate_lanes_for_close_stars() {
        let mut world = World::default();
        
        // create three stars close together
        let star1 = world.spawn_star("star1".to_string(), Point { x: 0, y: 0 });
        let star2 = world.spawn_star("star2".to_string(), Point { x: 30, y: 0 });
        let star3 = world.spawn_star("star3".to_string(), Point { x: 0, y: 30 });
        
        // generate lanes with max distance 50
        world.generate_star_lanes(50);
        
        // should have at least 2 lanes connecting these stars
        assert!(world.lanes.len() >= 2);
        
        // verify the lanes connect our stars
        let lane_pairs: Vec<(EntityId, EntityId)> = world.lanes.clone();
        let mut connections = std::collections::HashSet::new();
        for (a, b) in lane_pairs {
            connections.insert(a);
            connections.insert(b);
        }
        
        assert!(connections.contains(&star1));
        assert!(connections.contains(&star2));
        assert!(connections.contains(&star3));
    }

    #[test]
    fn test_no_lanes_for_far_stars() {
        let mut world = World::default();
        
        // create two stars very far apart
        let star1 = world.spawn_star("star1".to_string(), Point { x: 0, y: 0 });
        let star2 = world.spawn_star("star2".to_string(), Point { x: 200, y: 200 });
        
        // generate lanes with max distance 50
        world.generate_star_lanes(50);
        
        // should have exactly 1 lane (minimum connectivity rule)
        assert_eq!(world.lanes.len(), 1);
        assert_eq!(world.lanes[0], (star1, star2));
    }

    #[test]
    fn test_minimum_one_lane_per_star() {
        let mut world = World::default();
        
        // create isolated stars
        let stars: Vec<EntityId> = (0..5).map(|i| {
            world.spawn_star(
                format!("star{}", i),
                Point { x: i * 1000, y: 0 }
            )
        }).collect();
        
        // generate lanes with small max distance
        world.generate_star_lanes(50);
        
        // verify each star has at least one connection
        for &star in &stars {
            let has_connection = world.lanes.iter()
                .any(|&(a, b)| a == star || b == star);
            assert!(has_connection, "star {} has no connections", star);
        }
    }

    #[test]
    fn test_max_lanes_per_star() {
        let mut world = World::default();
        
        // create a central star with many neighbors
        let center = world.spawn_star("center".to_string(), Point { x: 0, y: 0 });
        
        // create 8 surrounding stars
        let mut surrounding = Vec::new();
        for i in 0..8 {
            let angle = (i as f64) * std::f64::consts::TAU / 8.0;
            let x = (30.0 * angle.cos()) as i32;
            let y = (30.0 * angle.sin()) as i32;
            surrounding.push(world.spawn_star(
                format!("star{}", i),
                Point { x, y }
            ));
        }
        
        // generate lanes
        world.generate_star_lanes(50);
        
        // count lanes for center star
        let center_lanes = world.lanes.iter()
            .filter(|&(a, b)| *a == center || *b == center)
            .count();
        
        // should not exceed MAX_LANES_PER_STAR
        assert!(center_lanes <= MAX_LANES_PER_STAR, 
                "center star has {} lanes, exceeds max of {}", 
                center_lanes, MAX_LANES_PER_STAR);
    }

    #[test]
    fn test_no_duplicate_lanes() {
        let mut world = World::default();
        
        // create a small cluster
        let _stars: Vec<EntityId> = vec![
            world.spawn_star("star1".to_string(), Point { x: 0, y: 0 }),
            world.spawn_star("star2".to_string(), Point { x: 20, y: 0 }),
            world.spawn_star("star3".to_string(), Point { x: 10, y: 20 }),
        ];
        
        // generate lanes
        world.generate_star_lanes(50);
        
        // check for duplicates
        let mut seen = std::collections::HashSet::new();
        for &(a, b) in &world.lanes {
            let key = if a < b { (a, b) } else { (b, a) };
            assert!(seen.insert(key), "duplicate lane found: {:?}", key);
        }
    }

    #[test]
    fn test_avoid_nearly_parallel_lanes() {
        let mut world = World::default();
        
        // create three stars in a line
        let star_a = world.spawn_star("a".to_string(), Point { x: 0, y: 0 });
        let star_b = world.spawn_star("b".to_string(), Point { x: 50, y: 0 });
        let star_c = world.spawn_star("c".to_string(), Point { x: 100, y: 0 });
        
        // add a fourth star to ensure minimum connectivity
        let _star_d = world.spawn_star("d".to_string(), Point { x: 50, y: 50 });
        
        // generate lanes
        world.generate_star_lanes(60);
        
        // should not have both A-B and B-C lanes as they would be parallel
        let has_ab = world.lanes.iter().any(|&(a, b)| 
            (a == star_a && b == star_b) || (a == star_b && b == star_a));
        let has_bc = world.lanes.iter().any(|&(a, b)| 
            (a == star_b && b == star_c) || (a == star_c && b == star_b));
        let has_ac = world.lanes.iter().any(|&(a, b)| 
            (a == star_a && b == star_c) || (a == star_c && b == star_a));
        
        // should not have all three parallel lanes
        let parallel_count = [has_ab, has_bc, has_ac].iter().filter(|&&x| x).count();
        assert!(parallel_count < 3, 
                "too many parallel lanes detected: AB={}, BC={}, AC={}", 
                has_ab, has_bc, has_ac);
    }

    #[test]
    fn test_prefer_shorter_lanes() {
        let mut world = World::default();
        
        // create a triangle of stars
        let star1 = world.spawn_star("star1".to_string(), Point { x: 0, y: 0 });
        let star2 = world.spawn_star("star2".to_string(), Point { x: 30, y: 0 });
        let star3 = world.spawn_star("star3".to_string(), Point { x: 15, y: 25 });
        
        // generate lanes
        world.generate_star_lanes(50);
        
        // all three stars should be connected since they're all close
        let connections: std::collections::HashSet<EntityId> = world.lanes.iter()
            .flat_map(|&(a, b)| vec![a, b])
            .collect();
        
        assert!(connections.contains(&star1));
        assert!(connections.contains(&star2));
        assert!(connections.contains(&star3));
        
        // should have exactly 3 lanes for a triangle
        assert_eq!(world.lanes.len(), 3);
    }
}
