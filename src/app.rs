//! the winit application: owns the window, gpu, egui, and game state, and
//! drives the fixed-timestep sim. replaces the old hand-rolled sdl `main` loop.

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::background::BackgroundLayer;
use crate::control_state::ControlState;
use crate::egui_layer::{EguiLayer, EguiPaintTarget};
use crate::game_loop::GameLoop;
use crate::gfx::{GpuState, WorldPrepareContext, WorldRenderer};
use crate::input;
use crate::map_generation;
use crate::palette;
use crate::sim_clock::SimClock;
use crate::ui;
use crate::viewport::{Viewport, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH};
use crate::world::{self, World};

/// fixed simulation timestep (100hz).
pub const SIMULATION_DT: Duration = Duration::from_millis(10);
/// render interval (about 60hz).
pub const RENDER_DT: Duration = Duration::from_nanos(16_666_667);
const ONE_SECOND: Duration = Duration::from_secs(1);

/// the different interaction modes the game can be in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    Playing,
    GameMenu,
    BuildMenu {
        mode: BuildMenuMode,
    },
    ShipyardMenu,
    /// shown when a ship build is rejected (the shipyard body can't afford it).
    ShipyardMenuError {
        message: String,
    },
    PlanetOverview {
        selected: Option<world::EntityId>,
    },
    MiningRouteMenu {
        ship_id: world::EntityId,
        mode: MiningRouteMenuMode,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildMenuMode {
    Main,
    SelectInfrastructure,
    EnterQuantity {
        infrastructure: world::types::InfrastructureType,
        quantity_string: String,
    },
    ConfirmQuote {
        infrastructure: world::types::InfrastructureType,
        amount: u32,
    },
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MiningRouteMenuMode {
    SelectTarget,
    SelectResource {
        target_id: world::EntityId,
    },
    SelectSell {
        target_id: world::EntityId,
        resource: world::types::RawResource,
    },
}

pub struct App {
    // window + gpu are None until `resumed` runs.
    gfx: Option<GpuState>,
    world_renderer: Option<WorldRenderer>,
    egui: Option<EguiLayer>,

    world: World,
    clock: SimClock,
    controls: ControlState,
    viewport: Viewport,
    game_state: GameState,
    background: BackgroundLayer,
}

impl App {
    pub fn new() -> Self {
        Self::with_seed(None)
    }

    /// construct the app, optionally seeding world generation for reproducible
    /// runs. `None` leaves the world's entropy-seeded rng (normal interactive
    /// play).
    pub fn with_seed(seed: Option<u64>) -> Self {
        let mut world = World::default();
        if let Some(seed) = seed {
            world.seed_rng(seed);
        }
        map_generation::populate_initial_galaxy(&mut world);
        // draw the background from the same rng so a seeded run is fully
        // reproducible, starfield included.
        let background = BackgroundLayer::new(&mut world.rng.0);

        let selection = if world.entities.is_empty() {
            vec![]
        } else {
            vec![world.entities[0]]
        };
        let controls = ControlState::new(selection);

        let mut game_loop = GameLoop::new(SIMULATION_DT, RENDER_DT);
        // clear any simulation backlog accumulated during setup.
        let _ = game_loop.step();
        let clock = SimClock::new(game_loop);

        Self {
            gfx: None,
            world_renderer: None,
            egui: None,
            world,
            clock,
            controls,
            viewport: Viewport::default(),
            game_state: GameState::MainMenu,
            background,
        }
    }

    /// render a single frame: clear (world pass) then composite egui on top.
    fn redraw(&mut self) {
        let App {
            gfx,
            world_renderer,
            egui,
            world,
            controls,
            game_state,
            clock,
            viewport,
            background,
            ..
        } = self;
        let (Some(gfx), Some(world_renderer), Some(egui)) =
            (gfx.as_mut(), world_renderer.as_mut(), egui.as_mut())
        else {
            return;
        };

        // the galaxy is only drawn in the in-game states; the main menu shows
        // a bare background.
        let show_world = !matches!(game_state, GameState::MainMenu);
        if show_world {
            world_renderer.prepare(WorldPrepareContext {
                device: &gfx.device,
                queue: &gfx.queue,
                world,
                viewport,
                background,
                controls,
                screen: (gfx.config.width, gfx.config.height),
            });
        }

        let Some(output) = gfx.acquire_frame() else {
            return;
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gfx.create_encoder();

        // world pass: clear to the scene background, then draw the tile world.
        {
            let mut world_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("world pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(palette::clear_color()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if show_world {
                world_renderer.draw(&mut world_pass);
            }
        }

        let (user_buffers, repaint_now) = egui.paint(
            EguiPaintTarget {
                window: &gfx.window,
                device: &gfx.device,
                queue: &gfx.queue,
                encoder: &mut encoder,
                view: &view,
                screen_size_in_pixels: [gfx.config.width, gfx.config.height],
            },
            |ctx| ui::build_ui(ctx, world, controls, game_state, clock, viewport),
        );

        gfx.queue.submit(
            user_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        output.present();
        clock.fps_counter += 1;

        // honor egui's requested repaint cadence (e.g. button hover animations).
        if repaint_now {
            gfx.window.request_redraw();
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // resume can fire more than once; only initialize gpu/window once.
        if self.gfx.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("sim")
            .with_inner_size(winit::dpi::LogicalSize::new(
                INITIAL_WINDOW_WIDTH,
                INITIAL_WINDOW_HEIGHT,
            ));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );

        let gfx = GpuState::new(window.clone());
        let world_renderer = WorldRenderer::new(&gfx.device, &gfx.queue, gfx.config.format);
        let egui = EguiLayer::new(&gfx.device, &window, gfx.config.format);

        self.viewport.screen_pixel_width = gfx.config.width;
        self.viewport.screen_pixel_height = gfx.config.height;

        self.gfx = Some(gfx);
        self.world_renderer = Some(world_renderer);
        self.egui = Some(egui);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // feed egui first; respect what it consumes so ui input doesn't leak to
        // the game. tight borrow scope so the rest of the method can take `self`.
        let consumed = match (self.egui.as_mut(), self.gfx.as_ref()) {
            (Some(egui), Some(gfx)) => {
                let response = egui.on_window_event(&gfx.window, &event);
                if response.repaint {
                    gfx.window.request_redraw();
                }
                response.consumed
            }
            _ => false,
        };

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.resize(size.width, size.height);
                }
                self.viewport.screen_pixel_width = size.width;
                self.viewport.screen_pixel_height = size.height;
            }
            WindowEvent::RedrawRequested => self.redraw(),
            other => {
                if !consumed {
                    let outcome = input::handle_window_event(
                        other,
                        &mut self.viewport,
                        &mut self.world,
                        &mut self.controls,
                        &mut self.game_state,
                    );
                    if outcome.request_redraw {
                        if let Some(gfx) = self.gfx.as_ref() {
                            gfx.window.request_redraw();
                        }
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // the ui / escape handler requests exit via this flag.
        if self.controls.quit_requested {
            event_loop.exit();
            return;
        }

        let now = Instant::now();

        // when paused, prevent simulation backlog from accumulating.
        if self.controls.paused {
            self.clock.game_loop.last_update = now;
        }

        let (steps, should_render) = self.clock.game_loop.step();

        if self.game_state == GameState::Playing && !self.controls.paused {
            for _ in 0..steps {
                self.clock.sim_units_counter += 1;
                self.clock.total_sim_ticks += 1;
                let dt = SIMULATION_DT.as_secs_f64() * self.controls.sim_speed as f64;
                self.world.update(dt, self.clock.total_sim_ticks);
            }
        }

        if now.duration_since(self.clock.last_second) >= ONE_SECOND {
            self.clock.sim_units_per_second = self.clock.sim_units_counter;
            self.clock.sim_units_counter = 0;
            self.clock.fps_per_second = self.clock.fps_counter;
            self.clock.fps_counter = 0;
            self.clock.last_second = now;
        }

        if should_render {
            // tracking camera: keep the selected entity centered.
            if self.controls.track_mode && self.game_state == GameState::Playing {
                if let Some(entity_id) = self.controls.selection.first() {
                    if let Some(loc) = self.world.get_location_f64(*entity_id) {
                        self.viewport.center_on_world(loc);
                    }
                }
            }
            if let Some(gfx) = self.gfx.as_ref() {
                gfx.window.request_redraw();
            }
        }

        let next_sim = self.clock.game_loop.last_update + SIMULATION_DT;
        let next_render = self.clock.game_loop.last_render + RENDER_DT;
        let wake_at = next_sim.min(next_render);
        event_loop.set_control_flow(ControlFlow::WaitUntil(wake_at));
    }
}
