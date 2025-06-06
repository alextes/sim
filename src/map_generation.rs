use rand::Rng;
use std::f64::consts::TAU;

use crate::buildings::BuildingType;
use crate::location::Point;
use crate::world::World;

const NUM_STARS: usize = 64;
const GALAXY_RADIUS: i32 = 600; // defines the spread of stars - increased for more space between systems
const MAX_PLANETS_PER_STAR: usize = 4;

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
        if let Some(ground_slot) = earth_buildings.find_first_empty_slot() {
            earth_buildings
                .build(ground_slot, BuildingType::Mine)
                .expect("failed to build initial mine");
        }
        // add a solar panel to the first available orbital slot
        if let Some(orbital_slot) = earth_buildings.find_first_empty_slot() {
            earth_buildings
                .build(orbital_slot, BuildingType::SolarPanel)
                .expect("failed to build initial solar panel");
        }
    }
}

pub fn populate_initial_galaxy<R: Rng>(world: &mut World, rng: &mut R) {
    let mut star_ids = vec![];
    for _ in 0..NUM_STARS {
        let star_name = generate_star_name(rng);

        let angle = rng.random_range(0.0..TAU);
        // linear distribution of radius sample: r = R * U, (U in [0,1])
        // this results in an areal density proportional to 1/r, i.e., denser towards the center.
        let radius_sample = rng.random_range(0.0..1.0f64);
        let radius = GALAXY_RADIUS as f64 * radius_sample;

        let x_pos = (radius * angle.cos()).round() as i32;
        let y_pos = (radius * angle.sin()).round() as i32;
        let star_id = world.spawn_star(star_name, Point { x: x_pos, y: y_pos });
        star_ids.push(star_id);
    }

    add_sol_system(world);

    // generate some planets for the other stars
    for &star_id in &star_ids {
        let num_planets = rng.random_range(0..=MAX_PLANETS_PER_STAR);
        let star_name = world.get_entity_name(star_id).unwrap_or_default();
        let mut last_radius = rng.random_range(4.0..8.0);

        for i in 0..num_planets {
            let planet_name = format!("{}-{}", star_name, i + 1);
            let radius = last_radius + rng.random_range(5.0..10.0);
            let initial_angle = rng.random_range(0.0..TAU);
            // slower for further planets
            let angular_velocity = rng.random_range(0.05..0.2) / (radius / 10.0);

            world.spawn_planet(
                planet_name,
                star_id,
                radius,
                initial_angle,
                angular_velocity,
            );
            last_radius = radius;
        }
    }

    // generate visual star lanes between stars
    world.generate_star_lanes();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_generate_star_name() {
        let mut rng = StdRng::seed_from_u64(42);
        let name1 = generate_star_name(&mut rng);
        assert_eq!(name1.len(), 4);
        assert!(name1.chars().next().unwrap().is_ascii_alphabetic());
        assert!(name1.chars().nth(1).unwrap().is_ascii_alphabetic());
        assert!(name1.chars().nth(2).unwrap().is_ascii_digit());
        assert!(name1.chars().nth(3).unwrap().is_ascii_digit());

        let name2 = generate_star_name(&mut rng);
        assert_ne!(name1, name2);
        assert_eq!(name2.len(), 4);
    }

    #[test]
    fn test_add_sol_system() {
        let mut world = World::default();
        add_sol_system(&mut world);

        // Check that sol, earth, moon were created
        let sol_id = world
            .iter_entities()
            .find(|&id| world.get_entity_name(id) == Some("sol".to_string()))
            .unwrap();
        let earth_id = world
            .iter_entities()
            .find(|&id| world.get_entity_name(id) == Some("earth".to_string()))
            .unwrap();
        let moon_id = world
            .iter_entities()
            .find(|&id| world.get_entity_name(id) == Some("moon".to_string()))
            .unwrap();

        assert_eq!(world.get_render_glyph(sol_id), '*');
        assert_eq!(world.get_render_glyph(earth_id), 'p');
        assert_eq!(world.get_render_glyph(moon_id), 'm');

        // Check earth has buildings
        let buildings = world.buildings.get(&earth_id).unwrap();
        assert!(buildings
            .slots
            .iter()
            .any(|s| s == &Some(BuildingType::Mine)));
        assert!(buildings
            .slots
            .iter()
            .any(|s| s == &Some(BuildingType::SolarPanel)));
    }

    #[test]
    fn test_populate_initial_galaxy() {
        let mut world = World::default();
        let mut rng = StdRng::seed_from_u64(42);
        populate_initial_galaxy(&mut world, &mut rng);

        // NUM_STARS from this function + 1 star from add_sol_system
        let star_count = world
            .iter_entities()
            .filter(|&id| world.get_render_glyph(id) == '*')
            .count();
        assert_eq!(star_count, NUM_STARS + 1);

        // sol system entities (3) + NUM_STARS + planets
        let planet_count = world
            .iter_entities()
            .filter(|&id| world.get_render_glyph(id) == 'p')
            .count();
        // 1 for earth
        assert!(planet_count >= 1);

        // star lanes should be generated
        assert!(!world.lanes.is_empty());
    }
}
