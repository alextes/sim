#![allow(dead_code)] // TODO remove later

use crate::buildings::{BuildingType, EntityBuildings};
use crate::world::types::EntityType;
use crate::world::types::{CelestialBodyData, Good, RawResource, Storable};
use crate::world::EntityId;
use crate::world::World;
use crate::SIMULATION_DT;
use std::collections::HashMap;
use std::sync::LazyLock;

// calculate simulation frequency based on simulation DT
static SIMULATION_HZ: LazyLock<f64> = LazyLock::new(|| 1.0 / SIMULATION_DT.as_secs_f64());

// --- Resource Generation Config ---
// Generate resources every N seconds of simulated time.
// SIMULATION_DT is 10ms (0.01s), so 100 ticks = 1.0 second.
pub const RESOURCE_INTERVAL_SECONDS: f64 = 1.0; // update once per second

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
        entity_types: &HashMap<EntityId, EntityType>,
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
            let entity_type = match entity_types.get(entity_id) {
                Some(t) => t,
                None => continue,
            };

            match entity_type {
                EntityType::Planet | EntityType::Moon | EntityType::GasGiant => {
                    // This entity type produces resources.
                }
                _ => continue, // Other types do not produce resources.
            }

            let buildings = match buildings_map.get(entity_id) {
                Some(b) => b,
                None => continue,
            };

            // handle raw resource extraction from mines
            let mine_infra = buildings
                .slots
                .iter()
                .filter(|s| s.is_some() && s.unwrap() == BuildingType::Mine)
                .count() as f32;

            if mine_infra > 0.0 {
                for (resource_type, yield_grade) in &celestial_data.yields {
                    let production = (celestial_data.population / 1_000_000.0)
                        * mine_infra
                        * *yield_grade
                        * production_multiplier;
                    let stock = celestial_data
                        .stocks
                        .entry(Storable::Raw(*resource_type))
                        .or_insert(0.0);
                    *stock += production;
                }
            }

            // handle manufactured goods production
            let cracker_infra = buildings
                .slots
                .iter()
                .filter(|s| s.is_some() && s.unwrap() == BuildingType::FuelCellCracker)
                .count() as f32;

            if cracker_infra > 0.0 {
                // recipe: 1 volatile + 0.1 metals -> 1 fuel cell
                let volatiles_needed = 1.0 * cracker_infra * production_multiplier;
                let metals_needed = 0.1 * cracker_infra * production_multiplier;

                let available_volatiles = celestial_data
                    .stocks
                    .get(&Storable::Raw(RawResource::Volatiles))
                    .copied()
                    .unwrap_or(0.0);
                let available_metals = celestial_data
                    .stocks
                    .get(&Storable::Raw(RawResource::Metals))
                    .copied()
                    .unwrap_or(0.0);

                let production_possible_by_volatiles = available_volatiles / 1.0;
                let production_possible_by_metals = available_metals / 0.1;

                let actual_production = volatiles_needed
                    .min(metals_needed)
                    .min(production_possible_by_volatiles)
                    .min(production_possible_by_metals);

                if actual_production > 0.0 {
                    // consume resources
                    *celestial_data
                        .stocks
                        .entry(Storable::Raw(RawResource::Volatiles))
                        .or_insert(0.0) -= actual_production * 1.0;
                    *celestial_data
                        .stocks
                        .entry(Storable::Raw(RawResource::Metals))
                        .or_insert(0.0) -= actual_production * 0.1;

                    // produce fuel cells
                    *celestial_data
                        .stocks
                        .entry(Storable::Good(Good::FuelCells))
                        .or_insert(0.0) += actual_production;
                }
            }

            // handle food production from farms
            let farm_infra = buildings
                .slots
                .iter()
                .filter(|s| s.is_some() && s.unwrap() == BuildingType::Farm)
                .count() as f32;

            if farm_infra > 0.0 {
                // recipe: 1 organics -> 1 food
                let organics_needed = 1.0 * farm_infra * production_multiplier;
                let available_organics = celestial_data
                    .stocks
                    .get(&Storable::Raw(RawResource::Organics))
                    .copied()
                    .unwrap_or(0.0);
                let actual_production = organics_needed.min(available_organics);

                if actual_production > 0.0 {
                    // consume organics
                    *celestial_data
                        .stocks
                        .entry(Storable::Raw(RawResource::Organics))
                        .or_insert(0.0) -= actual_production;
                    // produce food
                    *celestial_data
                        .stocks
                        .entry(Storable::Good(Good::Food))
                        .or_insert(0.0) += actual_production;
                }
            }
        }
    }

    /// Calculate the current production rates for all resources across all celestial bodies.
    /// this is an aggregate view for the UI, not a global stockpile.
    pub fn calculate_rates(
        &self,
        buildings_map: &HashMap<EntityId, EntityBuildings>,
        celestial_data_map: &HashMap<EntityId, CelestialBodyData>,
    ) -> HashMap<Storable, f32> {
        let mut rates = HashMap::new();

        // Calculate rates based on buildings
        for (entity_id, buildings) in buildings_map.iter() {
            let celestial_data = match celestial_data_map.get(entity_id) {
                Some(data) => data,
                None => continue,
            };

            let mine_infra = buildings
                .slots
                .iter()
                .filter(|s| matches!(s, Some(BuildingType::Mine)))
                .count() as f32;

            if mine_infra > 0.0 {
                for (resource_type, yield_grade) in &celestial_data.yields {
                    let production_rate =
                        (celestial_data.population / 1_000_000.0) * mine_infra * yield_grade;
                    *rates.entry(Storable::Raw(*resource_type)).or_insert(0.0) += production_rate;
                }
            }

            let cracker_infra = buildings
                .slots
                .iter()
                .filter(|s| matches!(s, Some(BuildingType::FuelCellCracker)))
                .count() as f32;

            if cracker_infra > 0.0 {
                // this is a simplified view. it does not account for input resource availability.
                let production_rate = cracker_infra * 1.0; // assuming 1 fuel cell per second per cracker
                *rates.entry(Storable::Good(Good::FuelCells)).or_insert(0.0) += production_rate;
            }

            let farm_infra = buildings
                .slots
                .iter()
                .filter(|s| matches!(s, Some(BuildingType::Farm)))
                .count() as f32;

            if farm_infra > 0.0 {
                // simplified view, does not account for input availability
                let production_rate = farm_infra * 1.0; // 1 food per second per farm
                *rates.entry(Storable::Good(Good::Food)).or_insert(0.0) += production_rate;
            }
        }
        rates
    }
}

/// returns the dynamic, local credit value for a single unit of a resource on a specific entity.
pub fn get_local_price(world: &World, entity_id: EntityId, resource: Storable) -> f64 {
    let base_price = get_resource_base_price(resource);

    let celestial_data = match world.celestial_data.get(&entity_id) {
        Some(data) => data,
        None => return base_price, // not a celestial body, return base price
    };

    let (stockpile, monthly_demand) = match resource {
        Storable::Raw(raw_resource) => (
            celestial_data.stocks.get(&resource).copied().unwrap_or(0.0),
            celestial_data
                .demands
                .get(&Storable::Raw(raw_resource))
                .copied()
                .unwrap_or(0.0),
        ),
        Storable::Good(_) => {
            // for now, goods don't have demand, so they trade at base price
            return base_price;
        }
    };

    const BUFFER_MONTHS: f32 = 3.0;
    // add a small epsilon to demand to avoid division by zero if demand is zero
    let demand_for_ratio = monthly_demand + 1e-6;
    let ratio = stockpile / (demand_for_ratio * BUFFER_MONTHS);

    // price is inversely proportional to supply/demand ratio
    let price_modifier = 1.0 / ratio.max(0.1); // prevent extreme multipliers

    (base_price * price_modifier as f64).clamp(base_price * 0.25, base_price * 4.0)
}

/// returns the base credit value for a single unit of a resource.
pub fn get_resource_base_price(resource: Storable) -> f64 {
    match resource {
        Storable::Raw(raw) => match raw {
            RawResource::Metals => 1.0,
            RawResource::Crystals => 5.0,
            RawResource::Organics => 2.0,
            RawResource::Volatiles => 1.5,
            RawResource::Isotopes => 10.0,
            RawResource::RareExotics => 20.0,
            RawResource::Microbes => 3.0,
            RawResource::DarkMatter => 100.0,
            RawResource::NobleGases => 4.0,
        },
        Storable::Good(good) => match good {
            Good::FuelCells => 2.0,
            Good::Food => 2.5,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buildings::{BuildingType, EntityBuildings, PLANET_SLOTS};
    use crate::world::types::{CelestialBodyData, RawResource, Storable};
    use std::collections::HashMap;

    fn create_test_data(
        mines: usize,
    ) -> (
        HashMap<EntityId, EntityType>,
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
        yields.insert(RawResource::Metals, 1.2);
        yields.insert(RawResource::Crystals, 0.4);
        yields.insert(RawResource::Organics, 0.8);

        celestial_data_map.insert(
            entity_id,
            CelestialBodyData {
                population: 1_000_000.0,
                yields,
                stocks: HashMap::new(),
                demands: HashMap::new(),
                credits: 0.0,
            },
        );

        let mut entity_types = HashMap::new();
        entity_types.insert(entity_id, EntityType::Planet);

        (entity_types, buildings_map, celestial_data_map)
    }

    #[test]
    fn test_resource_system_update() {
        let (entity_types, buildings, mut celestial_data) = create_test_data(1);
        let mut resource_system = ResourceSystem::default();

        resource_system.update(
            RESOURCE_INTERVAL_SECONDS,
            &entity_types,
            &buildings,
            &mut celestial_data,
        );

        let interval_f32 = RESOURCE_INTERVAL_SECONDS as f32;
        let stocks = &celestial_data.get(&1).unwrap().stocks;
        assert_eq!(
            *stocks.get(&Storable::Raw(RawResource::Metals)).unwrap(),
            1.0 * 1.0 * 1.2 * interval_f32
        );
        assert_eq!(
            *stocks.get(&Storable::Raw(RawResource::Crystals)).unwrap(),
            1.0 * 1.0 * 0.4 * interval_f32
        );
        assert_eq!(
            *stocks.get(&Storable::Raw(RawResource::Organics)).unwrap(),
            1.0 * 1.0 * 0.8 * interval_f32
        );
    }
}
