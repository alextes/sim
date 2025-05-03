use std::time::{Duration, Instant};

use tracing::debug;

/// A helper for driving a fixed-time-step simulation and a separate
/// render interval within a single loop.
pub struct GameLoop {
    sim_dt: Duration,
    render_dt: Duration,
    pub last_update: Instant,
    pub last_render: Instant,
}

impl GameLoop {
    /// Construct a new GameLoop with given simulation and render rates.
    pub fn new(sim_dt: Duration, render_dt: Duration) -> Self {
        let now = Instant::now();
        Self {
            sim_dt,
            render_dt,
            last_update: now,
            last_render: now,
        }
    }

    /// Returns (number of simulation steps to run, and whether to render).
    pub fn step(&mut self) -> (usize, bool) {
        let now = Instant::now();
        // accumulate all missed simulation ticks
        let mut steps = 0;
        while now.duration_since(self.last_update) >= self.sim_dt {
            self.last_update += self.sim_dt;
            steps += 1;
        }
        // check if it's time to render
        let mut should_render = false;
        if now.duration_since(self.last_render) >= self.render_dt {
            self.last_render += self.render_dt;
            should_render = true;
        }
        (steps, should_render)
    }
}
