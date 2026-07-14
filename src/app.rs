//! the winit application: owns the window, gpu, egui, and game state, and
//! drives the fixed-timestep sim. replaces the old hand-rolled sdl `main` loop.

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context};
use tracing::info;
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

#[derive(Debug)]
pub struct CaptureConfig {
    pub path: PathBuf,
    pub seed: u64,
    pub ticks: u64,
    pub width: u32,
    pub height: u32,
    pub start_playing: bool,
}

struct CaptureRequest {
    path: PathBuf,
    width: u32,
    height: u32,
    warmup_frames_remaining: u8,
    result: Option<anyhow::Result<()>>,
}

struct CaptureBuffer {
    buffer: wgpu::Buffer,
    padded_bytes_per_row: u32,
    unpadded_bytes_per_row: u32,
    width: u32,
    height: u32,
}

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
    capture: Option<CaptureRequest>,
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
            capture: None,
        }
    }

    pub fn with_capture(config: CaptureConfig) -> Self {
        let mut app = Self::with_seed(Some(config.seed));
        if config.start_playing {
            app.start_playing();
        }
        app.controls.paused = true;

        for tick in 1..=config.ticks {
            app.world.update(SIMULATION_DT.as_secs_f64(), tick);
        }
        app.clock.total_sim_ticks = config.ticks;
        app.viewport.screen_pixel_width = config.width;
        app.viewport.screen_pixel_height = config.height;
        app.capture = Some(CaptureRequest {
            path: config.path,
            width: config.width,
            height: config.height,
            warmup_frames_remaining: 1,
            result: None,
        });
        app
    }

    pub fn start_playing(&mut self) {
        self.game_state = GameState::Playing;
        self.controls.paused = false;
    }

    pub fn finish_capture(&mut self) -> anyhow::Result<()> {
        let Some(capture) = self.capture.as_mut() else {
            return Ok(());
        };
        capture
            .result
            .take()
            .ok_or_else(|| anyhow!("game exited before the screenshot was captured"))?
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
            capture,
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
        let screen = gfx
            .capture_texture
            .as_ref()
            .map(|texture| (texture.width(), texture.height()))
            .unwrap_or((gfx.config.width, gfx.config.height));
        let should_capture = capture.as_ref().is_some_and(|request| {
            request.result.is_none() && request.warmup_frames_remaining == 0
        });
        if show_world {
            world_renderer.prepare(WorldPrepareContext {
                device: &gfx.device,
                queue: &gfx.queue,
                world,
                viewport,
                background,
                controls,
                screen,
            });
        }

        let output = if gfx.capture_texture.is_some() {
            None
        } else {
            let Some(output) = gfx.acquire_frame() else {
                return;
            };
            Some(output)
        };
        let target_texture = gfx
            .capture_texture
            .as_ref()
            .or_else(|| output.as_ref().map(|output| &output.texture))
            .expect("render target is available");
        let view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
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
                screen_size_in_pixels: [screen.0, screen.1],
            },
            |ctx| ui::build_ui(ctx, world, controls, game_state, clock, viewport),
        );

        let render_submission = gfx.queue.submit(
            user_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        clock.fps_counter += 1;

        if should_capture {
            let request = capture.as_mut().expect("capture request is pending");
            let result = gfx
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(render_submission),
                    timeout: None,
                })
                .context("waiting for screenshot render")
                .and_then(|_| {
                    let mut copy_encoder = gfx.create_encoder();
                    let buffer = encode_capture(gfx, target_texture, &mut copy_encoder);
                    gfx.queue.submit(std::iter::once(copy_encoder.finish()));
                    save_capture(gfx, buffer, &request.path)
                });
            request.result = Some(result);
            controls.quit_requested = true;
        }

        if let Some(output) = output {
            output.present();
        }

        if let Some(request) = capture.as_mut() {
            if request.result.is_none() && request.warmup_frames_remaining > 0 {
                match gfx
                    .device
                    .poll(wgpu::PollType::wait_indefinitely())
                    .context("waiting for screenshot warm-up frame")
                {
                    Ok(_) => {
                        request.warmup_frames_remaining -= 1;
                        gfx.window.request_redraw();
                    }
                    Err(error) => {
                        request.result = Some(Err(error));
                        controls.quit_requested = true;
                    }
                }
            }
        }

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

        let attributes = Window::default_attributes().with_title("sim");
        let attributes = if let Some(capture) = self.capture.as_ref() {
            attributes.with_inner_size(winit::dpi::PhysicalSize::new(capture.width, capture.height))
        } else {
            attributes.with_inner_size(winit::dpi::LogicalSize::new(
                INITIAL_WINDOW_WIDTH,
                INITIAL_WINDOW_HEIGHT,
            ))
        };
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );

        let capture_size = self
            .capture
            .as_ref()
            .map(|capture| (capture.width, capture.height));
        let gfx = GpuState::new(window.clone(), capture_size);
        let world_renderer = WorldRenderer::new(&gfx.device, &gfx.queue, gfx.config.format);
        let pixels_per_point = self.capture.as_ref().map(|_| 1.0);
        let egui = EguiLayer::new(&gfx.device, &window, gfx.config.format, pixels_per_point);

        if self.capture.is_some() {
            gfx.queue.submit(std::iter::empty());
            if let Err(error) = gfx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .context("initializing screenshot GPU resources")
            {
                if let Some(capture) = self.capture.as_mut() {
                    capture.result = Some(Err(error));
                }
                self.controls.quit_requested = true;
            }
        }

        if self.capture.is_none() {
            self.viewport.screen_pixel_width = gfx.config.width;
            self.viewport.screen_pixel_height = gfx.config.height;
        }

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
                if self.capture.is_none() {
                    self.viewport.screen_pixel_width = size.width;
                    self.viewport.screen_pixel_height = size.height;
                }
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

fn encode_capture(
    gfx: &GpuState,
    texture: &wgpu::Texture,
    encoder: &mut wgpu::CommandEncoder,
) -> CaptureBuffer {
    let width = texture.width();
    let height = texture.height();
    let unpadded_bytes_per_row = width * 4;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(alignment) * alignment;
    let buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screenshot readback"),
        size: u64::from(padded_bytes_per_row) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: None,
            },
        },
        texture.size(),
    );

    CaptureBuffer {
        buffer,
        padded_bytes_per_row,
        unpadded_bytes_per_row,
        width,
        height,
    }
}

fn save_capture(
    gfx: &GpuState,
    capture: CaptureBuffer,
    path: &std::path::Path,
) -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::sync_channel(1);
    capture
        .buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    gfx.device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("waiting for screenshot GPU readback")?;
    receiver
        .recv()
        .context("receiving screenshot GPU readback")?
        .context("mapping screenshot GPU buffer")?;

    let channel_order = match gfx.config.format {
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => [0, 1, 2, 3],
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => [2, 1, 0, 3],
        format => return Err(anyhow!("unsupported screenshot surface format {format:?}")),
    };
    let mapped = capture.buffer.slice(..).get_mapped_range();
    let mut rgba = Vec::with_capacity((capture.width * capture.height * 4) as usize);
    for padded_row in mapped.chunks(capture.padded_bytes_per_row as usize) {
        let row = &padded_row[..capture.unpadded_bytes_per_row as usize];
        for color in row.chunks_exact(4) {
            rgba.extend(channel_order.map(|index| color[index]));
        }
    }
    drop(mapped);
    capture.buffer.unmap();

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating screenshot directory {}", parent.display()))?;
    }
    image::save_buffer(
        path,
        &rgba,
        capture.width,
        capture.height,
        image::ColorType::Rgba8,
    )
    .with_context(|| format!("saving screenshot to {}", path.display()))?;
    info!(path = %path.display(), width = capture.width, height = capture.height, "saved screenshot");
    Ok(())
}
