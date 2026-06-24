//! egui integration: context + winit platform state + wgpu renderer.
//!
//! kept separate from `GpuState` so the world pass and the egui pass hold
//! disjoint borrows during a frame.

use winit::event::WindowEvent;
use winit::window::Window;

pub struct EguiLayer {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

pub struct EguiPaintTarget<'a> {
    pub window: &'a Window,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub view: &'a wgpu::TextureView,
    pub screen_size_in_pixels: [u32; 2],
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

    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn paint(
        &mut self,
        target: EguiPaintTarget<'_>,
        mut add_ui: impl FnMut(&egui::Context),
    ) -> (Vec<wgpu::CommandBuffer>, bool) {
        let raw_input = self.state.take_egui_input(target.window);
        let full_output = self.ctx.run_ui(raw_input, |ui| {
            add_ui(ui.ctx());
        });
        self.state
            .handle_platform_output(target.window, full_output.platform_output);

        let repaint_now = full_output
            .viewport_output
            .values()
            .any(|v| v.repaint_delay.is_zero());

        let paint_jobs = self
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: target.screen_size_in_pixels,
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(target.device, target.queue, *id, image_delta);
        }
        let user_buffers = self.renderer.update_buffers(
            target.device,
            target.queue,
            target.encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let mut egui_pass = target
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target.view,
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
            self.renderer
                .render(&mut egui_pass, &paint_jobs, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        (user_buffers, repaint_now)
    }
}
