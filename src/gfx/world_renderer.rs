//! world renderer: sprite tiles plus line overlays.

use crate::background::BackgroundLayer;
use crate::control_state::ControlState;
use crate::gfx::line_batch::LineBatch;
use crate::gfx::sprite_batch::SpriteBatch;
use crate::viewport::Viewport;
use crate::world::World;

pub struct WorldRenderer {
    sprite: SpriteBatch,
    lines: LineBatch,
}

pub struct WorldPrepareContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub world: &'a World,
    pub viewport: &'a Viewport,
    pub background: &'a BackgroundLayer,
    pub controls: &'a ControlState,
    pub screen: (u32, u32),
}

impl WorldRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            sprite: SpriteBatch::new(device, queue, surface_format),
            lines: LineBatch::new(device, surface_format),
        }
    }

    pub fn prepare(&mut self, context: WorldPrepareContext<'_>) {
        self.sprite.prepare(
            context.device,
            context.queue,
            context.world,
            context.viewport,
            context.background,
            context.screen,
        );
        self.lines.prepare(
            context.device,
            context.queue,
            context.world,
            context.viewport,
            context.controls,
            context.screen,
        );
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // lanes/orbits behind the tiles, then tiles, then overlays on top.
        self.lines.draw_background(render_pass);
        self.sprite.draw(render_pass);
        self.lines.draw_foreground(render_pass);
    }
}
