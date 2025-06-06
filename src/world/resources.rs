#![allow(dead_code)] // TODO remove later

use crate::buildings::{BuildingType, EntityBuildings};
use crate::world::EntityId;
use crate::SIMULATION_DT;
use std::collections::HashMap;
use std::sync::LazyLock;

// calculate simulation frequency based on simulation DT
static SIMULATION_HZ: LazyLock<f64> = LazyLock::new(|| 1.0 / SIMULATION_DT.as_secs_f64());

// --- Resource Generation Config ---
// Generate resources every N seconds of simulated time.
// SIMULATION_DT is 10ms (0.01s), so 100 ticks = 1.0 second.
pub const RESOURCE_INTERVAL_SECONDS: f64 = 0.25; // update four times per second
pub const ENERGY_PER_SOLAR_PANEL_PER_INTERVAL: f32 = 1.0 * RESOURCE_INTERVAL_SECONDS as f32; // Energy per interval
pub const METAL_PER_MINE_PER_INTERVAL: f32 = 0.5 * RESOURCE_INTERVAL_SECONDS as f32; // Metal per interval

#[derive(Debug, Default)]
pub struct ResourceSystem {
    pub energy: f32,
    pub metal: f32,
    time_accumulator: f64, // Accumulates dt_seconds
}

impl ResourceSystem {
    /// Updates resource counts based on buildings and elapsed simulated time.
    pub fn update(
        &mut self,
        dt_seconds: f64, // Delta time for the current simulation step
        buildings_map: &HashMap<EntityId, EntityBuildings>,
    ) {
        self.time_accumulator += dt_seconds;

        // Process all full intervals that have passed
        while self.time_accumulator >= RESOURCE_INTERVAL_SECONDS {
            self.time_accumulator -= RESOURCE_INTERVAL_SECONDS;

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

        // The number of generation intervals per real second is 1.0 / RESOURCE_INTERVAL_SECONDS.
        // For example, if RESOURCE_INTERVAL_SECONDS is 0.5, then this is 2 intervals per second.
        let generation_intervals_per_second = 1.0 / RESOURCE_INTERVAL_SECONDS;

        let energy_rate = total_solar_panels as f32
            * ENERGY_PER_SOLAR_PANEL_PER_INTERVAL
            * generation_intervals_per_second as f32;
        let metal_rate = total_mines as f32
            * METAL_PER_MINE_PER_INTERVAL
            * generation_intervals_per_second as f32;

        (energy_rate, metal_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buildings::{BuildingType, SlotType, GROUND_SLOTS, ORBITAL_SLOTS};

    fn create_buildings_with_counts(
        solar_panels: usize,
        mines: usize,
    ) -> HashMap<EntityId, EntityBuildings> {
        let mut buildings_map = HashMap::new();
        let mut entity_id_counter = 0;

        let mut solar_panels_to_add = solar_panels;
        let mut mines_to_add = mines;

        while solar_panels_to_add > 0 || mines_to_add > 0 {
            let mut entity_buildings = EntityBuildings::new(true);
            let entity_id = entity_id_counter;
            entity_id_counter += 1;

            let solar_to_build = solar_panels_to_add.min(ORBITAL_SLOTS);
            for i in 0..solar_to_build {
                entity_buildings
                    .build(SlotType::Orbital, i, BuildingType::SolarPanel)
                    .unwrap();
            }
            solar_panels_to_add -= solar_to_build;

            let mines_to_build = mines_to_add.min(GROUND_SLOTS);
            for i in 0..mines_to_build {
                entity_buildings
                    .build(SlotType::Ground, i, BuildingType::Mine)
                    .unwrap();
            }
            mines_to_add -= mines_to_build;

            buildings_map.insert(entity_id, entity_buildings);
        }

        buildings_map
    }

    #[test]
    fn test_calculate_rates() {
        let rs = ResourceSystem::default();

        // no buildings
        let buildings_map_empty = HashMap::new();
        let (energy_rate, metal_rate) = rs.calculate_rates(&buildings_map_empty);
        assert_eq!(energy_rate, 0.0);
        assert_eq!(metal_rate, 0.0);

        // with buildings
        let buildings_map = create_buildings_with_counts(2, 3);
        let (energy_rate, metal_rate) = rs.calculate_rates(&buildings_map);

        let generation_intervals_per_second = 1.0 / RESOURCE_INTERVAL_SECONDS;
        let expected_energy_rate =
            2.0 * ENERGY_PER_SOLAR_PANEL_PER_INTERVAL * generation_intervals_per_second as f32;
        let expected_metal_rate =
            3.0 * METAL_PER_MINE_PER_INTERVAL * generation_intervals_per_second as f32;

        assert!((energy_rate - expected_energy_rate).abs() < f32::EPSILON);
        assert!((metal_rate - expected_metal_rate).abs() < f32::EPSILON);
    }

    #[test]
    fn test_resource_system_update() {
        let mut rs = ResourceSystem::default();
        let buildings_map = create_buildings_with_counts(1, 1);

        // accumulate time, but not enough for an interval
        rs.update(RESOURCE_INTERVAL_SECONDS / 2.0, &buildings_map);
        assert_eq!(rs.energy, 0.0);
        assert_eq!(rs.metal, 0.0);

        // accumulate enough for one interval
        rs.update(RESOURCE_INTERVAL_SECONDS / 2.0, &buildings_map);
        assert_eq!(rs.energy, ENERGY_PER_SOLAR_PANEL_PER_INTERVAL);
        assert_eq!(rs.metal, METAL_PER_MINE_PER_INTERVAL);

        // check that accumulator is near zero
        assert!(rs.time_accumulator < 1e-9);

        // accumulate enough for multiple intervals at once
        rs.energy = 0.0;
        rs.metal = 0.0;
        rs.time_accumulator = 0.0;
        rs.update(RESOURCE_INTERVAL_SECONDS * 2.5, &buildings_map);
        assert_eq!(rs.energy, ENERGY_PER_SOLAR_PANEL_PER_INTERVAL * 2.0);
        assert_eq!(rs.metal, METAL_PER_MINE_PER_INTERVAL * 2.0);
        assert!((rs.time_accumulator - RESOURCE_INTERVAL_SECONDS * 0.5).abs() < 1e-9);
    }
}
