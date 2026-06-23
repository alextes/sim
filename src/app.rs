//! the winit application: owns the window, gpu, egui, and game state, and
//! drives the fixed-timestep sim. replaces the old hand-rolled sdl `main` loop.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::warn;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::control_state::ControlState;
use crate::egui_layer::EguiLayer;
use crate::game_loop::GameLoop;
use crate::gfx::GpuState;
use crate::input;
use crate::map_generation;
use crate::palette;
use crate::sim_clock::SimClock;
use crate::ui;
use crate::viewport::{Viewport, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH};
use crate::world::{self, World};

/// fixed simulation timestep (100hz).
pub const SIMULATION_DT: Duration = Duration::from_millis(10);
/// render interval (10hz).
pub const RENDER_DT: Duration = Duration::from_millis(100);
const ONE_SECOND: Duration = Duration::from_secs(1);

/// the different interaction modes the game can be in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Intro,
    MainMenu,
    Playing,
    GameMenu,
    BuildMenu {
        mode: BuildMenuMode,
    },
    ShipyardMenu,
    ShipyardMenuError {
        message: String,
    },
    MiningRouteMenu {
        ship_id: world::EntityId,
        mode: MiningRouteMenuMode,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildMenuMode {
    Main,
    SelectBuilding,
    EnterQuantity {
        building: world::types::BuildingType,
        quantity_string: String,
    },
    ConfirmQuote {
        building: world::types::BuildingType,
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
    egui: Option<EguiLayer>,

    world: World,
    clock: SimClock,
    controls: ControlState,
    viewport: Viewport,
    game_state: GameState,
    intro_start: Instant,
}

impl App {
    pub fn new() -> Self {
        let mut world = World::default();
        let mut rng = rand::rng();
        map_generation::populate_initial_galaxy(&mut world, &mut rng);

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
            egui: None,
            world,
            clock,
            controls,
            viewport: Viewport::default(),
            game_state: GameState::Intro,
            intro_start: Instant::now(),
        }
    }

    /// render a single frame: clear (world pass) then composite egui on top.
    fn redraw(&mut self) {
        let App {
            gfx,
            egui,
            world,
            controls,
            game_state,
            clock,
            viewport,
            ..
        } = self;
        let (Some(gfx), Some(egui)) = (gfx.as_mut(), egui.as_mut()) else {
            return;
        };

        let output = match gfx.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                gfx.reconfigure();
                return;
            }
            // transient states (window not yet front, minimized, gpu busy):
            // skip this frame quietly; the render cadence will retry.
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => {
                return;
            }
            other => {
                warn!("surface unavailable: {other:?}");
                return;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        // world pass: stage 1 just clears to the scene background color. the
        // gpu sprite batch (stage 2) draws here before the egui pass.
        {
            let _world_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        // egui pass.
        let raw_input = egui.state.take_egui_input(&gfx.window);
        let full_output = egui.ctx.run_ui(raw_input, |ui| {
            ui::build_ui(ui.ctx(), world, controls, game_state, clock, viewport);
        });
        egui.state
            .handle_platform_output(&gfx.window, full_output.platform_output);
        let paint_jobs = egui
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gfx.config.width, gfx.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };
        for (id, image_delta) in &full_output.textures_delta.set {
            egui.renderer
                .update_texture(&gfx.device, &gfx.queue, *id, image_delta);
        }
        let user_buffers = egui.renderer.update_buffers(
            &gfx.device,
            &gfx.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let mut egui_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            egui.renderer
                .render(&mut egui_pass, &paint_jobs, &screen_descriptor);
        }
        for id in &full_output.textures_delta.free {
            egui.renderer.free_texture(id);
        }

        gfx.queue.submit(
            user_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        output.present();
        clock.fps_counter += 1;

        // honor egui's requested repaint cadence (e.g. button hover animations).
        if full_output
            .viewport_output
            .values()
            .any(|v| v.repaint_delay.is_zero())
        {
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
        let egui = EguiLayer::new(&gfx.device, &window, gfx.config.format);

        self.viewport.screen_pixel_width = gfx.config.width;
        self.viewport.screen_pixel_height = gfx.config.height;

        self.gfx = Some(gfx);
        self.egui = Some(egui);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // feed egui first; respect what it consumes so ui input doesn't leak to
        // the game. tight borrow scope so the rest of the method can take `self`.
        let consumed = match (self.egui.as_mut(), self.gfx.as_ref()) {
            (Some(egui), Some(gfx)) => {
                let response = egui.state.on_window_event(&gfx.window, &event);
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
                    input::handle_window_event(
                        other,
                        &mut self.viewport,
                        &mut self.world,
                        &mut self.controls,
                        &mut self.game_state,
                    );
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if self.game_state == GameState::Intro
            && self.intro_start.elapsed() >= Duration::from_millis(100)
        {
            self.game_state = GameState::MainMenu;
        }

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
                    if let Some(loc) = self.world.get_location(*entity_id) {
                        self.viewport.center_on_entity(loc.x, loc.y);
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
