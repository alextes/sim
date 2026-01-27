use crate::colors;
use crate::render::SpriteSheetRenderer;
use crate::world::types::RawResource;
use crate::world::{EntityId, World};
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn render_mining_route_menu(
    canvas: &mut Canvas<Window>,
    renderer: &SpriteSheetRenderer,
    world: &World,
    ship_id: EntityId,
    mode: &crate::MiningRouteMenuMode,
) {
    let mut lines: Vec<(String, sdl2::pixels::Color)> = Vec::new();
    lines.push(("mining route".to_string(), colors::WHITE));
    lines.push((
        format!(
            "ship: {}",
            world.get_entity_name(ship_id).unwrap_or_default()
        ),
        colors::LGRAY,
    ));
    lines.push(("".to_string(), colors::BLACK));

    match mode {
        crate::MiningRouteMenuMode::SelectTarget => {
            lines.push((
                "select target body (1-9), or (a)uto".to_string(),
                colors::WHITE,
            ));
            for (i, id) in list_bodies(world).into_iter().enumerate().take(9) {
                let name = world.get_entity_name(id).unwrap_or_else(|| id.to_string());
                lines.push((format!("({}) {}", i + 1, name), colors::WHITE));
            }
        }
        crate::MiningRouteMenuMode::SelectResource { target_id } => {
            let target_name = world.get_entity_name(*target_id).unwrap_or_default();
            lines.push((format!("target: {}", target_name), colors::LGRAY));
            lines.push(("".to_string(), colors::BLACK));
            lines.push(("select resource".to_string(), colors::WHITE));
            let mut idx = 1;
            if let Some(cd) = world.celestial_data.get(target_id) {
                for (&raw, _) in cd.yields.iter() {
                    let (label, color) = resource_label(raw);
                    lines.push((format!("({}) {}", idx, label), color));
                    idx += 1;
                }
            }
        }
        crate::MiningRouteMenuMode::SelectSell {
            target_id,
            resource,
        } => {
            let target_name = world.get_entity_name(*target_id).unwrap_or_default();
            let (res_name, res_color) = resource_label(*resource);
            lines.push((format!("target: {}", target_name), colors::LGRAY));
            lines.push((format!("resource: {}", res_name), res_color));
            lines.push(("".to_string(), colors::BLACK));
            lines.push(("select sell body".to_string(), colors::WHITE));
            for (i, id) in list_bodies(world).into_iter().enumerate().take(9) {
                let name = world.get_entity_name(id).unwrap_or_else(|| id.to_string());
                lines.push((format!("({}) {}", i + 1, name), colors::WHITE));
            }
        }
    }

    super::draw_centered_window(canvas, renderer, &lines);
}

fn list_bodies(world: &World) -> Vec<EntityId> {
    let mut v: Vec<EntityId> = world
        .iter_entities()
        .filter(|id| world.celestial_data.contains_key(id))
        .collect();
    v.sort();
    v
}

fn resource_label(r: RawResource) -> (&'static str, sdl2::pixels::Color) {
    match r {
        RawResource::Metals => ("metals", colors::LGRAY),
        RawResource::Organics => ("organics", colors::LGREEN),
        RawResource::Crystals => ("crystals", colors::CYAN),
        RawResource::Isotopes => ("isotopes", colors::MAGENTA),
        RawResource::Microbes => ("microbes", colors::YELLOW),
        RawResource::Volatiles => ("volatiles", colors::ORANGE),
        RawResource::RareExotics => ("exotics", colors::LRED),
        RawResource::DarkMatter => ("dark matter", colors::DGRAY),
        RawResource::NobleGases => ("noble gases", colors::LBLUE),
    }
}
