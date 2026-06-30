//! native egui ui: hud panels + modal menus, replacing the old hand-drawn sdl
//! `interface` module. dispatched by `GameState`; menu buttons mutate game/world
//! state directly (replacing the old key-driven handlers).

mod menus;

use egui::{Align2, Color32, Vec2};

use crate::app::GameState;
use crate::control_state::ControlState;
use crate::palette;
use crate::sim_clock::SimClock;
use crate::viewport::Viewport;
use crate::world::types::{Good, RawResource, Storable};
use crate::world::World;

/// build the whole frame's ui.
pub fn build_ui(
    ctx: &egui::Context,
    world: &mut World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    clock: &SimClock,
    viewport: &Viewport,
) {
    let state = game_state.clone();

    // hud is shown over the world in every in-game state.
    if shows_world(&state) {
        hud_panels(ctx, world, controls, clock, viewport);
        selected_object_panel(ctx, world, controls);
    }

    match state {
        GameState::MainMenu => main_menu(ctx, game_state, controls),
        GameState::Playing => {}
        GameState::GameMenu => game_menu(ctx, game_state, controls),
        GameState::BuildMenu { mode } => menus::build_menu(ctx, world, controls, game_state, &mode),
        GameState::ShipyardMenu => menus::shipyard_menu(ctx, world, controls, game_state, None),
        GameState::ShipyardMenuError { message } => {
            menus::shipyard_menu(ctx, world, controls, game_state, Some(&message))
        }
        GameState::PlanetOverview { .. } => {}
        GameState::MiningRouteMenu { ship_id, mode } => {
            menus::mining_route_menu(ctx, world, game_state, ship_id, &mode)
        }
    }
}

/// the in-game states draw the world (and hud) behind any modal.
fn shows_world(state: &GameState) -> bool {
    !matches!(state, GameState::MainMenu)
}

fn hud_panels(
    ctx: &egui::Context,
    world: &World,
    controls: &ControlState,
    clock: &SimClock,
    viewport: &Viewport,
) {
    // top-left: stardate + credits.
    egui::Area::new("hud_top_left".into())
        .anchor(Align2::LEFT_TOP, Vec2::new(8.0, 8.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                let stardate = clock.total_sim_ticks as f64 / 100.0;
                ui.label(format!("DATE: {stardate:.2}"));
                ui.label(format!("credits: {:.0}", world.player_credits));
            });
        });

    // top-right: sim speed + (optional) debug overlay.
    egui::Area::new("hud_top_right".into())
        .anchor(Align2::RIGHT_TOP, Vec2::new(-8.0, 8.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                if controls.paused {
                    ui.label("SPEED: PAUSED");
                } else {
                    ui.label(format!("SPEED: {}x", controls.sim_speed));
                }
                if controls.debug_enabled {
                    ui.separator();
                    ui.label(format!(
                        "SUPS {} FPS {}",
                        clock.sim_units_per_second, clock.fps_per_second
                    ));
                    ui.label(format!("zoom: {:.2}", viewport.zoom));
                }
            });
        });
}

fn selected_object_panel(ctx: &egui::Context, world: &World, controls: &ControlState) {
    if controls.selection.is_empty() {
        return;
    }
    egui::Area::new("selected_object".into())
        .anchor(Align2::LEFT_BOTTOM, Vec2::new(8.0, -8.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(220.0);
                if controls.selection.len() == 1 {
                    single_selection(ui, world, controls, controls.selection[0]);
                } else {
                    ui.label(format!("selected: {} items", controls.selection.len()));
                    let ships = controls
                        .selection
                        .iter()
                        .filter(|id| world.ships.contains_key(id))
                        .count();
                    if ships > 0 {
                        ui.colored_label(palette::GRAY, format!("- {ships} ships"));
                    }
                }
            });
        });
}

fn single_selection(ui: &mut egui::Ui, world: &World, controls: &ControlState, id: u32) {
    if controls.track_mode {
        ui.colored_label(palette::WHITE, "tracking");
    }
    let name = world.get_entity_name(id).unwrap_or_default();
    ui.label(format!("selected: {name}"));

    if let Some(data) = world.celestial_data.get(&id) {
        if data.population > 0.0 {
            ui.colored_label(palette::GRAY, format!("pop: {:.2}m", data.population));
        }
        if data.credits > 0.0 {
            ui.colored_label(palette::YELLOW, format!("civ credits: {:.0}", data.credits));
        }
        if !data.yields.is_empty() {
            ui.label("yields:");
            let mut yields: Vec<_> = data.yields.iter().collect();
            yields.sort_by_key(|(r, _)| **r);
            for (resource, grade) in yields {
                let (label, color) = raw_resource_display(*resource);
                ui.colored_label(color, format!("  {label}: {grade:.2}"));
            }
        }
        if !data.stocks.is_empty() {
            ui.label("stocks:");
            let mut stocks: Vec<_> = data.stocks.iter().collect();
            stocks.sort_by_key(|(s, _)| **s);
            for (storable, amount) in stocks {
                let (label, color) = storable_display(*storable);
                ui.colored_label(color, format!("  {label}: {amount:.1}"));
            }
        }
    }

    if let Some(buildings) = world.buildings.get(&id) {
        ui.separator();
        ui.label("infrastructure");
        if buildings.infra.is_empty() {
            ui.colored_label(palette::DGRAY, "  (none)");
        } else {
            let mut infra: Vec<_> = buildings.infra.iter().collect();
            infra.sort_by_key(|(bt, _)| format!("{bt:?}"));
            for (building, count) in infra {
                let name = crate::buildings::EntityBuildings::building_name(*building);
                ui.colored_label(palette::GRAY, format!("  - {name}: {count}"));
            }
        }
    }
}

fn main_menu(ctx: &egui::Context, game_state: &mut GameState, controls: &mut ControlState) {
    egui::Area::new("main_menu".into())
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            let button_fill = Color32::from_rgb(30, 32, 40);
            let button_hover = Color32::from_rgb(43, 46, 56);
            let button_active = Color32::from_rgb(55, 58, 70);
            let button_text = Color32::from_rgb(210, 216, 225);
            let button_size = Vec2::new(180.0, 44.0);

            ui.spacing_mut().item_spacing = Vec2::new(0.0, 12.0);
            ui.visuals_mut().widgets.inactive.bg_fill = button_fill;
            ui.visuals_mut().widgets.hovered.bg_fill = button_hover;
            ui.visuals_mut().widgets.active.bg_fill = button_active;
            ui.visuals_mut().widgets.inactive.fg_stroke.color = button_text;
            ui.visuals_mut().widgets.hovered.fg_stroke.color = button_text;
            ui.visuals_mut().widgets.active.fg_stroke.color = button_text;
            ui.visuals_mut().widgets.inactive.bg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(72, 76, 90));
            ui.visuals_mut().widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(100, 108, 128));
            ui.visuals_mut().widgets.active.bg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(118, 128, 148));

            ui.vertical_centered(|ui| {
                if ui
                    .add_sized(button_size, egui::Button::new("play"))
                    .clicked()
                {
                    *game_state = GameState::Playing;
                    controls.paused = false;
                }
                if ui
                    .add_sized(button_size, egui::Button::new("quit"))
                    .clicked()
                {
                    controls.quit_requested = true;
                }
            });
        });
}

fn game_menu(ctx: &egui::Context, game_state: &mut GameState, controls: &mut ControlState) {
    centered_window(ctx, "game menu", |ui| {
        if ui.button("resume").clicked() {
            *game_state = GameState::Playing;
            controls.paused = false;
        }
        if ui.button("quit game").clicked() {
            controls.quit_requested = true;
        }
    });
}

/// shared modal window: centered, fixed, non-collapsible.
fn centered_window(ctx: &egui::Context, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Window::new(title)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .show(ctx, |ui| add(ui));
}

/// display label + color for a raw resource (ported from the old sdl panel).
fn raw_resource_display(resource: RawResource) -> (&'static str, Color32) {
    match resource {
        RawResource::Metals => ("metals", palette::LGRAY),
        RawResource::Organics => ("organics", palette::LGREEN),
        RawResource::Crystals => ("crystals", palette::CYAN),
        RawResource::Isotopes => ("isotopes", palette::MAGENTA),
        RawResource::Microbes => ("microbes", palette::YELLOW),
        RawResource::Volatiles => ("volatiles", palette::ORANGE),
        RawResource::RareExotics => ("exotics", palette::LRED),
        RawResource::DarkMatter => ("dark matter", palette::DGRAY),
        RawResource::NobleGases => ("noble gases", palette::LBLUE),
    }
}

/// display label + color for any storable.
fn storable_display(storable: Storable) -> (&'static str, Color32) {
    match storable {
        Storable::Raw(r) => raw_resource_display(r),
        Storable::Good(Good::FuelCells) => ("fuel cells", palette::RED),
        Storable::Good(Good::Food) => ("food", palette::GREEN),
    }
}
