//! egui integration: context + winit platform state + wgpu renderer.
//!
//! kept separate from `GpuState` so the (future) world sprite pass and the egui
//! pass hold disjoint borrows during a frame.

use winit::window::Window;

pub struct EguiLayer {
    pub ctx: egui::Context,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
}

impl EguiLayer {
    pub fn new(
        device: &wgpu::Device,
        window: &Window,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let ctx = egui::Context::default();
        let viewport_id = ctx.viewport_id();
        let state = egui_winit::State::new(
            ctx.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: true,
                predictable_texture_filtering: false,
            },
        );
        Self {
            ctx,
            state,
            renderer,
        }
    }
}
