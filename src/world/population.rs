use crate::world::World;

impl World {
    pub(super) fn update_population(&mut self, dt_seconds: f64) {
        // population growth
        const ANNUAL_POPULATION_GROWTH_RATE: f32 = 0.015; // 1.5% per year
        const SECONDS_PER_YEAR: f32 = 365.0; // assuming 1 day = 1 second
        for data in self.celestial_data.values_mut() {
            if data.population > 0.0 {
                let growth = data.population
                    * ANNUAL_POPULATION_GROWTH_RATE
                    * (dt_seconds as f32 / SECONDS_PER_YEAR);
                data.population += growth;
            }
        }
    }
}
