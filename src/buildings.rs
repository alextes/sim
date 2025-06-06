#![allow(dead_code)] // TODO remove later

use serde::{Deserialize, Serialize};

/// Types of buildings that can be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    SolarPanel,
    Mine,
}

/// Specifies the category of a building slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotType {
    Ground,
    Orbital,
}

// Constants for slot counts
pub const GROUND_SLOTS: usize = 4;
pub const ORBITAL_SLOTS: usize = 4;

/// Represents the building slots available on an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBuildings {
    pub ground: [Option<BuildingType>; GROUND_SLOTS],
    pub orbital: [Option<BuildingType>; ORBITAL_SLOTS],
    pub has_ground_slots: bool,
}

impl EntityBuildings {
    /// Creates a new set of building slots, specifying if ground slots are present.
    pub fn new(has_ground_slots: bool) -> Self {
        Self {
            ground: Default::default(), // Initializes array with None
            orbital: Default::default(),
            has_ground_slots,
        }
    }

    /// Finds the index of the first empty slot of the specified type.
    pub fn find_first_empty_slot(&self, slot_type: SlotType) -> Option<usize> {
        match slot_type {
            SlotType::Ground => {
                if !self.has_ground_slots {
                    return None; // Cannot search ground if it doesn't exist
                }
                self.ground.iter().position(|&slot| slot.is_none())
            }
            SlotType::Orbital => self.orbital.iter().position(|&slot| slot.is_none()),
        }
    }

    /// Attempts to place a building in the specified slot.
    /// Returns Ok(()) on success, or an error message string on failure.
    pub fn build(
        &mut self,
        slot_type: SlotType,
        slot_index: usize,
        building: BuildingType,
    ) -> Result<(), &'static str> {
        // Validate building placement rules (Solar -> Orbital, Mine -> Ground)
        match (building, slot_type) {
            (BuildingType::SolarPanel, SlotType::Ground) => {
                return Err("solar panels can only be built in orbital slots.");
            }
            (BuildingType::Mine, SlotType::Orbital) => {
                return Err("mines can only be built in ground slots.");
            }
            _ => {} // Valid placement
        }

        match slot_type {
            SlotType::Ground => {
                if !self.has_ground_slots {
                    return Err("cannot build on ground: entity has no ground slots.");
                }
                if slot_index >= GROUND_SLOTS {
                    return Err("invalid ground slot index.");
                }
                if self.ground[slot_index].is_some() {
                    return Err("ground slot is already occupied.");
                }
                self.ground[slot_index] = Some(building);
                Ok(())
            }
            SlotType::Orbital => {
                if slot_index >= ORBITAL_SLOTS {
                    return Err("invalid orbital slot index.");
                }
                if self.orbital[slot_index].is_some() {
                    return Err("orbital slot is already occupied.");
                }
                self.orbital[slot_index] = Some(building);
                Ok(())
            }
        }
    }

    /// Helper to get a display name for a building type.
    pub fn building_name(building: BuildingType) -> &'static str {
        match building {
            BuildingType::SolarPanel => "solar panel",
            BuildingType::Mine => "mine",
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
    }

    #[test]
    fn test_new_entity_buildings() {
        let with_ground = EntityBuildings::new(true);
        assert!(with_ground.has_ground_slots);
        assert_eq!(with_ground.ground.len(), GROUND_SLOTS);
        assert!(with_ground.ground.iter().all(|s| s.is_none()));
        assert_eq!(with_ground.orbital.len(), ORBITAL_SLOTS);
        assert!(with_ground.orbital.iter().all(|s| s.is_none()));

        let without_ground = EntityBuildings::new(false);
        assert!(!without_ground.has_ground_slots);
    }

    #[test]
    fn test_find_first_empty_slot() {
        let mut buildings = EntityBuildings::new(true);
        assert_eq!(buildings.find_first_empty_slot(SlotType::Ground), Some(0));
        assert_eq!(buildings.find_first_empty_slot(SlotType::Orbital), Some(0));

        buildings.ground[0] = Some(BuildingType::Mine);
        assert_eq!(buildings.find_first_empty_slot(SlotType::Ground), Some(1));

        buildings.orbital[0] = Some(BuildingType::SolarPanel);
        assert_eq!(buildings.find_first_empty_slot(SlotType::Orbital), Some(1));

        // fill all ground slots
        for i in 0..GROUND_SLOTS {
            buildings.ground[i] = Some(BuildingType::Mine);
        }
        assert_eq!(buildings.find_first_empty_slot(SlotType::Ground), None);

        // test no ground slots
        let buildings_no_ground = EntityBuildings::new(false);
        assert_eq!(
            buildings_no_ground.find_first_empty_slot(SlotType::Ground),
            None
        );
    }

    #[test]
    fn test_build() {
        let mut buildings = EntityBuildings::new(true);

        // valid: mine on ground
        assert_eq!(
            buildings.build(SlotType::Ground, 0, BuildingType::Mine),
            Ok(())
        );
        assert_eq!(buildings.ground[0], Some(BuildingType::Mine));

        // valid: solar on orbital
        assert_eq!(
            buildings.build(SlotType::Orbital, 0, BuildingType::SolarPanel),
            Ok(())
        );
        assert_eq!(buildings.orbital[0], Some(BuildingType::SolarPanel));

        // invalid: slot occupied
        assert_eq!(
            buildings.build(SlotType::Ground, 0, BuildingType::Mine),
            Err("ground slot is already occupied.")
        );

        // invalid: mine on orbital
        assert_eq!(
            buildings.build(SlotType::Orbital, 1, BuildingType::Mine),
            Err("mines can only be built in ground slots.")
        );

        // invalid: solar on ground
        assert_eq!(
            buildings.build(SlotType::Ground, 1, BuildingType::SolarPanel),
            Err("solar panels can only be built in orbital slots.")
        );

        // invalid: index out of bounds
        assert_eq!(
            buildings.build(SlotType::Ground, GROUND_SLOTS, BuildingType::Mine),
            Err("invalid ground slot index.")
        );
        assert_eq!(
            buildings.build(SlotType::Orbital, ORBITAL_SLOTS, BuildingType::SolarPanel),
            Err("invalid orbital slot index.")
        );

        // invalid: no ground slots
        let mut buildings_no_ground = EntityBuildings::new(false);
        assert_eq!(
            buildings_no_ground.build(SlotType::Ground, 0, BuildingType::Mine),
            Err("cannot build on ground: entity has no ground slots.")
        );
    }
}
