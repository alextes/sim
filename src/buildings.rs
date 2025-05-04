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
