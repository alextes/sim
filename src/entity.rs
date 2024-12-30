use std::collections::HashMap;

use crate::location::Point;

pub type EntityId = u32;

pub enum EntityType {
    Moon,
    Planet,
    Space,
    Star,
}

pub type EntityTypeMap = HashMap<EntityId, EntityType>;

pub trait Orbital {
    fn update_position(&mut self, anchor_x: i32, anchor_y: i32, time_delta: f64);
}

pub struct OrbitalEntity {
    pub id: EntityId,
    pub anchor_id: EntityId,
    pub radius: f64,
    pub angle: f64,
    pub angular_velocity: f64, // radians per second
    pub position: Point,
}

impl Orbital for OrbitalEntity {
    fn update_position(&mut self, anchor_x: i32, anchor_y: i32, time_delta: f64) {
        self.angle += self.angular_velocity * time_delta;
        self.position.x = anchor_x + (self.radius * self.angle.cos()) as i32;
        self.position.y = anchor_y + (self.radius * self.angle.sin()) as i32;
    }
}
