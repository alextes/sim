mod app;
mod background;
mod cli;
mod command;
mod control_state;
mod egui_layer;
mod game_loop;
mod gfx;
mod infrastructure;
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

use anyhow::Context;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use winit::event_loop::{ControlFlow, EventLoop};

// re-exported so `crate::SIMULATION_DT` keeps resolving for the simulation code.
pub use app::SIMULATION_DT;

fn main() -> anyhow::Result<()> {
    let options = cli::CliOptions::parse(std::env::args().skip(1))?;
    if options.help {
        print!("{}", cli::HELP);
        return Ok(());
    }

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

    let mut app = if let Some(path) = options.screenshot {
        app::App::with_capture(app::CaptureConfig {
            path,
            seed: options.seed.unwrap_or(0),
            ticks: options.ticks,
            width: options.width,
            height: options.height,
            start_playing: options.start != Some(cli::StartState::MainMenu),
        })
    } else {
        let mut app = app::App::with_seed(options.seed);
        if options.start == Some(cli::StartState::Playing) {
            app.start_playing();
        }
        app
    };
    event_loop
        .run_app(&mut app)
        .context("event loop terminated with an error")?;
    app.finish_capture()?;
    Ok(())
}
