//! wgpu window, device, and surface state.
//!
//! holds the window plus the gpu resources needed to acquire and present a
//! frame. frame orchestration (world pass + egui pass) lives in `app::redraw`,
//! which needs game + egui state alongside these resources.

mod atlas;
mod line_batch;
mod sprite_batch;
mod world_renderer;

use std::sync::Arc;

use tracing::warn;
use winit::window::Window;

pub use world_renderer::{WorldPrepareContext, WorldRenderer};

pub struct GpuState {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub capture_texture: Option<wgpu::Texture>,
}

impl GpuState {
    pub fn new(window: Arc<Window>, capture_size: Option<(u32, u32)>) -> Self {
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
        let capture_texture = capture_size.map(|(width, height)| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("screenshot render target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            capture_texture,
        }
    }

    pub fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            })
    }

    pub fn acquire_frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Some(texture),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.reconfigure();
                None
            }
            // transient states (window not yet front, minimized, gpu busy):
            // skip this frame quietly; the render cadence will retry.
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => None,
            other => {
                warn!("surface unavailable: {other:?}");
                None
            }
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
