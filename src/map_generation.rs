use rand::Rng;
use std::f64::consts::TAU;

use crate::buildings::{BuildingType, SlotType};
use crate::location::Point;
use crate::world::World;

const NUM_RANDOM_STARS: usize = 12;
const GALAXY_RADIUS: i32 = 100; // defines the spread of stars

fn generate_star_name<R: Rng>(rng: &mut R) -> String {
    let letter1 = rng.random_range('a'..='z');
    let letter2 = rng.random_range('a'..='z');
    let num1 = rng.random_range(0..=9);
    let num2 = rng.random_range(0..=9);
    format!("{}{}{}{}", letter1, letter2, num1, num2)
}

fn add_sol_system(world: &mut World) {
    // sol at center
    let sol_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
    // earth: complete one orbit (2π) in 60 seconds → angular_velocity = tau / 60
    let earth_id = world.spawn_planet("earth".to_string(), sol_id, 16.0, 0.0, TAU / 60.0);
    // moon: faster orbit around earth, e.g. complete in 5 seconds
    let _moon_id = world.spawn_moon("moon".to_string(), earth_id, 4.0, 0.0, TAU / 5.0);

    // pre-build on earth
    if let Some(earth_buildings) = world.buildings.get_mut(&earth_id) {
        // add a mine to the first available ground slot
        if let Some(ground_slot) = earth_buildings.find_first_empty_slot(SlotType::Ground) {
            earth_buildings
                .build(SlotType::Ground, ground_slot, BuildingType::Mine)
                .expect("failed to build initial mine");
        }
        // add a solar panel to the first available orbital slot
        if let Some(orbital_slot) = earth_buildings.find_first_empty_slot(SlotType::Orbital) {
            earth_buildings
                .build(SlotType::Orbital, orbital_slot, BuildingType::SolarPanel)
                .expect("failed to build initial solar panel");
        }
    }
}

pub fn populate_initial_galaxy<R: Rng>(world: &mut World, rng: &mut R) {
    for _ in 0..NUM_RANDOM_STARS {
        let star_name = generate_star_name(rng);
        let x_pos = rng.random_range(-GALAXY_RADIUS..=GALAXY_RADIUS);
        let y_pos = rng.random_range(-GALAXY_RADIUS..=GALAXY_RADIUS);
        world.spawn_star(star_name, Point { x: x_pos, y: y_pos });
    }

    add_sol_system(world);
}
