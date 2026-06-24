//! wgpu device/surface state. replaces the old sdl canvas setup.
//!
//! holds the window plus the gpu resources needed to acquire and present a
//! frame. frame orchestration (world pass + egui pass) lives in `app::redraw`,
//! which needs game + egui state alongside these resources.

pub mod atlas;
pub mod sprite_batch;

use std::sync::Arc;

use winit::window::Window;

use sprite_batch::SpriteBatch;

pub struct GpuState {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub sprite: SpriteBatch,
}

impl GpuState {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create wgpu surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("failed to find a suitable gpu adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("sim device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .expect("failed to request gpu device");

        let caps = surface.get_capabilities(&adapter);
        // egui-wgpu 0.34 wants a gamma-space (non-srgb) target; it srgb-encodes
        // in its own shader. our clear color and the stage-2 sprite shader must
        // do the same, so pick a plain unorm format here.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                )
            })
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let sprite = SpriteBatch::new(&device, &queue, config.format);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            sprite,
        }
    }

    /// reconfigure the surface for a new size. ignores zero-size (minimize).
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// reconfigure at the current size, e.g. after a lost/outdated surface.
    pub fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }
}
