//! independent utility windows plus stateful build, shipyard, and mining-route
//! menus. button presses update ui state or issue world commands.

use crate::app::{BuildMenuMode, GameState, MiningRouteMenuMode};
use crate::command::Command;
use crate::control_state::ControlState;
use crate::infrastructure::{player_buildable_infrastructure, InfrastructureCategory};
use crate::palette;
use crate::ships::{buildable_ships, ShipBuildShortfall, ShipBuildable};
use crate::ui::{BodyDialog, BodyDialogs, UiState};
use crate::world::components::MiningRoute;
use crate::world::types::{
    ConstructionLayer, EntityType, Good, InfrastructureType, ProcurementKey, ProcurementPolicy,
    RawResource, Storable,
};
use crate::world::{get_resource_base_price, EntityId, World};

use super::{centered_window, raw_resource_display, storable_display};

const PROCUREMENT_RESOURCES: &[Storable] = &[
    Storable::Raw(RawResource::Metals),
    Storable::Raw(RawResource::Organics),
    Storable::Raw(RawResource::Crystals),
    Storable::Raw(RawResource::Volatiles),
    Storable::Good(Good::FuelCells),
    Storable::Good(Good::ConstructionMaterials),
    Storable::Good(Good::Food),
];

pub fn owned_bodies_dialog(
    ctx: &egui::Context,
    world: &World,
    controls: &mut ControlState,
    ui_state: &mut UiState,
) {
    if !ui_state.owned_bodies_open {
        return;
    }

    let bodies = world.owned_body_overview_entities();
    ui_state.normalize_owned_body_cursor(&bodies);
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
        ui_state.move_owned_body_cursor(&bodies, true);
    }
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
        ui_state.move_owned_body_cursor(&bodies, false);
    }
    let open_highlighted =
        ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    let mut open = true;
    let mut body_to_open = open_highlighted
        .then(|| ui_state.owned_body_cursor())
        .flatten();
    let mut close_requested = false;
    let screen = ctx.content_rect();
    egui::Window::new("owned bodies")
        .id(egui::Id::new("owned_bodies"))
        .open(&mut open)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .frame(egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin::symmetric(6, 3)))
        .default_pos(egui::Pos2::new((screen.right() - 334.0).max(24.0), 24.0))
        .default_width(286.0)
        .show(ctx, |ui| {
            ui.set_min_width(270.0);
            ui.scope(|ui| {
                ui.spacing_mut().button_padding = egui::Vec2::new(3.0, 0.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("owned bodies")
                            .text_style(egui::TextStyle::Small)
                            .strong()
                            .color(palette::TEXT),
                    );
                    ui.label(
                        egui::RichText::new("[o]")
                            .text_style(egui::TextStyle::Small)
                            .color(palette::OVERLAY1),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close = egui::Button::new(
                            egui::RichText::new("x").text_style(egui::TextStyle::Small),
                        );
                        if ui.add_sized([16.0, 14.0], close).clicked() {
                            close_requested = true;
                        }
                    });
                });
            });
            ui.separator();

            if bodies.is_empty() {
                ui.colored_label(palette::DGRAY, "no owned bodies");
                return;
            }

            egui::ScrollArea::vertical()
                .max_height((screen.height() - 150.0).clamp(100.0, 420.0))
                .show(ui, |ui| {
                    for &body in &bodies {
                        let name = world.get_entity_name(body).unwrap_or_default();
                        let system = world
                            .find_star_for_entity(body)
                            .and_then(|star| world.get_entity_name(star))
                            .unwrap_or_else(|| "unknown".to_owned());
                        let body_type = world
                            .get_entity_type(body)
                            .map(body_type_label)
                            .unwrap_or("body");
                        let selected = ui_state.owned_body_cursor() == Some(body);
                        let (rect, response) = ui.allocate_exact_size(
                            egui::Vec2::new(ui.available_width(), 48.0),
                            egui::Sense::click(),
                        );
                        let (fill, stroke) = if selected {
                            (palette::SURFACE1, palette::BLUE)
                        } else if response.hovered() {
                            (palette::SURFACE0, palette::SURFACE2)
                        } else {
                            (palette::MANTLE, palette::SURFACE0)
                        };
                        ui.painter().rect(
                            rect,
                            3.0,
                            fill,
                            egui::Stroke::new(1.0, stroke),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            rect.left_top() + egui::Vec2::new(10.0, 7.0),
                            egui::Align2::LEFT_TOP,
                            name,
                            egui::TextStyle::Button.resolve(ui.style()),
                            palette::TEXT,
                        );
                        ui.painter().text(
                            rect.left_top() + egui::Vec2::new(10.0, 27.0),
                            egui::Align2::LEFT_TOP,
                            format!("{system} · {body_type}"),
                            egui::TextStyle::Small.resolve(ui.style()),
                            palette::SUBTEXT0,
                        );
                        if response.clicked() {
                            response.surrender_focus();
                            ui_state.set_owned_body_cursor(body);
                            controls.selection = vec![body];
                            body_to_open = Some(body);
                        }
                        ui.add_space(4.0);
                    }
                });
            ui.colored_label(palette::OVERLAY1, "↑/↓ navigate · enter open");
        });
    ui_state.owned_bodies_open = open && !close_requested;
    if let Some(body) = body_to_open {
        controls.selection = vec![body];
        ui_state.open_body_dialog(body, BodyDialog::Overview);
    }
}

pub fn body_detail_dialogs(
    ctx: &egui::Context,
    world: &mut World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    ui_state: &mut UiState,
) {
    let owned_bodies = world.owned_body_overview_entities();
    for (body, mut dialogs) in ui_state.body_dialogs() {
        if !owned_bodies.contains(&body) {
            ui_state.update_body_dialogs(body, BodyDialogs::default());
            continue;
        }
        if dialogs.overview {
            dialogs.overview = body_overview_dialog(ctx, world, controls, game_state, body);
        }
        if dialogs.logistics {
            dialogs.logistics = body_logistics_dialog(ctx, world, body);
        }
        if dialogs.procurement {
            dialogs.procurement = body_procurement_dialog(ctx, world, body);
        }
        ui_state.update_body_dialogs(body, dialogs);
    }
}

fn body_overview_dialog(
    ctx: &egui::Context,
    world: &World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    body: EntityId,
) -> bool {
    let name = world.get_entity_name(body).unwrap_or_default();
    let Some(entity_type) = world.get_entity_type(body) else {
        return false;
    };
    let mut open = true;
    egui::Window::new(format!("{name} overview [i]"))
        .id(egui::Id::new(("body_overview", body)))
        .open(&mut open)
        .default_pos(body_dialog_position(body, egui::Pos2::new(24.0, 96.0)))
        .default_size(body_dialog_size(ctx, 440.0, 420.0))
        .show(ctx, |ui| {
            let detail_height = (ui.available_height() - 52.0).max(120.0);
            egui::ScrollArea::vertical()
                .max_height(detail_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    overview_tab(ui, world, body, entity_type);
                });
            ui.separator();
            body_actions(ui, world, controls, game_state, body);
        });
    open
}

fn body_logistics_dialog(ctx: &egui::Context, world: &World, body: EntityId) -> bool {
    let name = world.get_entity_name(body).unwrap_or_default();
    let Some(entity_type) = world.get_entity_type(body) else {
        return false;
    };
    let mut open = true;
    egui::Window::new(format!("{name} logistics [l]"))
        .id(egui::Id::new(("body_logistics", body)))
        .open(&mut open)
        .default_pos(body_dialog_position(body, egui::Pos2::new(480.0, 96.0)))
        .default_size(body_dialog_size(ctx, 420.0, 360.0))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    logistics_tab(ui, world, body, entity_type);
                });
        });
    open
}

fn body_procurement_dialog(ctx: &egui::Context, world: &mut World, body: EntityId) -> bool {
    let name = world.get_entity_name(body).unwrap_or_default();
    let Some(entity_type) = world.get_entity_type(body) else {
        return false;
    };
    let mut open = true;
    egui::Window::new(format!("{name} procurement [p]"))
        .id(egui::Id::new(("body_procurement", body)))
        .open(&mut open)
        .default_pos(body_dialog_position(body, egui::Pos2::new(220.0, 180.0)))
        .default_size(body_dialog_size(ctx, 620.0, 480.0))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    procurement_controls(ui, world, body, entity_type);
                });
        });
    open
}

fn body_dialog_position(body: EntityId, base: egui::Pos2) -> egui::Pos2 {
    let offset = (body % 6) as f32 * 18.0;
    base + egui::vec2(offset, offset)
}

fn body_dialog_size(ctx: &egui::Context, max_width: f32, max_height: f32) -> egui::Vec2 {
    let screen = ctx.content_rect();
    egui::vec2(
        (screen.width() - 48.0).max(260.0).min(max_width),
        (screen.height() - 96.0).max(200.0).min(max_height),
    )
}

pub(super) fn body_actions(
    ui: &mut egui::Ui,
    world: &World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    body: EntityId,
) {
    ui.horizontal_wrapped(|ui| {
        let can_build = world.get_entity_type(body) == Some(EntityType::Planet);
        if ui
            .add_enabled(can_build, egui::Button::new("build [b]"))
            .clicked()
        {
            controls.selection = vec![body];
            *game_state = GameState::BuildMenu {
                mode: BuildMenuMode::Main,
            };
        }
        let has_shipyard = world
            .infrastructure
            .get(&body)
            .is_some_and(|infrastructure| {
                infrastructure.operational_units_in_category(InfrastructureCategory::Shipbuilding)
                    > 0
            });
        if ui
            .add_enabled(has_shipyard, egui::Button::new("shipyard [s]"))
            .clicked()
        {
            controls.selection = vec![body];
            *game_state = GameState::ShipyardMenu;
        }
    });
}

fn overview_tab(ui: &mut egui::Ui, world: &World, body: EntityId, entity_type: EntityType) {
    ui.label("current stats");
    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::Grid::new(("planet_stats", body))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                stat_row(ui, "body class", body_type_label(entity_type));
                let system = world
                    .find_star_for_entity(body)
                    .and_then(|star| world.get_entity_name(star))
                    .unwrap_or_else(|| "unknown".to_owned());
                stat_row(ui, "system", system);
                if let Some(data) = world.celestial_data.get(&body) {
                    stat_row(
                        ui,
                        "population",
                        format!("{:.2}m", data.population / 1_000_000.0),
                    );
                    stat_row(ui, "civilian credits", format!("{:.0}", data.credits));
                }
                stat_row(
                    ui,
                    "energy generation",
                    format!("{:.1}", world.energy_generation_for_body(body)),
                );
                if entity_type == EntityType::Planet {
                    stat_row(ui, "moons", world.direct_moon_count(body).to_string());
                }
                if let Some(capacity) = world.infrastructure_capacity(body) {
                    stat_row(
                        ui,
                        "infrastructure load",
                        format!("{}/{}", capacity.allocated(), capacity.limit),
                    );
                    stat_row(ui, "completed units", capacity.completed.to_string());
                    stat_row(ui, "queued units", capacity.queued.to_string());
                }
            });
    });

    if let Some(data) = world.celestial_data.get(&body) {
        ui.add_space(8.0);
        ui.label("resource profile");
        egui::Frame::group(ui.style()).show(ui, |ui| {
            if data.yields.is_empty() {
                ui.colored_label(palette::DGRAY, "no known yields");
                return;
            }
            let mut yields: Vec<_> = data.yields.iter().collect();
            yields.sort_by_key(|(resource, _)| **resource);
            egui::Grid::new(("planet_yields", body))
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for (resource, grade) in yields {
                        let (label, color) = raw_resource_display(*resource);
                        ui.colored_label(color, label);
                        ui.label(format!("grade {grade:.2}"));
                        ui.end_row();
                    }
                });
        });
    }

    infrastructure_overview(ui, world, body);
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: impl Into<egui::WidgetText>) {
    ui.colored_label(palette::DGRAY, label);
    ui.label(value);
    ui.end_row();
}

fn infrastructure_overview(ui: &mut egui::Ui, world: &World, body: EntityId) {
    let Some(infrastructure) = world.infrastructure.get(&body) else {
        return;
    };
    ui.add_space(8.0);
    ui.label("infrastructure");
    egui::Frame::group(ui.style()).show(ui, |ui| {
        if infrastructure.infra.is_empty() {
            ui.colored_label(palette::DGRAY, "none");
            return;
        }
        let statuses = infrastructure.maintenance_statuses();
        let total_upkeep: f64 = statuses
            .iter()
            .map(|status| status.upkeep_per_interval)
            .sum();
        let total_arrears: f64 = statuses.iter().map(|status| status.arrears).sum();
        ui.label(format!(
            "upkeep {total_upkeep:.1} credits/month | arrears {total_arrears:.1}"
        ));
        for status in statuses {
            let name = status.infrastructure_type.definition().name;
            let state = if status.active { "active" } else { "inactive" };
            let color = if status.active {
                palette::GRAY
            } else {
                palette::RED
            };
            ui.colored_label(
                color,
                format!(
                    "{name} x{} | {state} | {:.1}/month | {:.1} arrears",
                    status.completed_units, status.upkeep_per_interval, status.arrears
                ),
            );
        }
    });
}

fn logistics_tab(ui: &mut egui::Ui, world: &World, body: EntityId, entity_type: EntityType) {
    storage_and_docks(ui, world, body, entity_type);
    stockpiles(ui, world, body, entity_type);
    construction_status(ui, world, body, entity_type);
}

fn stockpiles(ui: &mut egui::Ui, world: &World, body: EntityId, entity_type: EntityType) {
    let Some(data) = world.celestial_data.get(&body) else {
        return;
    };
    let primary_stocks = ConstructionLayer::primary_for(entity_type)
        .map(|layer| data.ordered_stocks_at(layer))
        .unwrap_or_default();
    let orbital_stocks = data.ordered_stocks_at(ConstructionLayer::Orbit);

    ui.separator();
    ui.label("stockpiles");
    if primary_stocks.is_empty() && orbital_stocks.is_empty() {
        ui.colored_label(palette::DGRAY, "empty");
        return;
    }
    if !primary_stocks.is_empty() {
        let stock_label = match entity_type {
            EntityType::GasGiant => "upper atmosphere",
            _ => "surface",
        };
        ui.colored_label(palette::DGRAY, stock_label);
        for (storable, amount) in primary_stocks {
            let (label, color) = storable_display(storable);
            ui.colored_label(color, format!("{label}: {amount:.1}"));
        }
    }
    if !orbital_stocks.is_empty() {
        ui.colored_label(palette::DGRAY, "orbit");
        for (storable, amount) in orbital_stocks {
            let (label, color) = storable_display(storable);
            ui.colored_label(color, format!("{label}: {amount:.1}"));
        }
    }
}

fn storage_and_docks(ui: &mut egui::Ui, world: &World, body: EntityId, entity_type: EntityType) {
    ui.label("storage and docks");
    for &layer in ConstructionLayer::available_for(entity_type) {
        let Some((capacity, used)) = world.storage_capacity(body, layer) else {
            continue;
        };
        let free = (capacity - used).max(0.0);
        let accepting = world
            .infrastructure
            .get(&body)
            .map(|infrastructure| infrastructure.accepting_storage_capacity(layer))
            .unwrap_or(0.0);
        let text = format!("{layer}: {used:.1}/{capacity:.1}, {free:.1} free");
        if accepting < capacity {
            ui.colored_label(palette::RED, format!("{text} (new deposits paused)"));
        } else {
            ui.label(text);
        }
    }
    if let Some((throughput, berths)) = world.orbital_dock_capacity(body) {
        let waiting = world.ships_waiting_to_unload(body);
        ui.label(format!(
            "unload: {throughput:.1}/interval | {berths} berths | {waiting} waiting"
        ));
    }
}

fn procurement_controls(
    ui: &mut egui::Ui,
    world: &mut World,
    body: EntityId,
    entity_type: EntityType,
) {
    ui.label("procurement");
    ui.colored_label(
        palette::DGRAY,
        "reserve | price ceiling | optional monthly cap",
    );

    let mut updates = Vec::new();
    for &layer in ConstructionLayer::available_for(entity_type) {
        egui::CollapsingHeader::new(format!("{layer} procurement"))
            .default_open(true)
            .show(ui, |ui| {
                for &resource in PROCUREMENT_RESOURCES {
                    if let Some(update) = procurement_control_row(ui, world, body, layer, resource)
                    {
                        updates.push(update);
                    }
                }
            });
    }

    if let Some(data) = world.celestial_data.get_mut(&body) {
        for (key, policy) in updates {
            if let Some(policy) = policy {
                data.procurement_policies.insert(key, policy);
            } else {
                data.procurement_policies.remove(&key);
            }
        }
    }
}

fn procurement_control_row(
    ui: &mut egui::Ui,
    world: &World,
    body: EntityId,
    layer: ConstructionLayer,
    resource: Storable,
) -> Option<(ProcurementKey, Option<ProcurementPolicy>)> {
    let key = ProcurementKey { layer, resource };
    let existing = world
        .celestial_data
        .get(&body)
        .and_then(|data| data.procurement_policies.get(&key).copied());
    let automatic_target = if resource == Storable::Good(Good::ConstructionMaterials) {
        world.construction_procurement_target(body, layer)
    } else {
        0.0
    };
    let mut policy = existing.unwrap_or(ProcurementPolicy {
        enabled: automatic_target > 0.0,
        reserve_target: automatic_target.max(100.0),
        maximum_unit_price: get_resource_base_price(resource) * 4.0,
        periodic_spend_cap: None,
    });
    policy.reserve_target = policy.reserve_target.max(automatic_target);
    let stock = world
        .celestial_data
        .get(&body)
        .map(|data| data.amount_at(layer, resource))
        .unwrap_or(0.0);
    let quote = world.procurement_quote(body, key);
    let mut changed = false;
    let mut reset = false;

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            changed |= ui.checkbox(&mut policy.enabled, "buy").changed();
            let (label, color) = storable_display(resource);
            ui.colored_label(color, label);
            ui.label(format!("stock {stock:.1}"));
            if let Some(quote) = quote {
                ui.label(format!(
                    "offer {:.1} @ {:.2}",
                    quote.wanted_quantity, quote.unit_price
                ));
            } else {
                ui.colored_label(palette::DGRAY, "offer closed");
            }
            reset = ui
                .add_enabled(existing.is_some(), egui::Button::new("reset"))
                .clicked();
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(18.0);
            ui.label("reserve");
            changed |= ui
                .add(egui::DragValue::new(&mut policy.reserve_target).speed(1.0))
                .changed();
            ui.label("max price");
            changed |= ui
                .add(egui::DragValue::new(&mut policy.maximum_unit_price).speed(0.25))
                .changed();

            let mut cap_enabled = policy.periodic_spend_cap.is_some();
            let mut cap = policy.periodic_spend_cap.unwrap_or_else(|| {
                (policy.reserve_target as f64 * policy.maximum_unit_price).max(1.0)
            });
            changed |= ui.checkbox(&mut cap_enabled, "spend cap").changed();
            if cap_enabled {
                changed |= ui.add(egui::DragValue::new(&mut cap).speed(1.0)).changed();
            } else {
                ui.colored_label(palette::DGRAY, "none");
            }
            policy.periodic_spend_cap = cap_enabled.then_some(cap.max(0.0));
        });
    });
    policy.reserve_target = policy.reserve_target.max(0.0);
    policy.maximum_unit_price = policy.maximum_unit_price.max(0.0);

    if reset {
        Some((key, None))
    } else if changed {
        Some((key, Some(policy)))
    } else {
        None
    }
}

fn construction_status(ui: &mut egui::Ui, world: &World, body: EntityId, entity_type: EntityType) {
    let Some(infrastructure) = world.infrastructure.get(&body) else {
        return;
    };
    ui.separator();
    ui.label("construction queue");
    if infrastructure.build_queue.is_empty() {
        ui.colored_label(palette::DGRAY, "(empty)");
        return;
    }

    if let Some((infrastructure_type, _)) = infrastructure.build_queue.front() {
        if let Some(layer) = infrastructure_type.construction_layer(entity_type) {
            let remaining = (infrastructure_type
                .definition()
                .construction_material_cost()
                - infrastructure.construction_progress)
                .max(0.0);
            let available = world
                .celestial_data
                .get(&body)
                .map(|data| data.amount_at(layer, Storable::Good(Good::ConstructionMaterials)))
                .unwrap_or(0.0);
            let shortfall = (remaining - available).max(0.0);
            if shortfall > 0.0 {
                ui.colored_label(
                    palette::RED,
                    format!(
                        "blocked: {:.1} more construction materials needed at {layer}",
                        shortfall
                    ),
                );
            } else {
                ui.label(format!("front item supplied at {layer}"));
            }
        }
    }

    for &layer in ConstructionLayer::available_for(entity_type) {
        let lifetime = infrastructure.remaining_construction_material(entity_type, layer);
        if lifetime > 0.0 {
            ui.label(format!(
                "lifetime project cost remaining at {layer}: {lifetime:.1} construction materials"
            ));
        }
    }
    for (infrastructure_type, count) in &infrastructure.build_queue {
        ui.label(format!(
            "{} x{count}",
            infrastructure_type.definition().name
        ));
    }
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

pub fn build_menu(
    ctx: &egui::Context,
    world: &mut World,
    controls: &ControlState,
    game_state: &mut GameState,
    mode: &BuildMenuMode,
) {
    let Some(entity_id) = controls.selection.first().copied() else {
        *game_state = GameState::Playing;
        return;
    };
    if !world.is_player_controlled(entity_id)
        || world.get_entity_type(entity_id) != Some(EntityType::Planet)
    {
        *game_state = GameState::Playing;
        return;
    }
    let name = world.get_entity_name(entity_id).unwrap_or_default();

    centered_window(ctx, "build menu", |ui| {
        ui.heading(name.as_str());
        match mode {
            BuildMenuMode::Main => build_main(ui, world, game_state, entity_id),
            BuildMenuMode::SelectInfrastructure => build_select(ui, world, game_state, entity_id),
            BuildMenuMode::EnterQuantity {
                infrastructure,
                quantity_string,
            } => build_quantity(
                ui,
                world,
                game_state,
                entity_id,
                *infrastructure,
                quantity_string,
            ),
            BuildMenuMode::ConfirmQuote {
                infrastructure,
                amount,
            } => build_confirm(ui, world, game_state, entity_id, *infrastructure, *amount),
        }
    });
}

fn build_main(ui: &mut egui::Ui, world: &World, game_state: &mut GameState, entity_id: EntityId) {
    if let Some(capacity) = world.infrastructure_capacity(entity_id) {
        ui.label(format!(
            "infrastructure capacity: {}/{} ({} queued)",
            capacity.allocated(),
            capacity.limit,
            capacity.queued
        ));
    }
    ui.separator();
    ui.label("construction queue:");
    if let Some(infrastructure) = world.infrastructure.get(&entity_id) {
        if infrastructure.build_queue.is_empty() {
            ui.colored_label(palette::DGRAY, "  (empty)");
        } else {
            for (infrastructure_type, count) in &infrastructure.build_queue {
                ui.label(format!(
                    "  - {} x{count}",
                    infrastructure_type.definition().name
                ));
            }
        }
    }
    ui.separator();
    if ui.button("add to queue").clicked() {
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::SelectInfrastructure,
        };
    }
    if ui.button("close").clicked() {
        *game_state = GameState::Playing;
    }
}

fn build_select(ui: &mut egui::Ui, world: &World, game_state: &mut GameState, entity_id: EntityId) {
    ui.label("select infrastructure:");
    for definition in player_buildable_infrastructure() {
        let infrastructure = definition.infrastructure_type;
        let available = world.can_queue_player_infrastructure(entity_id, infrastructure, 1);
        let has_capacity = world
            .infrastructure_capacity(entity_id)
            .is_some_and(|capacity| capacity.can_fit(definition.capacity_use));
        let label = if !has_capacity {
            format!("{} (no capacity)", definition.name)
        } else if infrastructure == InfrastructureType::Spaceport && !available {
            "spaceport (maximum size)".to_string()
        } else {
            definition.name.to_string()
        };
        if ui
            .add_enabled(available, egui::Button::new(label))
            .clicked()
        {
            *game_state = GameState::BuildMenu {
                mode: BuildMenuMode::EnterQuantity {
                    infrastructure,
                    quantity_string: String::new(),
                },
            };
        }
    }
    if ui.button("back").clicked() {
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::Main,
        };
    }
}

fn build_quantity(
    ui: &mut egui::Ui,
    world: &World,
    game_state: &mut GameState,
    entity_id: EntityId,
    infrastructure: InfrastructureType,
    quantity_string: &str,
) {
    ui.label(format!(
        "infrastructure: {}",
        infrastructure.definition().name
    ));
    if infrastructure == InfrastructureType::Spaceport {
        ui.label(format!(
            "units available before large: {}",
            world.remaining_spaceport_units(entity_id)
        ));
    }
    let mut qty = quantity_string.to_string();
    let response = ui.add(egui::TextEdit::singleline(&mut qty).hint_text("quantity"));
    let amount = qty.trim().parse::<u32>().ok().filter(|n| *n > 0);
    let required_capacity = amount
        .map(|amount| infrastructure.definition().capacity_for(amount))
        .unwrap_or(0);
    let capacity = world.infrastructure_capacity(entity_id);
    if let Some(capacity) = capacity {
        ui.label(format!(
            "capacity: {} available, {required_capacity} required",
            capacity.remaining()
        ));
    }
    let eligible = amount.is_some_and(|amount| {
        world.can_queue_player_infrastructure(entity_id, infrastructure, amount)
    });
    let exceeds_capacity =
        amount.is_some() && !capacity.is_some_and(|capacity| capacity.can_fit(required_capacity));
    if exceeds_capacity {
        ui.colored_label(palette::RED, "quantity exceeds available capacity");
    } else if infrastructure == InfrastructureType::Spaceport && amount.is_some() && !eligible {
        ui.colored_label(
            palette::RED,
            "quantity exceeds the available spaceport size",
        );
    }
    let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let confirm = ui
        .add_enabled(eligible, egui::Button::new("confirm"))
        .clicked();
    let cancel = ui.button("cancel").clicked();

    if cancel {
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::Main,
        };
    } else if (enter || confirm) && eligible {
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::ConfirmQuote {
                infrastructure,
                amount: amount.unwrap(),
            },
        };
    } else if qty != quantity_string {
        // persist the edited text back into the state machine for next frame.
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::EnterQuantity {
                infrastructure,
                quantity_string: qty,
            },
        };
    }
}

fn build_confirm(
    ui: &mut egui::Ui,
    world: &mut World,
    game_state: &mut GameState,
    entity_id: EntityId,
    infrastructure: InfrastructureType,
    amount: u32,
) {
    ui.label(format!(
        "build {amount}x {}?",
        infrastructure.definition().name
    ));
    ui.label("lifetime project cost:");
    let costs = infrastructure.definition().scaled_costs(amount);
    let build_layer = world
        .get_entity_type(entity_id)
        .and_then(|entity_type| infrastructure.construction_layer(entity_type));
    if let Some(layer) = build_layer {
        ui.label(format!("required at: {layer}"));
        if let (Some(entity_type), Some(existing)) = (
            world.get_entity_type(entity_id),
            world.infrastructure.get(&entity_id),
        ) {
            let mut projected = existing.clone();
            projected.queue_build(infrastructure, amount);
            let staged = projected.staged_construction_material(entity_type, layer);
            let current = world
                .celestial_data
                .get(&entity_id)
                .map(|body| {
                    body.amount_at(
                        layer,
                        crate::world::types::Storable::Good(
                            crate::world::types::Good::ConstructionMaterials,
                        ),
                    )
                })
                .unwrap_or(0.0);
            ui.label(format!(
                "procurement horizon: {staged:.1} staged ({:.1} outstanding)",
                (staged - current).max(0.0)
            ));
        }
    }
    let required_capacity = infrastructure.definition().capacity_for(amount);
    if let Some(capacity) = world.infrastructure_capacity(entity_id) {
        let color = if capacity.can_fit(required_capacity) {
            palette::WHITE
        } else {
            palette::RED
        };
        ui.colored_label(
            color,
            format!(
                "capacity: {required_capacity} required ({} available)",
                capacity.remaining()
            ),
        );
    }
    for cost in costs {
        let storable = cost.resource;
        let have = world
            .celestial_data
            .get(&entity_id)
            .and_then(|data| build_layer.map(|layer| data.amount_at(layer, storable)))
            .unwrap_or(0.0);
        let color = if have < cost.quantity {
            palette::RED
        } else {
            palette::WHITE
        };
        ui.colored_label(
            color,
            format!("  {:.1} {storable} (have {have:.1})", cost.quantity),
        );
    }
    ui.horizontal(|ui| {
        let eligible = world.can_queue_player_infrastructure(entity_id, infrastructure, amount);
        if ui.add_enabled(eligible, egui::Button::new("yes")).clicked() {
            world.add_command(Command::Build {
                entity_id,
                infrastructure_type: infrastructure,
                amount,
            });
            *game_state = GameState::Playing;
        }
        if ui.button("no").clicked() {
            *game_state = GameState::BuildMenu {
                mode: BuildMenuMode::Main,
            };
        }
    });
}

pub fn shipyard_menu(
    ctx: &egui::Context,
    world: &mut World,
    controls: &ControlState,
    game_state: &mut GameState,
    error: Option<&str>,
) {
    let shipyard_id = controls.selection.first().copied();
    centered_window(ctx, "shipyard", |ui| {
        if let Some(message) = error {
            ui.colored_label(palette::RED, "build error:");
            ui.label(message);
            if ui.button("continue").clicked() {
                *game_state = GameState::Playing;
            }
            return;
        }
        let Some(shipyard_id) = shipyard_id else {
            *game_state = GameState::Playing;
            return;
        };
        ui.label("build ship?");
        for buildable in buildable_ships() {
            let label = format!("{}  ({})", buildable.name, cost_summary(*buildable));
            if ui.button(label).clicked() {
                try_build_ship(world, game_state, shipyard_id, *buildable);
            }
        }
        if ui.button("close").clicked() {
            *game_state = GameState::Playing;
        }
    });
}

/// a "80 metals, 30 crystals" summary of a ship's build cost.
fn cost_summary(buildable: ShipBuildable) -> String {
    buildable
        .costs
        .iter()
        .map(|cost| {
            let label = storable_display(cost.resource).0;
            format!("{:.0} {label}", cost.quantity)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// issue a ship build if the shipyard body can afford it, otherwise route to
/// the shipyard error state naming the missing resource.
fn try_build_ship(
    world: &mut World,
    game_state: &mut GameState,
    shipyard_id: EntityId,
    buildable: ShipBuildable,
) {
    let shortfall = {
        match world.celestial_data.get(&shipyard_id) {
            Some(body) => buildable
                .first_shortfall(body.stocks_at(crate::world::types::ConstructionLayer::Surface)),
            None => buildable.costs.first().map(|cost| ShipBuildShortfall {
                resource: cost.resource,
                required: cost.quantity,
                available: 0.0,
            }),
        }
    };
    match shortfall {
        Some(shortfall) => {
            let (label, _) = storable_display(shortfall.resource);
            *game_state = GameState::ShipyardMenuError {
                message: format!(
                    "not enough {label} (need {:.0}, have {:.0})",
                    shortfall.required, shortfall.available
                ),
            };
        }
        None => {
            world.add_command(Command::BuildShip {
                shipyard_entity_id: shipyard_id,
                ship_type: buildable.ship_type,
                civilian_credit_cost: None,
            });
            *game_state = GameState::Playing;
        }
    }
}

pub fn mining_route_menu(
    ctx: &egui::Context,
    world: &mut World,
    game_state: &mut GameState,
    ship_id: EntityId,
    mode: &MiningRouteMenuMode,
) {
    let ship_name = world.get_entity_name(ship_id).unwrap_or_default();
    centered_window(ctx, "mining route", |ui| {
        ui.label(format!("ship: {ship_name}"));
        match mode {
            MiningRouteMenuMode::SelectTarget => {
                ui.label("select target body:");
                if ui.button("auto (best route)").clicked() {
                    let route = world.compute_best_mining_route();
                    world.set_mining_route(ship_id, route);
                    *game_state = GameState::Playing;
                }
                for body in list_bodies(world) {
                    let name = world.get_entity_name(body).unwrap_or_default();
                    if ui.button(name).clicked() {
                        *game_state = GameState::MiningRouteMenu {
                            ship_id,
                            mode: MiningRouteMenuMode::SelectResource { target_id: body },
                        };
                    }
                }
                if ui.button("close").clicked() {
                    *game_state = GameState::Playing;
                }
            }
            MiningRouteMenuMode::SelectResource { target_id } => {
                let target_name = world.get_entity_name(*target_id).unwrap_or_default();
                ui.label(format!("target: {target_name}"));
                ui.label("select resource:");
                let mut resources: Vec<RawResource> = world
                    .celestial_data
                    .get(target_id)
                    .map(|d| d.yields.keys().copied().collect())
                    .unwrap_or_default();
                resources.sort();
                for resource in resources {
                    let (label, color) = raw_resource_display(resource);
                    if ui
                        .add(egui::Button::new(egui::RichText::new(label).color(color)))
                        .clicked()
                    {
                        *game_state = GameState::MiningRouteMenu {
                            ship_id,
                            mode: MiningRouteMenuMode::SelectSell {
                                target_id: *target_id,
                                resource,
                            },
                        };
                    }
                }
                if ui.button("back").clicked() {
                    *game_state = GameState::MiningRouteMenu {
                        ship_id,
                        mode: MiningRouteMenuMode::SelectTarget,
                    };
                }
            }
            MiningRouteMenuMode::SelectSell {
                target_id,
                resource,
            } => {
                let target_name = world.get_entity_name(*target_id).unwrap_or_default();
                ui.label(format!("target: {target_name}"));
                let (resource_label, _) = raw_resource_display(*resource);
                ui.label(format!("resource: {resource_label}"));
                ui.label("select sell body:");
                for body in list_bodies(world) {
                    let name = world.get_entity_name(body).unwrap_or_default();
                    if ui.button(name).clicked() {
                        world.set_mining_route(
                            ship_id,
                            Some(MiningRoute {
                                target_body: *target_id,
                                resource: *resource,
                                sell_body: body,
                            }),
                        );
                        *game_state = GameState::Playing;
                    }
                }
                if ui.button("back").clicked() {
                    *game_state = GameState::MiningRouteMenu {
                        ship_id,
                        mode: MiningRouteMenuMode::SelectResource {
                            target_id: *target_id,
                        },
                    };
                }
            }
        }
    });
}

/// all celestial bodies, sorted by id, for menu lists.
fn list_bodies(world: &World) -> Vec<EntityId> {
    let mut bodies: Vec<EntityId> = world.celestial_data.keys().copied().collect();
    bodies.sort_unstable();
    bodies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn procurement_resources_follow_v1_display_order() {
        assert_eq!(
            PROCUREMENT_RESOURCES,
            &[
                Storable::Raw(RawResource::Metals),
                Storable::Raw(RawResource::Organics),
                Storable::Raw(RawResource::Crystals),
                Storable::Raw(RawResource::Volatiles),
                Storable::Good(Good::FuelCells),
                Storable::Good(Good::ConstructionMaterials),
                Storable::Good(Good::Food),
            ]
        );
    }
}
