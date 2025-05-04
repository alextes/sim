use std::collections::HashMap;

use crate::location::{LocationSystem, Point};

mod resources;

pub use resources::Resources;

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
    entity_names: HashMap<EntityId, &'static str>,
    /// location system managing static and orbital positions
    locations: LocationSystem,
    /// Global resource counters for the player
    pub resources: Resources,
}

impl World {
    /// Create a static entity at a fixed point (e.g. a star).
    pub fn spawn_star(&mut self, name: &'static str, position: Point) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(id);
        self.entity_names.insert(id, name);
        self.render_glyphs.insert(id, '*');
        self.locations.add_static(id, position);
        id
    }

    /// Create an orbiting entity (e.g. planet or moon) around an existing entity.
    pub fn spawn_planet(
        &mut self,
        name: &'static str,
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
        id
    }

    /// Create an orbiting moon, using the 'm' glyph.
    pub fn spawn_moon(
        &mut self,
        name: &'static str,
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
        id
    }

    /// Advance all orbiters by dt_seconds, updating their stored positions.
    pub fn update(&mut self, dt_seconds: f64) {
        self.locations.update(dt_seconds);
    }

    /// Return the current universal position of an entity.
    pub fn get_location(&self, entity: EntityId) -> Option<Point> {
        self.locations.get_location(entity)
    }

    /// Iterate over all entity IDs in creation order.
    pub fn iter_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().cloned()
    }

    /// Get the human-readable name of an entity, if any.
    pub fn get_entity_name(&self, entity: EntityId) -> Option<&'static str> {
        self.entity_names.get(&entity).copied()
    }

    /// Get the glyph used for rendering this entity.
    pub fn get_render_glyph(&self, entity: EntityId) -> Option<char> {
        self.render_glyphs.get(&entity).copied()
    }
}
