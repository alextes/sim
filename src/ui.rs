//! egui ui. stage 1 is a single diagnostic panel proving the stack is wired:
//! live sim counters + buttons that mutate game state. the real panels and
//! modal menus (ported from the old `interface` module) return in stage 3.

use crate::app::GameState;
use crate::control_state::ControlState;
use crate::sim_clock::SimClock;
use crate::viewport::Viewport;
use crate::world::World;

pub fn build_ui(
    ctx: &egui::Context,
    world: &World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    clock: &SimClock,
    viewport: &Viewport,
) {
    egui::Window::new("sim — wgpu hello").show(ctx, |ui| {
        ui.label(format!("state: {game_state:?}"));
        ui.label(format!("tick: {}", clock.total_sim_ticks));
        ui.label(format!("sups: {}", clock.sim_units_per_second));
        ui.label(format!("fps: {}", clock.fps_per_second));
        ui.label(format!("entities: {}", world.entities.len()));
        ui.label(format!("zoom: {:.2}", viewport.zoom));
        ui.label(format!("speed: {}x", controls.sim_speed));
        ui.label(format!(
            "track: {}",
            if controls.track_mode { "on" } else { "off" }
        ));

        let selection = match controls.selection.first() {
            Some(&id) => {
                let name = world.get_entity_name(id).unwrap_or_else(|| "?".to_string());
                format!("{} ({}) +{} more", name, id, controls.selection.len() - 1)
            }
            None => "none".to_string(),
        };
        ui.label(format!("selected: {selection}"));

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("play").clicked() {
                *game_state = GameState::Playing;
                controls.paused = false;
            }
            let pause_label = if controls.paused { "resume" } else { "pause" };
            if ui.button(pause_label).clicked() {
                controls.paused = !controls.paused;
            }
        });
    });
}
