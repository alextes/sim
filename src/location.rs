use std::collections::HashMap;

use crate::world::EntityId;
use tracing::error;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PointF64 {
    pub x: f64,
    pub y: f64,
}

/// Manages static and orbital positions for entities, with nested anchoring support.
#[derive(Debug, Default)]
pub struct LocationSystem {
    entries: HashMap<EntityId, LocatedEntity>,
}

#[derive(Debug)]
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
            if let Some(LocatedEntity::Orbital {
                ref mut angle,
                ref mut position,
                ..
            }) = self.entries.get_mut(&id)
            {
                *angle = new_angle;
                *position = new_pos;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_static() {
        let mut ls = LocationSystem::default();
        ls.add_static(1, Point { x: 5, y: -3 });
        let p = ls.get_location(1).unwrap();
        assert_eq!(p, Point { x: 5, y: -3 });
    }

    #[test]
    fn test_add_orbital_initial_position() {
        let mut ls = LocationSystem::default();
        // anchor at (10, 20)
        ls.add_static(0, Point { x: 10, y: 20 });
        // radius 5, angle 0 => x offset 5, y offset 0
        ls.add_orbital(1, 0, 5.0, 0.0, 1.0);
        let p = ls.get_location(1).unwrap();
        assert_eq!(p, Point { x: 15, y: 20 });
    }

    #[test]
    fn test_orbital_update() {
        let mut ls = LocationSystem::default();
        // anchor at origin
        ls.add_static(0, Point { x: 0, y: 0 });
        // one revolution per second
        let w = std::f64::consts::TAU;
        ls.add_orbital(1, 0, 10.0, 0.0, w);
        // advance by 0.25s => angle = π/2
        ls.update(0.25);
        let p = ls.get_location(1).unwrap();
        // cos(π/2)=0, sin(π/2)=1 => position should be (0, 10)
        assert_eq!(p, Point { x: 0, y: 10 });
    }
}
