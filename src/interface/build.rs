use crate::{
    buildings::EntityBuildings,
    colors,
    render::SpriteSheetRenderer,
    world::{types::BuildingType, EntityId, World},
    BuildMenuMode,
};
use sdl2::{pixels::Color, render::Canvas, video::Window};
use strum::IntoEnumIterator;

use super::{draw_centered_window, PANEL_TEXT_COLOR};

pub fn render_build_menu(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    selected_id: Option<EntityId>,
    mode: &BuildMenuMode,
) {
    let mut lines: Vec<(String, Color)> = Vec::new();
    let entity_id = match selected_id {
        Some(id) => id,
        None => {
            lines.push(("no entity selected".to_string(), colors::LRED));
            draw_centered_window(canvas, renderer, &lines);
            return;
        }
    };

    let entity_name = world
        .get_entity_name(entity_id)
        .unwrap_or_else(|| "unknown".to_string());
    let entity_type = world.get_entity_type(entity_id);

    lines.push((format!("{entity_name} build menu"), PANEL_TEXT_COLOR));

    if let Some(et) = entity_type {
        let type_str = match et {
            crate::world::types::EntityType::Planet => "planet",
            crate::world::types::EntityType::Moon => "moon",
            crate::world::types::EntityType::GasGiant => "gas giant",
            _ => "n/a",
        };
        lines.push((format!("type: {type_str}"), colors::LGRAY));
    }

    lines.push(("".to_string(), colors::BLACK));

    match mode {
        BuildMenuMode::Main => {
            lines.push(("construction queue:".to_string(), PANEL_TEXT_COLOR));

            if let Some(buildings) = world.buildings.get(&entity_id) {
                if buildings.build_queue.is_empty() {
                    lines.push(("(empty)".to_string(), colors::DGRAY));
                } else {
                    for (building_type, count) in &buildings.build_queue {
                        let name = EntityBuildings::building_name(*building_type);
                        lines.push((format!("- {name} x{count}"), colors::WHITE));
                    }
                }
            }

            lines.push(("".to_string(), colors::BLACK));
            lines.push(("(a)dd to queue".to_string(), colors::WHITE));
            lines.push(("(q)uit".to_string(), colors::WHITE));
        }
        BuildMenuMode::SelectBuilding => {
            lines.push(("select building type:".to_string(), PANEL_TEXT_COLOR));
            for (i, building_type) in BuildingType::iter().enumerate() {
                let name = EntityBuildings::building_name(building_type);
                lines.push((format!("({}) {}", i + 1, name), colors::WHITE));
            }
        }
        BuildMenuMode::EnterQuantity {
            building,
            quantity_string,
        } => {
            let name = EntityBuildings::building_name(*building);
            lines.push((format!("building: {name}"), colors::LGRAY));
            lines.push(("".to_string(), colors::BLACK));
            lines.push(("enter quantity:".to_string(), PANEL_TEXT_COLOR));
            lines.push((quantity_string.clone(), colors::WHITE));
            lines.push(("".to_string(), colors::BLACK));
            lines.push(("(enter) confirm".to_string(), colors::WHITE));
            lines.push(("(esc) cancel".to_string(), colors::WHITE));
        }
        BuildMenuMode::ConfirmQuote { building, amount } => {
            let name = EntityBuildings::building_name(*building);
            lines.push((format!("build {amount}x {name}?"), PANEL_TEXT_COLOR));
            lines.push(("".to_string(), colors::BLACK));
            lines.push(("cost:".to_string(), PANEL_TEXT_COLOR));

            let costs = EntityBuildings::get_build_costs(*building, *amount);
            if let Some(celestial_data) = world.celestial_data.get(&entity_id) {
                let mut sorted_costs: Vec<_> = costs.into_iter().collect();
                sorted_costs.sort_by_key(|(resource, _)| *resource);

                for (resource, cost) in sorted_costs {
                    let stock = celestial_data.stocks.get(&resource).copied().unwrap_or(0.0);
                    let color = if stock < cost {
                        colors::LRED
                    } else {
                        colors::WHITE
                    };
                    lines.push((format!("- {cost:.1} {resource:?} (have {stock:.1})"), color));
                }
            }

            lines.push(("".to_string(), colors::BLACK));
            lines.push(("(y)es".to_string(), colors::WHITE));
            lines.push(("(n)o".to_string(), colors::WHITE));
        }
    }

    draw_centered_window(canvas, renderer, &lines);
}
