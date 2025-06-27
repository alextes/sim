#![allow(dead_code)] // TODO remove later

use crate::buildings::{BuildingType, EntityBuildings};
use crate::world::types::{CelestialBodyData, ResourceType};
use crate::world::EntityId;
use crate::SIMULATION_DT;
use std::collections::HashMap;
use std::sync::LazyLock;

// calculate simulation frequency based on simulation DT
static SIMULATION_HZ: LazyLock<f64> = LazyLock::new(|| 1.0 / SIMULATION_DT.as_secs_f64());

// --- Resource Generation Config ---
// Generate resources every N seconds of simulated time.
// SIMULATION_DT is 10ms (0.01s), so 100 ticks = 1.0 second.
pub const RESOURCE_INTERVAL_SECONDS: f64 = 1.0; // update once per second
pub const ENERGY_PER_SOLAR_PANEL_PER_INTERVAL: f32 = 1.0 * RESOURCE_INTERVAL_SECONDS as f32; // Energy per interval

#[derive(Debug, Default)]
pub struct ResourceSystem {
    // This system is now only responsible for ticking time forward for production.
    // All resource stockpiles are held in CelestialBodyData.
    time_accumulator: f64, // Accumulates dt_seconds
}

impl ResourceSystem {
    /// Updates resource counts based on buildings and elapsed simulated time.
    pub fn update(
        &mut self,
        dt_seconds: f64, // Delta time for the current simulation step
        render_glyphs: &HashMap<EntityId, char>,
        buildings_map: &HashMap<EntityId, EntityBuildings>,
        celestial_data_map: &mut HashMap<EntityId, CelestialBodyData>,
    ) {
        self.time_accumulator += dt_seconds;

        let num_intervals = (self.time_accumulator / RESOURCE_INTERVAL_SECONDS).floor() as u32;

        if num_intervals == 0 {
            return;
        }

        self.time_accumulator -= num_intervals as f64 * RESOURCE_INTERVAL_SECONDS;

        let production_multiplier = num_intervals as f32 * RESOURCE_INTERVAL_SECONDS as f32;

        for (entity_id, celestial_data) in celestial_data_map.iter_mut() {
            // As per user request, only for planets for now
            let glyph = render_glyphs.get(entity_id).copied().unwrap_or('?');
            if glyph != 'p' && glyph != 'm' && glyph != 'g' {
                continue;
            }

            let buildings = match buildings_map.get(entity_id) {
                Some(b) => b,
                None => continue,
            };

            let infra = buildings
                .slots
                .iter()
                .filter(|s| s.is_some() && s.unwrap() == BuildingType::Mine)
                .count() as f32;

            if infra == 0.0 {
                continue;
            }

            for (resource_type, yield_grade) in &celestial_data.yields {
                let production =
                    celestial_data.population * infra * *yield_grade * production_multiplier;
                let stock = celestial_data.stocks.entry(*resource_type).or_insert(0.0);
                *stock += production;
            }
        }
    }

    /// Calculate the current production rates for all resources across all celestial bodies.
    /// this is an aggregate view for the UI, not a global stockpile.
    pub fn calculate_rates(
        &self,
        buildings_map: &HashMap<EntityId, EntityBuildings>,
        celestial_data_map: &HashMap<EntityId, CelestialBodyData>,
    ) -> HashMap<ResourceType, f32> {
        let mut rates = HashMap::new();

        // Calculate rates based on buildings
        for (entity_id, buildings) in buildings_map.iter() {
            let celestial_data = match celestial_data_map.get(entity_id) {
                Some(data) => data,
                None => continue,
            };

            let mut infra = 0.0;
            let mut energy_rate = 0.0; // Track energy separately for now

            for building in buildings.slots.iter().flatten() {
                match building {
                    BuildingType::Mine => infra += 1.0,
                    BuildingType::SolarPanel => {
                        // note: this is a simple placeholder for energy.
                        // a proper implementation would have energy as a resource with local storage.
                        energy_rate += ENERGY_PER_SOLAR_PANEL_PER_INTERVAL;
                    }
                    BuildingType::Shipyard => {}
                }
            }

            if infra > 0.0 {
                for (resource_type, yield_grade) in &celestial_data.yields {
                    let production_rate = celestial_data.population * infra * yield_grade;
                    *rates.entry(*resource_type).or_insert(0.0) += production_rate;
                }
            }
        }
        rates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buildings::{BuildingType, EntityBuildings, PLANET_SLOTS};
    use crate::world::types::{CelestialBodyData, ResourceType};
    use std::collections::HashMap;

    fn create_test_data(
        mines: usize,
    ) -> (
        HashMap<EntityId, char>,
        HashMap<EntityId, EntityBuildings>,
        HashMap<EntityId, CelestialBodyData>,
    ) {
        let mut buildings_map = HashMap::new();
        let mut buildings_data = EntityBuildings::new(PLANET_SLOTS);
        let entity_id = 1;

        for i in 0..mines {
            if i < PLANET_SLOTS {
                buildings_data.build(i, BuildingType::Mine).unwrap();
            }
        }
        buildings_map.insert(entity_id, buildings_data);

        let mut celestial_data_map = HashMap::new();
        let mut yields = HashMap::new();
        yields.insert(ResourceType::Metals, 1.2);
        yields.insert(ResourceType::Crystals, 0.4);
        yields.insert(ResourceType::Organics, 0.8);

        celestial_data_map.insert(
            entity_id,
            CelestialBodyData {
                population: 1.0,
                yields,
                stocks: HashMap::new(),
            },
        );

        let mut render_glyphs = HashMap::new();
        render_glyphs.insert(entity_id, 'p');

        (render_glyphs, buildings_map, celestial_data_map)
    }

    #[test]
    fn test_resource_system_update() {
        let (glyphs, buildings, mut celestial_data) = create_test_data(1);
        let mut resource_system = ResourceSystem::default();

        resource_system.update(
            RESOURCE_INTERVAL_SECONDS,
            &glyphs,
            &buildings,
            &mut celestial_data,
        );

        let interval_f32 = RESOURCE_INTERVAL_SECONDS as f32;
        let stocks = &celestial_data.get(&1).unwrap().stocks;
        assert_eq!(
            *stocks.get(&ResourceType::Metals).unwrap(),
            1.0 * 1.0 * 1.2 * interval_f32
        );
        assert_eq!(
            *stocks.get(&ResourceType::Crystals).unwrap(),
            1.0 * 1.0 * 0.4 * interval_f32
        );
        assert_eq!(
            *stocks.get(&ResourceType::Organics).unwrap(),
            1.0 * 1.0 * 0.8 * interval_f32
        );
    }
}
