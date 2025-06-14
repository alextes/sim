#![allow(dead_code)] // TODO remove later

use serde::{Deserialize, Serialize};

/// Types of buildings that can be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    SolarPanel,
    Mine,
    Shipyard,
}

// Constants for slot counts
pub const PLANET_SLOTS: usize = 4;
pub const MOON_SLOTS: usize = 2;

/// Represents the building slots available on an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBuildings {
    pub slots: Vec<Option<BuildingType>>,
}

impl EntityBuildings {
    /// Creates a new set of building slots, specifying the number of slots.
    pub fn new(num_slots: usize) -> Self {
        Self {
            slots: vec![None; num_slots],
        }
    }

    /// Finds the index of the first empty slot.
    pub fn find_first_empty_slot(&self) -> Option<usize> {
        self.slots.iter().position(|&slot| slot.is_none())
    }

    /// Attempts to place a building in the specified slot.
    /// Returns Ok(()) on success, or an error message string on failure.
    pub fn build(&mut self, slot_index: usize, building: BuildingType) -> Result<(), &'static str> {
        if slot_index >= self.slots.len() {
            return Err("invalid slot index.");
        }
        if self.slots[slot_index].is_some() {
            return Err("slot is already occupied.");
        }
        self.slots[slot_index] = Some(building);
        Ok(())
    }

    /// Helper to get a display name for a building type.
    pub fn building_name(building: BuildingType) -> &'static str {
        match building {
            BuildingType::SolarPanel => "solar panel",
            BuildingType::Mine => "mine",
            BuildingType::Shipyard => "shipyard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_name() {
        assert_eq!(
            EntityBuildings::building_name(BuildingType::SolarPanel),
            "solar panel"
        );
        assert_eq!(EntityBuildings::building_name(BuildingType::Mine), "mine");
        assert_eq!(
            EntityBuildings::building_name(BuildingType::Shipyard),
            "shipyard"
        );
    }

    #[test]
    fn test_new_entity_buildings() {
        let p_buildings = EntityBuildings::new(PLANET_SLOTS);
        assert_eq!(p_buildings.slots.len(), PLANET_SLOTS);
        assert!(p_buildings.slots.iter().all(|s| s.is_none()));

        let m_buildings = EntityBuildings::new(MOON_SLOTS);
        assert_eq!(m_buildings.slots.len(), MOON_SLOTS);

        let no_buildings = EntityBuildings::new(0);
        assert_eq!(no_buildings.slots.len(), 0);
    }

    #[test]
    fn test_find_first_empty_slot() {
        let mut buildings = EntityBuildings::new(PLANET_SLOTS);
        assert_eq!(buildings.find_first_empty_slot(), Some(0));

        buildings.slots[0] = Some(BuildingType::Mine);
        assert_eq!(buildings.find_first_empty_slot(), Some(1));

        // fill all slots
        for i in 0..PLANET_SLOTS {
            buildings.slots[i] = Some(BuildingType::Mine);
        }
        assert_eq!(buildings.find_first_empty_slot(), None);
    }

    #[test]
    fn test_build() {
        let mut buildings = EntityBuildings::new(PLANET_SLOTS);

        // valid: mine
        assert_eq!(buildings.build(0, BuildingType::Mine), Ok(()));
        assert_eq!(buildings.slots[0], Some(BuildingType::Mine));

        // valid: solar on orbital
        assert_eq!(buildings.build(1, BuildingType::SolarPanel), Ok(()));
        assert_eq!(buildings.slots[1], Some(BuildingType::SolarPanel));

        // invalid: slot occupied
        assert_eq!(
            buildings.build(0, BuildingType::Mine),
            Err("slot is already occupied.")
        );

        // invalid: index out of bounds
        assert_eq!(
            buildings.build(PLANET_SLOTS, BuildingType::Mine),
            Err("invalid slot index.")
        );
    }
}
