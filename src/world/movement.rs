use crate::location::PointF64;
use crate::world::World;

impl World {
    pub(super) fn update_ship_movement(&mut self, dt: f64) {
        let mut updates = Vec::new();
        for (&ship_id, &destination) in &self.move_orders {
            if let Some(current_pos) = self.locations.get_location_f64(ship_id) {
                let ship_speed = if let Some(ship_info) = self.ships.get(&ship_id) {
                    ship_info.speed
                } else {
                    tracing::warn!(
                        "ship {} has move order but no shipinfo, using default speed 1.0",
                        ship_id
                    );
                    1.0
                };

                let dx = destination.x - current_pos.x;
                let dy = destination.y - current_pos.y;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < 0.1 {
                    updates.push((ship_id, None)); // remove order
                } else {
                    let travel_dist = ship_speed * dt;
                    let new_pos = if travel_dist >= distance {
                        destination
                    } else {
                        PointF64 {
                            x: current_pos.x + (dx / distance) * travel_dist,
                            y: current_pos.y + (dy / distance) * travel_dist,
                        }
                    };
                    updates.push((ship_id, Some(new_pos)));
                }
            }
        }

        for (ship_id, new_pos_opt) in updates {
            if let Some(new_pos) = new_pos_opt {
                self.locations.set_position_f64(ship_id, new_pos).ok();
            } else {
                self.move_orders.remove(&ship_id);
            }
        }
    }
}
