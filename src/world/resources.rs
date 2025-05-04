#![allow(dead_code)] // TODO remove later

use crate::buildings::{BuildingType, EntityBuildings};
use crate::world::EntityId;
use crate::SIMULATION_DT;
use std::collections::HashMap; // Import the constant from main.rs

// Calculate simulation frequency based on imported DT
const SIMULATION_HZ: f64 = 1.0 / SIMULATION_DT.as_secs_f64();

// --- Resource Generation Config ---
pub const RESOURCE_TICK_INTERVAL: u64 = 100; // Generate resources every 100 simulation ticks
pub const ENERGY_PER_SOLAR_PANEL_PER_INTERVAL: f32 = 1.0; // Corresponds to 1.0/sec at 100Hz
pub const METAL_PER_MINE_PER_INTERVAL: f32 = 0.5; // Corresponds to 0.5/sec at 100Hz

#[derive(Debug, Default)]
pub struct ResourceSystem {
    pub energy: f32,
    pub metal: f32,
}

impl ResourceSystem {
    /// Updates resource counts based on buildings and the current simulation tick.
    pub fn update(
        &mut self,
        current_tick: u64,
        buildings_map: &HashMap<EntityId, EntityBuildings>,
    ) {
        if current_tick > 0 && current_tick % RESOURCE_TICK_INTERVAL == 0 {
            let mut total_solar_panels = 0;
            let mut total_mines = 0;

            for buildings in buildings_map.values() {
                total_solar_panels += buildings
                    .orbital
                    .iter()
                    .filter(|&&slot| slot == Some(BuildingType::SolarPanel))
                    .count();
                if buildings.has_ground_slots {
                    total_mines += buildings
                        .ground
                        .iter()
                        .filter(|&&slot| slot == Some(BuildingType::Mine))
                        .count();
                }
            }

            self.energy += total_solar_panels as f32 * ENERGY_PER_SOLAR_PANEL_PER_INTERVAL;
            self.metal += total_mines as f32 * METAL_PER_MINE_PER_INTERVAL;
        }
    }

    /// Calculates the current income rates per second based on existing buildings.
    /// Returns (energy_per_second, metal_per_second).
    pub fn calculate_rates(
        &self,
        buildings_map: &HashMap<EntityId, EntityBuildings>,
    ) -> (f32, f32) {
        let mut total_solar_panels = 0;
        let mut total_mines = 0;

        for buildings in buildings_map.values() {
            total_solar_panels += buildings
                .orbital
                .iter()
                .filter(|&&slot| slot == Some(BuildingType::SolarPanel))
                .count();
            if buildings.has_ground_slots {
                total_mines += buildings
                    .ground
                    .iter()
                    .filter(|&&slot| slot == Some(BuildingType::Mine))
                    .count();
            }
        }

        // Calculate rate per second
        let ticks_per_second = SIMULATION_HZ;
        let generation_intervals_per_second = ticks_per_second / RESOURCE_TICK_INTERVAL as f64;

        let energy_rate = total_solar_panels as f32
            * ENERGY_PER_SOLAR_PANEL_PER_INTERVAL
            * generation_intervals_per_second as f32;
        let metal_rate = total_mines as f32
            * METAL_PER_MINE_PER_INTERVAL
            * generation_intervals_per_second as f32;

        (energy_rate, metal_rate)
    }
}
