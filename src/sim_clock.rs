//! drives the fixed-timestep simulation and tracks sims-per-second / fps stats.
//!
//! wraps the reusable `GameLoop` and the per-second counters that used to live
//! as locals in the old sdl `main` loop.

use std::time::Instant;

use crate::game_loop::GameLoop;

pub struct SimClock {
    pub game_loop: GameLoop,
    pub total_sim_ticks: u64,
    /// sim steps run since the last per-second reset.
    pub sim_units_counter: u64,
    /// most recent sims-per-second sample.
    pub sim_units_per_second: u64,
    /// frames drawn since the last per-second reset.
    pub fps_counter: u32,
    /// most recent frames-per-second sample.
    pub fps_per_second: u32,
    pub last_second: Instant,
}

impl SimClock {
    pub fn new(game_loop: GameLoop) -> Self {
        Self {
            game_loop,
            total_sim_ticks: 0,
            sim_units_counter: 0,
            sim_units_per_second: 0,
            fps_counter: 0,
            fps_per_second: 0,
            last_second: Instant::now(),
        }
    }
}
