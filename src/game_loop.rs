use std::time::{Duration, Instant};

/// Maximum number of simulation steps to catch up in one tick
const MAX_SIM_STEPS_PER_TICK: usize = 10;

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
        // accumulate all missed simulation ticks, but cap at MAX_SIM_STEPS_PER_TICK
        let mut steps = 0;
        while now.duration_since(self.last_update) >= self.sim_dt && steps < MAX_SIM_STEPS_PER_TICK
        {
            self.last_update += self.sim_dt;
            steps += 1;
        }
        // If we were more than MAX_SIM_STEPS_PER_TICK behind, drop the extra backlog
        if now.duration_since(self.last_update) >= self.sim_dt {
            self.last_update = now;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_simulation_steps_and_render() {
        let sim_dt = Duration::from_millis(10);
        let render_dt = Duration::from_millis(100);
        let mut gl = GameLoop::new(sim_dt, render_dt);
        // simulate being behind by 3 simulation steps and ready to render
        gl.last_update = gl.last_update - sim_dt * 3;
        gl.last_render = gl.last_render - render_dt;
        let (steps, should_render) = gl.step();
        assert_eq!(steps, 3);
        assert!(should_render);
    }

    #[test]
    fn test_simulation_step_cap() {
        let sim_dt = Duration::from_millis(10);
        let render_dt = Duration::from_millis(100);
        let mut gl = GameLoop::new(sim_dt, render_dt);
        // simulate being behind beyond max cap
        gl.last_update = gl.last_update - sim_dt * ((MAX_SIM_STEPS_PER_TICK + 5) as u32);
        let (steps, should_render) = gl.step();
        assert_eq!(steps, MAX_SIM_STEPS_PER_TICK);
        assert!(!should_render);
    }
}
