//! the stateful modal menus: build, shipyard, mining route. button presses
//! drive the `GameState` machine and issue world commands, replacing the old
//! sdl key handlers.

use strum::IntoEnumIterator;

use crate::app::{BuildMenuMode, GameState, MiningRouteMenuMode};
use crate::command::Command;
use crate::control_state::ControlState;
use crate::infrastructure::EntityInfrastructure;
use crate::palette;
use crate::ships::{buildable_ships, ShipBuildShortfall, ShipBuildable};
use crate::world::components::MiningRoute;
use crate::world::types::{EntityType, InfrastructureType, RawResource};
use crate::world::{EntityId, World};

use super::{centered_window, raw_resource_display, storable_display};

pub fn planet_overview(
    ctx: &egui::Context,
    world: &World,
    controls: &mut ControlState,
    game_state: &mut GameState,
    selected: Option<EntityId>,
) {
    let bodies = world.owned_body_overview_entities();
    let current = selected
        .filter(|entity| bodies.contains(entity))
        .or_else(|| bodies.first().copied());
    if current != selected {
        *game_state = GameState::PlanetOverview { selected: current };
    }

    egui::Window::new("planet overview")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(true)
        .default_width(720.0)
        .default_height(460.0)
        .show(ctx, |ui| {
            if bodies.is_empty() {
                ui.colored_label(palette::DGRAY, "no owned planets");
                if ui.button("close").clicked() {
                    *game_state = GameState::Playing;
                }
                return;
            }

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(220.0);
                    ui.label("owned bodies");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("planet_overview_body_list")
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for body in &bodies {
                                let name = world.get_entity_name(*body).unwrap_or_default();
                                let selected_row = current == Some(*body);
                                if ui.selectable_label(selected_row, name).clicked() {
                                    controls.selection = vec![*body];
                                    *game_state = GameState::PlanetOverview {
                                        selected: Some(*body),
                                    };
                                }
                            }
                        });
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.set_min_width(420.0);
                    if let Some(body) = current {
                        planet_detail(ui, world, body);
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("build").clicked() {
                                controls.selection = vec![body];
                                *game_state = GameState::BuildMenu {
                                    mode: BuildMenuMode::Main,
                                };
                            }
                            let has_shipyard =
                                world
                                    .infrastructure
                                    .get(&body)
                                    .is_some_and(|infrastructure| {
                                        infrastructure.get_count(InfrastructureType::Shipyard) > 0
                                    });
                            if ui
                                .add_enabled(has_shipyard, egui::Button::new("shipyard"))
                                .clicked()
                            {
                                controls.selection = vec![body];
                                *game_state = GameState::ShipyardMenu;
                            }
                            if ui.button("close").clicked() {
                                *game_state = GameState::Playing;
                            }
                        });
                    }
                });
            });
        });
}

fn planet_detail(ui: &mut egui::Ui, world: &World, body: EntityId) {
    let name = world.get_entity_name(body).unwrap_or_default();
    ui.heading(name);
    if let Some(entity_type) = world.get_entity_type(body) {
        ui.colored_label(palette::GRAY, body_type_label(entity_type));
    }

    egui::ScrollArea::vertical()
        .id_salt("planet_overview_detail")
        .max_height(350.0)
        .show(ui, |ui| {
            if let Some(data) = world.celestial_data.get(&body) {
                ui.label(format!("population: {:.2}m", data.population));
                ui.label(format!("civ credits: {:.0}", data.credits));
                if !data.yields.is_empty() {
                    ui.separator();
                    ui.label("yields");
                    let mut yields: Vec<_> = data.yields.iter().collect();
                    yields.sort_by_key(|(resource, _)| **resource);
                    for (resource, grade) in yields {
                        let (label, color) = raw_resource_display(*resource);
                        ui.colored_label(color, format!("{label}: {grade:.2}"));
                    }
                }
                if !data.stocks.is_empty() {
                    ui.separator();
                    ui.label("stocks");
                    let mut stocks: Vec<_> = data.stocks.iter().collect();
                    stocks.sort_by_key(|(storable, _)| **storable);
                    for (storable, amount) in stocks {
                        let (label, color) = storable_display(*storable);
                        ui.colored_label(color, format!("{label}: {amount:.1}"));
                    }
                }
            }

            if let Some(infrastructure) = world.infrastructure.get(&body) {
                ui.separator();
                ui.label("infrastructure");
                if infrastructure.infra.is_empty() {
                    ui.colored_label(palette::DGRAY, "(none)");
                } else {
                    let mut infra: Vec<_> = infrastructure.infra.iter().collect();
                    infra.sort_by_key(|(infrastructure, _)| format!("{infrastructure:?}"));
                    for (infrastructure, count) in infra {
                        let name = EntityInfrastructure::infrastructure_name(*infrastructure);
                        ui.colored_label(palette::GRAY, format!("{name}: {count}"));
                    }
                }

                ui.separator();
                ui.label("construction queue");
                if infrastructure.build_queue.is_empty() {
                    ui.colored_label(palette::DGRAY, "(empty)");
                } else {
                    for (infrastructure_type, count) in &infrastructure.build_queue {
                        ui.label(format!(
                            "{} x{count}",
                            EntityInfrastructure::infrastructure_name(*infrastructure_type)
                        ));
                    }
                }
            }
        });
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
    let name = world.get_entity_name(entity_id).unwrap_or_default();

    centered_window(ctx, "build menu", |ui| {
        ui.heading(name.as_str());
        match mode {
            BuildMenuMode::Main => build_main(ui, world, game_state, entity_id),
            BuildMenuMode::SelectInfrastructure => build_select(ui, game_state),
            BuildMenuMode::EnterQuantity {
                infrastructure,
                quantity_string,
            } => build_quantity(ui, game_state, *infrastructure, quantity_string),
            BuildMenuMode::ConfirmQuote {
                infrastructure,
                amount,
            } => build_confirm(ui, world, game_state, entity_id, *infrastructure, *amount),
        }
    });
}

fn build_main(ui: &mut egui::Ui, world: &World, game_state: &mut GameState, entity_id: EntityId) {
    ui.separator();
    ui.label("construction queue:");
    if let Some(infrastructure) = world.infrastructure.get(&entity_id) {
        if infrastructure.build_queue.is_empty() {
            ui.colored_label(palette::DGRAY, "  (empty)");
        } else {
            for (infrastructure_type, count) in &infrastructure.build_queue {
                ui.label(format!(
                    "  - {} x{count}",
                    EntityInfrastructure::infrastructure_name(*infrastructure_type)
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

fn build_select(ui: &mut egui::Ui, game_state: &mut GameState) {
    ui.label("select infrastructure:");
    for infrastructure in InfrastructureType::iter() {
        if ui
            .button(EntityInfrastructure::infrastructure_name(infrastructure))
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
    game_state: &mut GameState,
    infrastructure: InfrastructureType,
    quantity_string: &str,
) {
    ui.label(format!(
        "infrastructure: {}",
        EntityInfrastructure::infrastructure_name(infrastructure)
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
        EntityInfrastructure::infrastructure_name(infrastructure)
    ));
    ui.label("cost:");
    let costs = EntityInfrastructure::get_build_costs(infrastructure, amount);
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
