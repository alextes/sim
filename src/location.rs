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
    /// orbital IDs ordered by dependency depth, then entity ID. maintaining
    /// this on mutation keeps simulation and rendering iteration deterministic
    /// without sorting on their hot paths.
    orbital_order: Vec<EntityId>,
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
        let previous = self.entries.insert(
            entity,
            LocatedEntity::Static(PointF64 {
                x: position.x as f64,
                y: position.y as f64,
            }),
        );
        self.remove_from_orbital_order_if_needed(entity, previous.as_ref());
    }

    /// Add a mobile entity.
    pub fn add_mobile(&mut self, entity: EntityId, position: PointF64) {
        let previous = self.entries.insert(entity, LocatedEntity::Mobile(position));
        self.remove_from_orbital_order_if_needed(entity, previous.as_ref());
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
        let previous = self.entries.insert(
            entity,
            LocatedEntity::Orbital {
                anchor,
                radius,
                angle: initial_angle,
                angular_velocity,
                position,
            },
        );
        if !matches!(previous, Some(LocatedEntity::Orbital { .. })) {
            self.orbital_order.push(entity);
        }
        self.sort_orbital_order();
    }

    /// number of orbital ancestors above `entity` (0 for a non-orbital or an
    /// orbital anchored to a static body). used to order the orbital update so
    /// parents advance before their children.
    fn orbital_depth(entries: &HashMap<EntityId, LocatedEntity>, entity: EntityId) -> u32 {
        let mut depth = 0;
        let mut current = entity;
        while let Some(LocatedEntity::Orbital { anchor, .. }) = entries.get(&current) {
            depth += 1;
            current = *anchor;
            // guard against malformed cycles in the anchor chain.
            if depth > 64 {
                break;
            }
        }
        depth
    }

    fn sort_orbital_order(&mut self) {
        let entries = &self.entries;
        self.orbital_order
            .sort_by_key(|&id| (Self::orbital_depth(entries, id), id));
    }

    fn remove_from_orbital_order_if_needed(
        &mut self,
        entity: EntityId,
        previous: Option<&LocatedEntity>,
    ) {
        if matches!(previous, Some(LocatedEntity::Orbital { .. })) {
            self.orbital_order.retain(|&id| id != entity);
            self.sort_orbital_order();
        }
    }

    /// advance all orbitals by dt seconds, updating their positions. orbitals
    /// are processed parent-before-child so a body reads its anchor's fresh
    /// position within the same tick.
    pub fn update(&mut self, dt: f64) {
        for index in 0..self.orbital_order.len() {
            let id = self.orbital_order[index];
            let Some(LocatedEntity::Orbital {
                anchor,
                radius,
                angle,
                angular_velocity,
                ..
            }) = self.entries.get(&id)
            else {
                continue;
            };
            let (anchor, radius, old_angle, angular_velocity) =
                (*anchor, *radius, *angle, *angular_velocity);
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
        self.orbital_order.iter().filter_map(|&id| {
            let loc = self.entries.get(&id)?;
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

    #[test]
    fn orbital_iteration_is_ordered_by_depth_then_id() {
        let mut ls = LocationSystem::default();
        ls.add_static(100, Point { x: 0, y: 0 });
        ls.add_orbital(30, 100, 10.0, 0.0, 1.0);
        ls.add_orbital(20, 100, 12.0, 0.0, 1.0);
        // the child has the lowest ID but must follow both direct children.
        ls.add_orbital(10, 20, 2.0, 0.0, 1.0);

        let ids: Vec<_> = ls.iter_orbitals().map(|(id, _)| id).collect();

        assert_eq!(ids, vec![20, 30, 10]);
    }

    #[test]
    fn nested_orbitals_update_parent_before_child() {
        let mut ls = LocationSystem::default();
        ls.add_static(100, Point { x: 0, y: 0 });
        ls.add_orbital(20, 100, 10.0, 0.0, std::f64::consts::FRAC_PI_2);
        ls.add_orbital(10, 20, 2.0, 0.0, 0.0);

        ls.update(1.0);

        let child = ls.get_location_f64(10).unwrap();
        assert!((child.x - 2.0).abs() < 1e-10);
        assert!((child.y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn replacing_locations_maintains_orbital_order() {
        let mut ls = LocationSystem::default();
        ls.add_static(100, Point { x: 0, y: 0 });
        ls.add_orbital(30, 100, 10.0, 0.0, 1.0);
        ls.add_orbital(10, 30, 2.0, 0.0, 1.0);

        // reparenting changes depth and therefore iteration order.
        ls.add_orbital(10, 100, 2.0, 0.0, 1.0);
        let ids: Vec<_> = ls.iter_orbitals().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![10, 30]);

        // changing an orbital into a mobile entity removes it from the index.
        ls.add_mobile(10, PointF64 { x: 1.0, y: 2.0 });
        let ids: Vec<_> = ls.iter_orbitals().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![30]);
    }
}
