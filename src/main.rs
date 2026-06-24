// temporary for the sdl->wgpu migration: deleting the old rendering/ui/input
// consumers leaves a batch of sim methods dormant until stage 2/3 rewire them.
// remove this once the egui ui is rebuilt (stage 3).
#![allow(dead_code)]

mod app;
mod background;
mod buildings;
mod command;
mod control_state;
mod egui_layer;
mod game_loop;
mod gfx;
mod input;
mod location;
mod map_generation;
mod palette;
mod ships;
mod sim_clock;
mod tileset;
mod ui;
mod viewport;
mod world;

use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use winit::event_loop::{ControlFlow, EventLoop};

// re-exported so `crate::SIMULATION_DT` keeps resolving for the simulation code.
pub use app::SIMULATION_DT;

fn main() {
    // default deps (winit/wgpu/naga) to warn so they don't flood the log, but
    // keep our own crate at debug. RUST_LOG still overrides everything.
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env_lossy()
        .add_directive("sim=debug".parse().expect("valid log directive"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    info!("starting sim");

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = app::App::new();
    event_loop
        .run_app(&mut app)
        .expect("event loop terminated with an error");
}
