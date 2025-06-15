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

/// commodities extracted from celestial bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Commodity {
    Ore,
    Gas,
    Crystal,
}

/// deposit information for a celestial body.
/// each body has three deposits (ore, gas, crystal) for now.
#[derive(Debug, Clone)]
pub struct BodyResourceData {
    pub ore_yield: f32,
    pub gas_yield: f32,
    pub crystal_yield: f32,
    /// current population on the body (kept simple for now, default 1.0)
    pub population: f32,
}

impl Default for BodyResourceData {
    fn default() -> Self {
        Self {
            ore_yield: 1.0,
            gas_yield: 1.0,
            crystal_yield: 1.0,
            population: 1.0,
        }
    }
}

#[derive(Debug, Default)]
pub struct ResourceSystem {
    pub energy: f32,
    pub metal: f32,
    // new global resource counters extracted from celestial bodies
    pub ore: f32,
    pub gas: f32,
    pub crystal: f32,
    time_accumulator: f64, // Accumulates dt_seconds
}

impl ResourceSystem {
    /// Updates resource counts based on buildings and elapsed simulated time.
    /// additionally extracts raw commodities from celestial bodies following the
    /// formula: population x infrastructure x yield_grade.
    ///
    /// - population is stored in `BodyResourceData` (currently constant 1.0).
    /// - infrastructure is the number of `BuildingType::Mine` on the body.
    /// - yield_grade is the per-commodity yield stored on the body.
    pub fn update(
        &mut self,
        dt_seconds: f64, // Delta time for the current simulation step
        buildings_map: &HashMap<EntityId, EntityBuildings>,
        body_resources_map: &HashMap<EntityId, BodyResourceData>,
    ) {
        self.time_accumulator += dt_seconds;

        // Only update resources if enough time has passed
        if self.time_accumulator >= RESOURCE_INTERVAL_SECONDS {
            self.time_accumulator -= RESOURCE_INTERVAL_SECONDS;

            // ---------------------- building-based resources ----------------------
            // existing energy & metal generation from buildings
            for buildings in buildings_map.values() {
                for building in buildings.slots.iter().flatten() {
                    match building {
                        BuildingType::SolarPanel => {
                            self.energy += ENERGY_PER_SOLAR_PANEL_PER_INTERVAL;
                        }
                        BuildingType::Mine => {
                            self.metal += METAL_PER_MINE_PER_INTERVAL;
                        }
                        BuildingType::Shipyard => {}
                    }
                }
            }

            // ---------------------- commodity extraction ----------------------
            for (entity_id, body_data) in body_resources_map {
                // infrastructure: number of mines on this body
                let infra = buildings_map
                    .get(entity_id)
                    .map(|b| {
                        b.slots
                            .iter()
                            .filter(|slot| matches!(slot, Some(BuildingType::Mine)))
                            .count() as f32
                    })
                    .unwrap_or(0.0);

                if infra <= 0.0 {
                    continue; // nothing to extract here
                }

                let pop = body_data.population.max(0.0);

                // extraction amount for each commodity in this interval
                let ore_amount = pop * infra * body_data.ore_yield;
                let gas_amount = pop * infra * body_data.gas_yield;
                let crystal_amount = pop * infra * body_data.crystal_yield;

                self.ore += ore_amount;
                self.gas += gas_amount;
                self.crystal += crystal_amount;
            }
        }
    }

    /// Calculate the current production rates for energy and metal.
    pub fn calculate_rates(
        &self,
        buildings_map: &HashMap<EntityId, EntityBuildings>,
    ) -> (f32, f32) {
        let mut energy_rate = 0.0;
        let mut metal_rate = 0.0;

        // Calculate rates based on buildings
        for buildings in buildings_map.values() {
            // Count solar panels and mines in all slots
            for building in buildings.slots.iter().flatten() {
                match building {
                    BuildingType::SolarPanel => {
                        energy_rate += ENERGY_PER_SOLAR_PANEL_PER_INTERVAL;
                    }
                    BuildingType::Mine => {
                        metal_rate += METAL_PER_MINE_PER_INTERVAL;
                    }
                    BuildingType::Shipyard => {}
                }
            }
        }

        (energy_rate, metal_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buildings::{BuildingType, EntityBuildings, PLANET_SLOTS};
    use std::collections::HashMap;

    fn create_buildings_with_counts(
        solar_panels: usize,
        mines: usize,
    ) -> HashMap<EntityId, EntityBuildings> {
        let mut buildings = HashMap::<EntityId, EntityBuildings>::new();
        let mut buildings_data = EntityBuildings::new(PLANET_SLOTS);

        // Add solar panels
        for i in 0..solar_panels {
            if i < PLANET_SLOTS {
                buildings_data.build(i, BuildingType::SolarPanel).unwrap();
            }
        }

        // Add mines
        for i in 0..mines {
            if solar_panels + i < PLANET_SLOTS {
                buildings_data
                    .build(solar_panels + i, BuildingType::Mine)
                    .unwrap();
            }
        }

        buildings.insert(1, buildings_data);
        buildings
    }

    #[test]
    fn test_calculate_rates() {
        let buildings = create_buildings_with_counts(2, 1);
        let resource_system = ResourceSystem::default();
        let (energy_rate, metal_rate) = resource_system.calculate_rates(&buildings);

        let expected_energy = 2.0 * ENERGY_PER_SOLAR_PANEL_PER_INTERVAL;
        let expected_metal = 1.0 * METAL_PER_MINE_PER_INTERVAL;

        assert_eq!(energy_rate, expected_energy);
        assert_eq!(metal_rate, expected_metal);
    }

    #[test]
    fn test_resource_system_update_with_body_resources() {
        let buildings = create_buildings_with_counts(0, 2);
        let mut resource_system = ResourceSystem::default();

        // prepare body resources map with single body id 1
        let mut body_resources_map = HashMap::new();
        body_resources_map.insert(
            1,
            BodyResourceData {
                ore_yield: 1.0,
                gas_yield: 2.0,
                crystal_yield: 0.5,
                population: 1.0,
            },
        );

        // Update for one full interval
        resource_system.update(
            RESOURCE_INTERVAL_SECONDS,
            &buildings,
            &body_resources_map,
        );

        // two mines * population 1 * yield
        assert_eq!(resource_system.ore, 2.0);
        assert_eq!(resource_system.gas, 4.0);
        assert_eq!(resource_system.crystal, 1.0);
    }
}
