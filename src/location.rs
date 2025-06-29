use std::collections::HashMap;

use anyhow::{anyhow, Result};

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

impl From<(f64, f64)> for PointF64 {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrbitalInfo {
    pub anchor: EntityId,
    pub radius: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct OrbitalParameters {
    pub anchor: EntityId,
    pub radius: f64,
    pub angle: f64,
    pub angular_velocity: f64,
}

/// Manages static and orbital positions for entities, with nested anchoring support.
#[derive(Debug, Default)]
pub struct LocationSystem {
    entries: HashMap<EntityId, LocatedEntity>,
}

#[derive(Debug)]
enum LocatedEntity {
    Static(PointF64),
    Orbital {
        anchor: EntityId,
        radius: f64,
        angle: f64,
        angular_velocity: f64,
        position: PointF64,
    },
    Mobile(PointF64),
}

impl LocationSystem {
    /// Add a static (fixed) position for an entity.
    pub fn add_static(&mut self, entity: EntityId, position: Point) {
        self.entries.insert(
            entity,
            LocatedEntity::Static(PointF64 {
                x: position.x as f64,
                y: position.y as f64,
            }),
        );
    }

    /// Add a mobile entity.
    pub fn add_mobile(&mut self, entity: EntityId, position: PointF64) {
        self.entries.insert(entity, LocatedEntity::Mobile(position));
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
        let anchor_pos = self.get_location_f64(anchor).unwrap_or_default();
        let position = PointF64 {
            x: anchor_pos.x + radius * initial_angle.cos(),
            y: anchor_pos.y + radius * initial_angle.sin(),
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
            let anchor_pos = self.get_location_f64(anchor).unwrap_or_else(|| {
                error!("update: anchor {} not found", anchor);
                PointF64::default()
            });
            let new_pos = PointF64 {
                x: anchor_pos.x + (radius * new_angle.cos()),
                y: anchor_pos.y + (radius * new_angle.sin()),
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

    /// sets the position of a mobile entity.
    #[cfg(test)]
    pub fn set_position(&mut self, entity: EntityId, new_pos: Point) -> Result<()> {
        match self.entries.get_mut(&entity) {
            Some(LocatedEntity::Mobile(pos)) => {
                pos.x = new_pos.x as f64;
                pos.y = new_pos.y as f64;
                Ok(())
            }
            Some(LocatedEntity::Static(_)) => {
                Err(anyhow!("cannot set position on a static entity"))
            }
            Some(LocatedEntity::Orbital { .. }) => {
                Err(anyhow!("cannot set position on an orbital entity"))
            }
            None => Err(anyhow!("entity not found")),
        }
    }

    /// sets the precise f64 position of a mobile entity.
    pub fn set_position_f64(&mut self, entity: EntityId, new_pos: PointF64) -> Result<()> {
        match self.entries.get_mut(&entity) {
            Some(LocatedEntity::Mobile(pos)) => {
                *pos = new_pos;
                Ok(())
            }
            Some(LocatedEntity::Static(_)) => {
                Err(anyhow!("cannot set f64 position on a static entity"))
            }
            Some(LocatedEntity::Orbital { .. }) => {
                Err(anyhow!("cannot set f64 position on an orbital entity"))
            }
            None => Err(anyhow!("entity not found")),
        }
    }

    /// Get the current absolute position of an entity.
    pub fn get_location(&self, entity: EntityId) -> Option<Point> {
        self.get_location_f64(entity).map(|p| Point {
            x: p.x.round() as i32,
            y: p.y.round() as i32,
        })
    }

    /// Get the current precise absolute f64 position of an entity.
    pub fn get_location_f64(&self, entity: EntityId) -> Option<PointF64> {
        self.entries.get(&entity).map(|loc| match loc {
            LocatedEntity::Static(p) => *p,
            LocatedEntity::Orbital { position, .. } => *position,
            LocatedEntity::Mobile(p) => *p,
        })
    }

    pub fn get_orbital_parameters(&self, entity: EntityId) -> Option<OrbitalParameters> {
        if let Some(LocatedEntity::Orbital {
            anchor,
            radius,
            angle,
            angular_velocity,
            ..
        }) = self.entries.get(&entity)
        {
            Some(OrbitalParameters {
                anchor: *anchor,
                radius: *radius,
                angle: *angle,
                angular_velocity: *angular_velocity,
            })
        } else {
            None
        }
    }

    pub fn iter_orbitals(&self) -> impl Iterator<Item = (EntityId, OrbitalInfo)> + '_ {
        self.entries.iter().filter_map(|(&id, loc)| {
            if let LocatedEntity::Orbital { anchor, radius, .. } = loc {
                Some((
                    id,
                    OrbitalInfo {
                        anchor: *anchor,
                        radius: *radius,
                    },
                ))
            } else {
                None
            }
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
    fn test_set_position_mobile() {
        let mut ls = LocationSystem::default();
        ls.add_mobile(1, PointF64 { x: 0.0, y: 0.0 });
        assert_eq!(ls.get_location(1), Some(Point { x: 0, y: 0 }));
        ls.set_position(1, Point { x: 10, y: -10 }).unwrap();
        assert_eq!(ls.get_location(1), Some(Point { x: 10, y: -10 }));
    }

    #[test]
    fn test_set_position_static_fails() {
        let mut ls = LocationSystem::default();
        ls.add_static(1, Point { x: 0, y: 0 });
        assert!(ls.set_position(1, Point { x: 10, y: -10 }).is_err());
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
