use std::collections::HashMap;

use crate::entity::EntityId;
use tracing::error;

#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Manages static and orbital positions for entities, with nested anchoring support.
pub struct LocationSystem {
    entries: HashMap<EntityId, LocatedEntity>,
}

enum LocatedEntity {
    Static(Point),
    Orbital {
        anchor: EntityId,
        radius: f64,
        angle: f64,
        angular_velocity: f64,
        position: Point,
    },
}

impl LocationSystem {
    /// Create an empty LocationSystem.
    pub fn new() -> Self {
        LocationSystem {
            entries: HashMap::new(),
        }
    }

    /// Add a static (fixed) position for an entity.
    pub fn add_static(&mut self, entity: EntityId, position: Point) {
        self.entries.insert(entity, LocatedEntity::Static(position));
    }

    /// Add an orbital entry; initial position is computed relative to the anchor's current location.
    pub fn add_orbital(
        &mut self,
        entity: EntityId,
        anchor: EntityId,
        radius: f64,
        initial_angle: f64,
        angular_velocity: f64,
    ) {
        let anchor_pos = self.get_location(anchor).unwrap_or(Point { x: 0, y: 0 });
        let position = Point {
            x: anchor_pos.x + (radius * initial_angle.cos()) as i32,
            y: anchor_pos.y + (radius * initial_angle.sin()) as i32,
        };
        self.entries.insert(
            entity,
            LocatedEntity::Orbital {
                anchor,
                radius,
                angle: initial_angle,
                angular_velocity,
                position,
            },
        );
    }

    /// Advance all orbitals by dt seconds, updating their positions in spawn order.
    pub fn update(&mut self, dt: f64) {
        // collect all orbitals
        let mut updates = Vec::new();
        for (&id, loc) in &self.entries {
            if let LocatedEntity::Orbital {
                anchor,
                radius,
                angle,
                angular_velocity,
                ..
            } = loc
            {
                updates.push((id, *anchor, *radius, *angle, *angular_velocity));
            }
        }
        // apply in order, so anchors update before children
        for (id, anchor, radius, old_angle, angular_velocity) in updates {
            let new_angle = old_angle + angular_velocity * dt;
            let anchor_pos = self.get_location(anchor).unwrap_or_else(|| {
                error!("update: anchor {} not found", anchor);
                Point { x: 0, y: 0 }
            });
            let new_pos = Point {
                x: anchor_pos.x + (radius * new_angle.cos()) as i32,
                y: anchor_pos.y + (radius * new_angle.sin()) as i32,
            };
            if let Some(loc_ent) = self.entries.get_mut(&id) {
                if let LocatedEntity::Orbital {
                    ref mut angle,
                    ref mut position,
                    ..
                } = loc_ent
                {
                    *angle = new_angle;
                    *position = new_pos;
                }
            }
        }
    }

    /// Get the current absolute position of an entity.
    pub fn get_location(&self, entity: EntityId) -> Option<Point> {
        self.entries.get(&entity).map(|loc| match loc {
            LocatedEntity::Static(p) => *p,
            LocatedEntity::Orbital { position, .. } => *position,
        })
    }
}
