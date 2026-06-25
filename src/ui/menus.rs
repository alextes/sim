//! the stateful modal menus: build, shipyard, mining route. button presses
//! drive the `GameState` machine and issue world commands, replacing the old
//! sdl key handlers.

use strum::IntoEnumIterator;

use crate::app::{BuildMenuMode, GameState, MiningRouteMenuMode};
use crate::buildings::EntityBuildings;
use crate::command::Command;
use crate::control_state::ControlState;
use crate::palette;
use crate::ships::{buildable_ships, ShipBuildShortfall, ShipBuildable};
use crate::world::components::MiningRoute;
use crate::world::types::{BuildingType, RawResource};
use crate::world::{EntityId, World};

use super::{centered_window, raw_resource_display, storable_display};

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
    let name = world.get_entity_name(entity_id).unwrap_or_default();

    centered_window(ctx, "build menu", |ui| {
        ui.heading(name.as_str());
        match mode {
            BuildMenuMode::Main => build_main(ui, world, game_state, entity_id),
            BuildMenuMode::SelectBuilding => build_select(ui, game_state),
            BuildMenuMode::EnterQuantity {
                building,
                quantity_string,
            } => build_quantity(ui, game_state, *building, quantity_string),
            BuildMenuMode::ConfirmQuote { building, amount } => {
                build_confirm(ui, world, game_state, entity_id, *building, *amount)
            }
        }
    });
}

fn build_main(ui: &mut egui::Ui, world: &World, game_state: &mut GameState, entity_id: EntityId) {
    ui.separator();
    ui.label("construction queue:");
    if let Some(buildings) = world.buildings.get(&entity_id) {
        if buildings.build_queue.is_empty() {
            ui.colored_label(palette::DGRAY, "  (empty)");
        } else {
            for (building, count) in &buildings.build_queue {
                ui.label(format!(
                    "  - {} x{count}",
                    EntityBuildings::building_name(*building)
                ));
            }
        }
    }
    ui.separator();
    if ui.button("add to queue").clicked() {
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::SelectBuilding,
        };
    }
    if ui.button("close").clicked() {
        *game_state = GameState::Playing;
    }
}

fn build_select(ui: &mut egui::Ui, game_state: &mut GameState) {
    ui.label("select building:");
    for building in BuildingType::iter() {
        if ui
            .button(EntityBuildings::building_name(building))
            .clicked()
        {
            *game_state = GameState::BuildMenu {
                mode: BuildMenuMode::EnterQuantity {
                    building,
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
    game_state: &mut GameState,
    building: BuildingType,
    quantity_string: &str,
) {
    ui.label(format!(
        "building: {}",
        EntityBuildings::building_name(building)
    ));
    let mut qty = quantity_string.to_string();
    let response = ui.add(egui::TextEdit::singleline(&mut qty).hint_text("quantity"));
    let amount = qty.trim().parse::<u32>().ok().filter(|n| *n > 0);
    let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let confirm = ui
        .add_enabled(amount.is_some(), egui::Button::new("confirm"))
        .clicked();
    let cancel = ui.button("cancel").clicked();

    if cancel {
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::Main,
        };
    } else if (enter || confirm) && amount.is_some() {
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::ConfirmQuote {
                building,
                amount: amount.unwrap(),
            },
        };
    } else if qty != quantity_string {
        // persist the edited text back into the state machine for next frame.
        *game_state = GameState::BuildMenu {
            mode: BuildMenuMode::EnterQuantity {
                building,
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
    building: BuildingType,
    amount: u32,
) {
    ui.label(format!(
        "build {amount}x {}?",
        EntityBuildings::building_name(building)
    ));
    ui.label("cost:");
    let costs = EntityBuildings::get_build_costs(building, amount);
    let mut items: Vec<_> = costs.into_iter().collect();
    items.sort_by_key(|(storable, _)| format!("{storable}"));
    for (storable, cost) in &items {
        let have = world
            .celestial_data
            .get(&entity_id)
            .and_then(|d| d.stocks.get(storable))
            .copied()
            .unwrap_or(0.0);
        let color = if have < *cost {
            palette::RED
        } else {
            palette::WHITE
        };
        ui.colored_label(color, format!("  {cost:.1} {storable} (have {have:.1})"));
    }
    ui.horizontal(|ui| {
        if ui.button("yes").clicked() {
            world.add_command(Command::Build {
                entity_id,
                building_type: building,
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
            Some(body) => buildable.first_shortfall(&body.stocks),
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
