//! native egui ui: hud panels + modal menus, replacing the old hand-drawn sdl
//! `interface` module. dispatched by `GameState`; menu buttons mutate game/world
//! state directly (replacing the old key-driven handlers).

mod menus;

use egui::{Align2, Color32, Pos2, Rect, Vec2};

use crate::app::GameState;
use crate::control_state::ControlState;
use crate::palette;
use crate::sim_clock::SimClock;
use crate::viewport::Viewport;
use crate::world::types::{EntityType, Good, RawResource, Spaceport, Storable};
use crate::world::{EntityId, World};

const PLANET_PANEL_WIDTH: f32 = 240.0;
const PLANET_PANEL_ESTIMATED_HEIGHT: f32 = 240.0;
const PLANET_PANEL_GAP: f32 = 24.0;
const PLANET_PANEL_MARGIN: f32 = 8.0;

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
        selected_object_panel(ctx, world, controls, viewport);
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
        GameState::PlanetOverview { selected } => {
            menus::planet_overview(ctx, world, controls, game_state, selected)
        }
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

fn selected_object_panel(
    ctx: &egui::Context,
    world: &World,
    controls: &ControlState,
    viewport: &Viewport,
) {
    if controls.selection.is_empty() {
        return;
    }

    if controls.selection.len() == 1 {
        let selected = controls.selection[0];
        if world.get_entity_type(selected) == Some(EntityType::Planet) {
            if let Some(screen_position) =
                visible_entity_screen_position(ctx, world, viewport, selected)
            {
                floating_planet_panel(ctx, world, controls, selected, screen_position);
                return;
            }
        }
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

#[derive(Debug, PartialEq)]
struct PlanetPanelData {
    name: String,
    system_name: String,
    moon_count: usize,
    resources: Vec<(RawResource, f32)>,
    energy_generation: f32,
    solar_panel_count: u32,
    spaceport: Option<Spaceport>,
}

fn planet_panel_data(world: &World, planet_id: EntityId) -> Option<PlanetPanelData> {
    if world.get_entity_type(planet_id) != Some(EntityType::Planet) {
        return None;
    }

    let name = world.get_entity_name(planet_id)?;
    let system_name = world
        .find_star_for_entity(planet_id)
        .and_then(|star_id| world.get_entity_name(star_id))
        .unwrap_or_else(|| "unknown".to_string());
    let mut resources: Vec<_> = world
        .celestial_data
        .get(&planet_id)
        .map(|data| {
            data.yields
                .iter()
                .map(|(&resource, &grade)| (resource, grade))
                .collect()
        })
        .unwrap_or_default();
    resources.sort_by_key(|(resource, _)| *resource);
    let solar_panel_count = world
        .infrastructure
        .get(&planet_id)
        .map(|infrastructure| {
            infrastructure.get_count(crate::world::types::InfrastructureType::SolarPanel)
        })
        .unwrap_or(0);

    Some(PlanetPanelData {
        name,
        system_name,
        moon_count: world.direct_moon_count(planet_id),
        resources,
        energy_generation: world.energy_generation_for_body(planet_id),
        solar_panel_count,
        spaceport: world.spaceport_for_planet(planet_id),
    })
}

fn visible_entity_screen_position(
    ctx: &egui::Context,
    world: &World,
    viewport: &Viewport,
    entity_id: EntityId,
) -> Option<Pos2> {
    let location = world.get_location_f64(entity_id)?;
    let (screen_x, screen_y) = viewport.world_to_screen_px(location.x, location.y);
    if screen_x < 0.0
        || screen_y < 0.0
        || screen_x > viewport.screen_pixel_width as f64
        || screen_y > viewport.screen_pixel_height as f64
    {
        return None;
    }

    let pixels_per_point = ctx.pixels_per_point();
    Some(Pos2::new(
        screen_x as f32 / pixels_per_point,
        screen_y as f32 / pixels_per_point,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlanetPanelPlacement {
    position: Pos2,
    pivot: Align2,
}

fn planet_panel_placement(planet: Pos2, screen: Rect) -> PlanetPanelPlacement {
    let fits_left =
        planet.x - PLANET_PANEL_GAP - PLANET_PANEL_WIDTH >= screen.left() + PLANET_PANEL_MARGIN;
    let (position_x, pivot) = if fits_left {
        (planet.x - PLANET_PANEL_GAP, Align2::RIGHT_CENTER)
    } else {
        (planet.x + PLANET_PANEL_GAP, Align2::LEFT_CENTER)
    };
    let half_height = PLANET_PANEL_ESTIMATED_HEIGHT / 2.0;
    let min_y = screen.top() + PLANET_PANEL_MARGIN + half_height;
    let max_y = screen.bottom() - PLANET_PANEL_MARGIN - half_height;
    let position_y = if min_y <= max_y {
        planet.y.clamp(min_y, max_y)
    } else {
        screen.center().y
    };

    PlanetPanelPlacement {
        position: Pos2::new(position_x, position_y),
        pivot,
    }
}

fn floating_planet_panel(
    ctx: &egui::Context,
    world: &World,
    controls: &ControlState,
    planet_id: EntityId,
    screen_position: Pos2,
) {
    let Some(data) = planet_panel_data(world, planet_id) else {
        return;
    };
    let screen = ctx.content_rect();
    let placement = planet_panel_placement(screen_position, screen);

    egui::Area::new("selected_planet".into())
        .fixed_pos(placement.position)
        .pivot(placement.pivot)
        .constrain_to(screen)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(PLANET_PANEL_WIDTH);
                if controls.track_mode {
                    ui.colored_label(palette::WHITE, "tracking");
                }
                ui.heading(data.name);
                ui.label(format!("system: {}", data.system_name));
                ui.label(format!("moons: {}", data.moon_count));
                ui.separator();
                ui.label("available resources");
                if data.resources.is_empty() {
                    ui.colored_label(palette::DGRAY, "  (none)");
                } else {
                    for (resource, grade) in data.resources {
                        let (label, color) = raw_resource_display(resource);
                        ui.colored_label(color, format!("  {label}: {grade:.2}"));
                    }
                }
                ui.separator();
                ui.label(format!("energy generation: {:.0}", data.energy_generation));
                ui.label(format!("orbital solar panels: {}", data.solar_panel_count));
                if let Some(spaceport) = data.spaceport {
                    ui.separator();
                    ui.label(format!("spaceport: {}", spaceport.name));
                    ui.label(format!("size: {}", spaceport.size.label()));
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

    if let Some(infrastructure) = world.infrastructure.get(&id) {
        ui.separator();
        ui.label("infrastructure");
        if infrastructure.infra.is_empty() {
            ui.colored_label(palette::DGRAY, "  (none)");
        } else {
            let mut infra: Vec<_> = infrastructure.infra.iter().collect();
            infra.sort_by_key(|(bt, _)| format!("{bt:?}"));
            for (infrastructure_type, count) in infra {
                let name = crate::infrastructure::EntityInfrastructure::infrastructure_name(
                    *infrastructure_type,
                );
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
                egui::Stroke::new(1.0_f32, Color32::from_rgb(72, 76, 90));
            ui.visuals_mut().widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0_f32, Color32::from_rgb(100, 108, 128));
            ui.visuals_mut().widgets.active.bg_stroke =
                egui::Stroke::new(1.0_f32, Color32::from_rgb(118, 128, 148));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;
    use crate::world::types::{InfrastructureType, RawResource, SpaceportSize};

    #[test]
    fn planet_panel_data_is_sorted_and_uses_completed_orbital_infrastructure() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let planet_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 0.0);
        world.spawn_moon("moon".to_string(), planet_id, 2.0, 0.0, 0.0);
        let body = world.celestial_data.get_mut(&planet_id).unwrap();
        body.yields.clear();
        body.yields.insert(RawResource::Crystals, 2.0);
        body.yields.insert(RawResource::Metals, 1.0);
        let infrastructure = world.infrastructure.get_mut(&planet_id).unwrap();
        infrastructure
            .infra
            .insert(InfrastructureType::Spaceport, 1);
        infrastructure
            .infra
            .insert(InfrastructureType::SolarPanel, 2);
        infrastructure.queue_build(InfrastructureType::Spaceport, 2);
        infrastructure.queue_build(InfrastructureType::SolarPanel, 3);

        let data = planet_panel_data(&world, planet_id).unwrap();

        assert_eq!(data.name, "earth");
        assert_eq!(data.system_name, "sol");
        assert_eq!(data.moon_count, 1);
        assert_eq!(
            data.resources,
            vec![(RawResource::Metals, 1.0), (RawResource::Crystals, 2.0)]
        );
        assert_eq!(data.energy_generation, 2.0);
        assert_eq!(data.solar_panel_count, 2);
        assert_eq!(
            data.spaceport,
            Some(Spaceport {
                name: "earth spaceport".to_string(),
                size: SpaceportSize::Small,
            })
        );
    }

    #[test]
    fn planet_panel_prefers_left_and_flips_near_left_edge() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));

        let left = planet_panel_placement(Pos2::new(500.0, 300.0), screen);
        assert_eq!(left.pivot, Align2::RIGHT_CENTER);
        assert_eq!(left.position, Pos2::new(476.0, 300.0));

        let right = planet_panel_placement(Pos2::new(100.0, 20.0), screen);
        assert_eq!(right.pivot, Align2::LEFT_CENTER);
        assert_eq!(right.position.x, 124.0);
        assert_eq!(right.position.y, 128.0);
    }
}
