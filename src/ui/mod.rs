//! native egui ui: hud panels, attached foldouts, independent utility windows,
//! and modal action flows. replaces the old hand-drawn sdl `interface` module.

mod menus;
mod state;

pub(crate) use state::BodyDialogs;
pub use state::{BodyDialog, UiState};

use egui::{Align2, Color32, Pos2, Rect, Vec2};

use crate::app::GameState;
use crate::control_state::ControlState;
use crate::palette;
use crate::sim_clock::SimClock;
use crate::viewport::Viewport;
use crate::world::types::{EntityType, Good, RawResource, Spaceport, Storable};
use crate::world::{EntityId, World};

const BODY_FOLDOUT_WIDTH: f32 = 280.0;
const BODY_FOLDOUT_ESTIMATED_HEIGHT: f32 = 290.0;
const BODY_FOLDOUT_MARGIN: f32 = 12.0;

/// build the whole frame's ui.
pub fn build_ui(
    ctx: &egui::Context,
    world: &mut World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    ui_state: &mut UiState,
    clock: &SimClock,
    viewport: &Viewport,
) {
    let state = game_state.clone();

    // hud is shown over the world in every in-game state.
    if shows_world(&state) {
        hud_panels(ctx, world, controls, clock, viewport);
        selected_object_panel(ctx, world, controls, game_state, ui_state, viewport);
        menus::owned_bodies_dialog(ctx, world, controls, ui_state);
        menus::body_detail_dialogs(ctx, world, controls, game_state, ui_state);
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
    world: &mut World,
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
                    ui.checkbox(&mut world.enable_civilian_ai, "autonomous civilian AI");
                }
            });
        });
}

fn selected_object_panel(
    ctx: &egui::Context,
    world: &World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    ui_state: &mut UiState,
    viewport: &Viewport,
) {
    if controls.selection.is_empty() {
        return;
    }

    if controls.selection.len() == 1 {
        let selected = controls.selection[0];
        if is_owned_developed_body(world, selected)
            && body_foldout(
                ctx, world, controls, game_state, ui_state, viewport, selected,
            )
        {
            return;
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

fn is_owned_developed_body(world: &World, entity: EntityId) -> bool {
    world.is_player_controlled(entity)
        && matches!(
            world.get_entity_type(entity),
            Some(EntityType::Planet | EntityType::Moon | EntityType::GasGiant)
        )
}

#[derive(Debug, PartialEq)]
struct BodyFoldoutData {
    name: String,
    body_type: EntityType,
    system_name: String,
    population: f32,
    civilian_credits: f64,
    moon_count: usize,
    resources: Vec<(RawResource, f32)>,
    energy_generation: f32,
    infrastructure_load: Option<(u32, u32)>,
    spaceport: Option<Spaceport>,
}

fn body_foldout_data(world: &World, body: EntityId) -> Option<BodyFoldoutData> {
    if !is_owned_developed_body(world, body) {
        return None;
    }

    let name = world.get_entity_name(body)?;
    let body_type = world.get_entity_type(body)?;
    let system_name = world
        .find_star_for_entity(body)
        .and_then(|star_id| world.get_entity_name(star_id))
        .unwrap_or_else(|| "unknown".to_string());
    let data = world.celestial_data.get(&body)?;
    let mut resources: Vec<_> = data
        .yields
        .iter()
        .map(|(&resource, &grade)| (resource, grade))
        .collect();
    resources.sort_by_key(|(resource, _)| *resource);
    let infrastructure_load = world
        .infrastructure_capacity(body)
        .map(|capacity| (capacity.allocated(), capacity.limit));

    Some(BodyFoldoutData {
        name,
        body_type,
        system_name,
        population: data.population,
        civilian_credits: data.credits,
        moon_count: world.direct_moon_count(body),
        resources,
        energy_generation: world.energy_generation_for_body(body),
        infrastructure_load,
        spaceport: (body_type == EntityType::Planet)
            .then(|| world.spaceport_for_planet(body))
            .flatten(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BodyFoldoutPlacement {
    position: Pos2,
    pivot: Align2,
    connector: [Pos2; 3],
}

fn body_foldout_placement(body_top: Pos2, screen: Rect) -> BodyFoldoutPlacement {
    let right_space = screen.right() - body_top.x;
    let left_space = body_top.x - screen.left();
    let open_right = right_space >= BODY_FOLDOUT_WIDTH + 72.0 || right_space >= left_space;
    let panel_top = (body_top.y - 44.0).clamp(
        screen.top() + BODY_FOLDOUT_MARGIN,
        (screen.bottom() - BODY_FOLDOUT_ESTIMATED_HEIGHT - BODY_FOLDOUT_MARGIN)
            .max(screen.top() + BODY_FOLDOUT_MARGIN),
    );
    let connector_y = panel_top + 24.0;

    if open_right {
        let panel_left = (body_top.x + 64.0).clamp(
            screen.left() + BODY_FOLDOUT_MARGIN,
            (screen.right() - BODY_FOLDOUT_WIDTH - BODY_FOLDOUT_MARGIN)
                .max(screen.left() + BODY_FOLDOUT_MARGIN),
        );
        let edge = Pos2::new(panel_left - 6.0, connector_y);
        BodyFoldoutPlacement {
            position: Pos2::new(panel_left, panel_top),
            pivot: Align2::LEFT_TOP,
            connector: [
                body_top,
                Pos2::new((body_top.x + edge.x) / 2.0, connector_y),
                edge,
            ],
        }
    } else {
        let panel_right = (body_top.x - 64.0).clamp(
            (screen.left() + BODY_FOLDOUT_WIDTH + BODY_FOLDOUT_MARGIN)
                .min(screen.right() - BODY_FOLDOUT_MARGIN),
            screen.right() - BODY_FOLDOUT_MARGIN,
        );
        let edge = Pos2::new(panel_right + 6.0, connector_y);
        BodyFoldoutPlacement {
            position: Pos2::new(panel_right, panel_top),
            pivot: Align2::RIGHT_TOP,
            connector: [
                body_top,
                Pos2::new((body_top.x + edge.x) / 2.0, connector_y),
                edge,
            ],
        }
    }
}

fn body_foldout(
    ctx: &egui::Context,
    world: &World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    ui_state: &mut UiState,
    viewport: &Viewport,
    body: EntityId,
) -> bool {
    let Some(data) = body_foldout_data(world, body) else {
        return false;
    };
    let Some(screen_position) = visible_entity_screen_position(ctx, world, viewport, body) else {
        return false;
    };
    let pixels_per_point = ctx.pixels_per_point();
    let radius = (world.get_render_size(body) * viewport.world_tile_pixel_size_on_screen() / 2.0)
        .max(4.0) as f32
        / pixels_per_point;
    let body_top = screen_position - egui::vec2(0.0, radius);
    let screen = ctx.content_rect();
    let placement = body_foldout_placement(body_top, screen);

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new(("body_foldout_connector", body)),
    ));
    let connector_stroke = egui::Stroke::new(1.5, palette::OVERLAY2);
    painter.line_segment(
        [placement.connector[0], placement.connector[1]],
        connector_stroke,
    );
    painter.line_segment(
        [placement.connector[1], placement.connector[2]],
        connector_stroke,
    );
    painter.circle_filled(placement.connector[0], 2.5, palette::BLUE);

    egui::Area::new(egui::Id::new(("selected_body_foldout", body)))
        .fixed_pos(placement.position)
        .pivot(placement.pivot)
        .constrain_to(screen)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(BODY_FOLDOUT_WIDTH);
                if controls.track_mode {
                    ui.colored_label(palette::LAVENDER, "tracking");
                }
                ui.horizontal_wrapped(|ui| {
                    ui.heading(&data.name);
                    ui.colored_label(palette::SUBTEXT0, body_type_label(data.body_type));
                });
                egui::Grid::new(("body_foldout_stats", body))
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        foldout_stat(ui, "system", &data.system_name);
                        foldout_stat(
                            ui,
                            "population",
                            format!("{:.2}m", data.population / 1_000_000.0),
                        );
                        foldout_stat(
                            ui,
                            "civilian credits",
                            format!("{:.0}", data.civilian_credits),
                        );
                        foldout_stat(ui, "energy", format!("{:.1}", data.energy_generation));
                        if data.body_type == EntityType::Planet {
                            foldout_stat(ui, "moons", data.moon_count.to_string());
                        }
                        if let Some((allocated, limit)) = data.infrastructure_load {
                            foldout_stat(ui, "infrastructure", format!("{allocated}/{limit}"));
                        }
                        foldout_stat(ui, "resource sites", data.resources.len().to_string());
                        if let Some(spaceport) = &data.spaceport {
                            foldout_stat(ui, "spaceport", spaceport.size.label());
                        }
                    });
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if ui.button("overview [i]").clicked() {
                        ui_state.open_body_dialog(body, BodyDialog::Overview);
                    }
                    if ui.button("logistics [l]").clicked() {
                        ui_state.open_body_dialog(body, BodyDialog::Logistics);
                    }
                    if ui.button("procurement [p]").clicked() {
                        ui_state.open_body_dialog(body, BodyDialog::Procurement);
                    }
                });
                menus::body_actions(ui, world, controls, game_state, body);
            });
        });
    true
}

fn foldout_stat(ui: &mut egui::Ui, label: &str, value: impl Into<egui::WidgetText>) {
    ui.colored_label(palette::OVERLAY1, label);
    ui.label(value);
    ui.end_row();
}

fn body_type_label(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Planet => "planet",
        EntityType::Moon => "moon",
        EntityType::GasGiant => "gas giant",
        EntityType::Star => "star",
        EntityType::Ship => "ship",
    }
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
        let primary_stocks = world
            .get_entity_type(id)
            .and_then(crate::world::types::ConstructionLayer::primary_for)
            .map(|layer| data.ordered_stocks_at(layer))
            .unwrap_or_default();
        if !primary_stocks.is_empty() {
            let stock_label = match world.get_entity_type(id) {
                Some(EntityType::GasGiant) => "upper-atmosphere stocks:",
                _ => "surface stocks:",
            };
            ui.label(stock_label);
            for (storable, amount) in primary_stocks {
                let (label, color) = storable_display(storable);
                ui.colored_label(color, format!("  {label}: {amount:.1}"));
            }
        }
        let orbital_stocks = data.ordered_stocks_at(crate::world::types::ConstructionLayer::Orbit);
        if !orbital_stocks.is_empty() {
            ui.label("orbital stocks:");
            for (storable, amount) in orbital_stocks {
                let (label, color) = storable_display(storable);
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
                let name = infrastructure_type.definition().name;
                ui.colored_label(palette::GRAY, format!("  - {name}: {count}"));
            }
        }
    }

    if let Some(ai) = world.civilian_ai.get(&id) {
        ui.separator();
        ui.label("civilian mining");
        let manual = world.mining_routes.contains_key(&id);
        let active_route = world.active_mining_route(id);
        let opportunity = active_route
            .and_then(|route| world.estimate_mining_route(id, route))
            .or_else(|| {
                (!manual && world.enable_civilian_ai)
                    .then(|| world.best_mining_opportunity(id))
                    .flatten()
            });
        ui.colored_label(
            palette::SUBTEXT0,
            format!(
                "{} · {}",
                civilian_ship_state_label(&ai.state),
                if manual { "manual route" } else { "autonomous" }
            ),
        );
        if let Some(estimate) = opportunity {
            let source = world
                .get_entity_name(estimate.route.target_body)
                .unwrap_or_else(|| "unknown".to_owned());
            let buyer = world
                .get_entity_name(estimate.route.sell_body)
                .unwrap_or_else(|| "unknown".to_owned());
            let (resource, color) = raw_resource_display(estimate.route.resource);
            ui.label(format!("route: {source} -> {buyer}"));
            ui.colored_label(
                color,
                format!(
                    "{resource}: {:.1} @ {:.2}",
                    estimate.sale_quantity, estimate.unit_price
                ),
            );
            ui.colored_label(
                palette::SUBTEXT0,
                format!(
                    "travel {:.1}s · mining {:.1}s @ {:.2}x",
                    estimate.travel_time, estimate.mining_time, estimate.mining_yield
                ),
            );
            ui.colored_label(
                palette::SUBTEXT0,
                format!(
                    "revenue {:.1} · costs {:.1}",
                    estimate.sale_revenue, estimate.operating_cost
                ),
            );
            let profit_color = if estimate.expected_profit > 0.0 {
                palette::GREEN
            } else {
                palette::RED
            };
            ui.colored_label(
                profit_color,
                format!(
                    "profit {:.1} / {:.1}s ({:.2}/s)",
                    estimate.expected_profit, estimate.cycle_time, estimate.profit_per_second
                ),
            );
        } else if active_route.is_some() {
            ui.colored_label(palette::YELLOW, "buyer has no current purchase offer");
        } else if !world.enable_civilian_ai && !manual {
            ui.colored_label(palette::OVERLAY1, "autonomous AI is off [f4]");
        } else {
            ui.colored_label(palette::OVERLAY1, "no profitable route found");
        }
    }
}

fn civilian_ship_state_label(state: &crate::world::components::CivilianShipState) -> &'static str {
    use crate::world::components::CivilianShipState;

    match state {
        CivilianShipState::Idle => "idle",
        CivilianShipState::MovingToMine { .. } => "outbound",
        CivilianShipState::Mining { .. } => "mining",
        CivilianShipState::ReturningToSell { .. } => "delivering",
        CivilianShipState::WaitingToUnload { .. } => "waiting to unload",
    }
}

fn main_menu(ctx: &egui::Context, game_state: &mut GameState, controls: &mut ControlState) {
    let screen = ctx.content_rect();
    let layout = main_menu_layout(screen);
    paint_main_menu_logo(ctx, layout);

    egui::Area::new("main_menu".into())
        .fixed_pos(layout.buttons_position)
        .show(ctx, |ui| {
            ui.set_width(layout.button_size.x);
            ui.spacing_mut().item_spacing = Vec2::new(0.0, layout.button_gap);
            let widgets = &mut ui.style_mut().visuals.widgets;
            for visuals in [
                &mut widgets.inactive,
                &mut widgets.noninteractive,
                &mut widgets.open,
            ] {
                visuals.bg_fill = Color32::TRANSPARENT;
                visuals.weak_bg_fill = Color32::TRANSPARENT;
                visuals.bg_stroke = egui::Stroke::new(1.5, palette::SURFACE2);
            }
            widgets.hovered.bg_fill = Color32::TRANSPARENT;
            widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
            widgets.hovered.bg_stroke = egui::Stroke::new(2.0, palette::BLUE);
            widgets.active.bg_fill = Color32::TRANSPARENT;
            widgets.active.weak_bg_fill = Color32::TRANSPARENT;
            widgets.active.bg_stroke = egui::Stroke::new(2.0, palette::LAVENDER);

            ui.vertical(|ui| {
                let play = egui::Button::new(
                    egui::RichText::new("play")
                        .text_style(egui::TextStyle::Button)
                        .color(palette::TEXT),
                )
                .corner_radius(2);
                if ui.add_sized(layout.button_size, play).clicked() {
                    *game_state = GameState::Playing;
                    controls.paused = false;
                }
                let quit = egui::Button::new(
                    egui::RichText::new("quit")
                        .text_style(egui::TextStyle::Button)
                        .color(palette::SUBTEXT1),
                )
                .corner_radius(2);
                if ui.add_sized(layout.button_size, quit).clicked() {
                    controls.quit_requested = true;
                }
            });
        });
}

#[derive(Clone, Copy, Debug)]
struct MainMenuLayout {
    logo_center: Pos2,
    scale: f32,
    title_position: Pos2,
    buttons_position: Pos2,
    button_size: Vec2,
    button_gap: f32,
}

fn main_menu_layout(screen: Rect) -> MainMenuLayout {
    let scale = (screen.width() / 400.0)
        .min(screen.height() / 300.0)
        .clamp(0.72, 1.25);
    let center = screen.center();
    let button_size = Vec2::new(118.0, 28.0) * scale;
    MainMenuLayout {
        logo_center: center - Vec2::new(0.0, 70.0 * scale),
        scale,
        title_position: center - Vec2::new(0.0, 7.0 * scale),
        buttons_position: Pos2::new(center.x - button_size.x / 2.0, center.y + 20.0 * scale),
        button_size,
        button_gap: 10.0 * scale,
    }
}

fn paint_main_menu_logo(ctx: &egui::Context, layout: MainMenuLayout) {
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("main_menu_logo"),
    ));
    let scale = layout.scale;
    let orbit_rotation = -0.48_f32;
    let orbit_radius = Vec2::new(74.0, 22.0) * scale;
    let mut orbit = Vec::with_capacity(97);
    for point in 0..=96 {
        let angle = std::f32::consts::TAU * point as f32 / 96.0;
        let local = Vec2::new(orbit_radius.x * angle.cos(), orbit_radius.y * angle.sin());
        let rotated = Vec2::new(
            local.x * orbit_rotation.cos() - local.y * orbit_rotation.sin(),
            local.x * orbit_rotation.sin() + local.y * orbit_rotation.cos(),
        );
        orbit.push(layout.logo_center + rotated);
    }
    painter.add(egui::Shape::line(
        orbit,
        egui::Stroke::new(2.25 * scale, palette::SUBTEXT0),
    ));
    painter.circle_stroke(
        layout.logo_center,
        38.0 * scale,
        egui::Stroke::new(2.5 * scale, palette::TEXT),
    );

    let moon_angle = -0.12_f32;
    let moon_local = Vec2::new(
        orbit_radius.x * moon_angle.cos(),
        orbit_radius.y * moon_angle.sin(),
    );
    let moon_offset = Vec2::new(
        moon_local.x * orbit_rotation.cos() - moon_local.y * orbit_rotation.sin(),
        moon_local.x * orbit_rotation.sin() + moon_local.y * orbit_rotation.cos(),
    );
    let moon_center = layout.logo_center + moon_offset;
    painter.circle_filled(moon_center, 5.5 * scale, palette::BLUE);
    painter.circle_stroke(
        moon_center,
        5.5 * scale,
        egui::Stroke::new(1.25 * scale, palette::TEXT),
    );
    painter.text(
        layout.title_position,
        Align2::CENTER_CENTER,
        "sim",
        egui::FontId::new(26.0 * scale, egui::FontFamily::Monospace),
        palette::TEXT,
    );
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
        Storable::Good(Good::ConstructionMaterials) => ("construction materials", palette::LGRAY),
        Storable::Good(Good::Food) => ("food", palette::GREEN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;
    use crate::world::types::{InfrastructureType, RawResource, SpaceportSize};

    #[test]
    fn body_foldout_data_is_sorted_and_uses_current_body_stats() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let planet_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 0.0);
        world.set_player_controlled(planet_id);
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

        let expected_capacity = world.infrastructure_capacity(planet_id).unwrap();
        let data = body_foldout_data(&world, planet_id).unwrap();

        assert_eq!(data.name, "earth");
        assert_eq!(data.body_type, EntityType::Planet);
        assert_eq!(data.system_name, "sol");
        assert_eq!(data.moon_count, 1);
        assert_eq!(
            data.resources,
            vec![(RawResource::Metals, 1.0), (RawResource::Crystals, 2.0)]
        );
        assert_eq!(data.energy_generation, 2.0);
        assert_eq!(
            data.infrastructure_load,
            Some((expected_capacity.allocated(), expected_capacity.limit))
        );
        assert_eq!(data.spaceport.unwrap().size, SpaceportSize::Small);
    }

    #[test]
    fn body_foldout_prefers_the_roomier_side_and_stays_on_screen() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));

        let left = body_foldout_placement(Pos2::new(500.0, 300.0), screen);
        assert_eq!(left.pivot, Align2::RIGHT_TOP);
        assert_eq!(left.position, Pos2::new(436.0, 256.0));
        assert_eq!(left.connector[0], Pos2::new(500.0, 300.0));

        let right = body_foldout_placement(Pos2::new(100.0, 20.0), screen);
        assert_eq!(right.pivot, Align2::LEFT_TOP);
        assert_eq!(right.position, Pos2::new(164.0, 12.0));
        assert_eq!(right.connector[2], Pos2::new(158.0, 36.0));
    }

    #[test]
    fn main_menu_layout_stays_balanced_on_compact_and_wide_screens() {
        for size in [Vec2::new(280.0, 180.0), Vec2::new(600.0, 400.0)] {
            let screen = Rect::from_min_size(Pos2::ZERO, size);
            let layout = main_menu_layout(screen);
            let buttons_right = layout.buttons_position.x + layout.button_size.x;
            let buttons_bottom =
                layout.buttons_position.y + layout.button_size.y * 2.0 + layout.button_gap;

            assert_eq!(layout.logo_center.x, screen.center().x);
            assert!(layout.logo_center.y - 42.0 * layout.scale > screen.top());
            assert!(layout.title_position.y > layout.logo_center.y);
            assert!(layout.title_position.y < layout.buttons_position.y);
            assert!(layout.buttons_position.x >= screen.left());
            assert!(buttons_right <= screen.right());
            assert!(buttons_bottom <= screen.bottom());
        }
    }
}
