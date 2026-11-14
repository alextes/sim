//! headless scenario runner: builds the real seeded world, advances it by a
//! fixed number of simulation ticks, and dumps the resulting state as stable,
//! sorted JSON. no window, wgpu, or egui is touched.

use std::collections::HashMap;
use std::fmt::{Debug, Write};

use crate::app::SIMULATION_DT;
use crate::background::BackgroundLayer;
use crate::map_generation;
use crate::world::types::EntityType;
use crate::world::{EntityId, World};

/// build the initial world exactly as the windowed app does, including the
/// background starfield draw that consumes world rng, so headless runs and
/// screenshot captures with the same seed see the same simulation.
pub fn seeded_world(seed: Option<u64>) -> (World, BackgroundLayer) {
    let mut world = World::default();
    if let Some(seed) = seed {
        world.seed_rng(seed);
    }
    map_generation::populate_initial_galaxy(&mut world);
    let background = BackgroundLayer::new(&mut world.rng.0);
    (world, background)
}

/// advance the world by `ticks` fixed simulation steps. shared with the
/// screenshot capture path.
pub fn advance(world: &mut World, ticks: u64) {
    for tick in 1..=ticks {
        world.update(SIMULATION_DT.as_secs_f64(), tick);
    }
}

/// run a scenario and return the JSON dump of the final world state.
pub fn run(seed: u64, ticks: u64) -> String {
    let (mut world, _background) = seeded_world(Some(seed));
    advance(&mut world, ticks);
    dump(&world, seed, ticks)
}

/// render the world as pretty-printed JSON with every collection sorted, so
/// identical worlds always produce byte-identical output.
pub fn dump(world: &World, seed: u64, ticks: u64) -> String {
    let mut ids: Vec<EntityId> = world.entities.clone();
    ids.sort_unstable();

    let entities: Vec<Json> = ids.iter().map(|&id| entity_json(world, id)).collect();

    let root = Json::object([
        ("seed", Json::Int(seed as i64)),
        ("ticks", Json::Int(ticks as i64)),
        ("player_credits", Json::Num(world.player_credits)),
        ("enable_civilian_ai", Json::Bool(world.enable_civilian_ai)),
        ("entity_count", Json::Int(ids.len() as i64)),
        ("entities", Json::Array(entities)),
    ]);
    let mut out = String::new();
    root.write(&mut out, 0);
    out.push('\n');
    out
}

fn entity_json(world: &World, id: EntityId) -> Json {
    let mut fields: Vec<(&str, Json)> = vec![
        ("id", Json::Int(id as i64)),
        (
            "name",
            world
                .get_entity_name(id)
                .map(Json::Str)
                .unwrap_or(Json::Null),
        ),
        (
            "type",
            world
                .get_entity_type(id)
                .map(|t| debug_str(&t))
                .unwrap_or(Json::Null),
        ),
        (
            "player_controlled",
            Json::Bool(world.player_controlled.contains(&id)),
        ),
        (
            "position",
            world
                .get_location_f64(id)
                .map(|p| Json::object([("x", Json::Num(p.x)), ("y", Json::Num(p.y))]))
                .unwrap_or(Json::Null),
        ),
    ];

    if let Some(body) = world.celestial_data.get(&id) {
        fields.push(("credits", Json::Num(body.credits)));
        fields.push(("population", Json::Num(body.population as f64)));
        fields.push(("stocks", sorted_amounts(&body.stocks)));
        fields.push(("orbital_stocks", sorted_amounts(&body.orbital_stocks)));
    }

    if let Some(infra) = world.infrastructure.get(&id) {
        let mut units: Vec<(String, u32)> = infra
            .infra
            .iter()
            .map(|(kind, count)| (format!("{kind:?}"), *count))
            .collect();
        units.sort();
        let queue = infra
            .build_queue
            .iter()
            .map(|(kind, count)| {
                Json::object([
                    ("type", debug_str(kind)),
                    ("amount", Json::Int(*count as i64)),
                ])
            })
            .collect();
        fields.push((
            "infrastructure",
            Json::object([
                (
                    "units",
                    Json::Object(
                        units
                            .into_iter()
                            .map(|(k, v)| (k, Json::Int(v as i64)))
                            .collect(),
                    ),
                ),
                ("build_queue", Json::Array(queue)),
                (
                    "construction_progress",
                    Json::Num(infra.construction_progress as f64),
                ),
            ]),
        ));
    }

    if world.get_entity_type(id) == Some(EntityType::Ship) || world.ships.contains_key(&id) {
        fields.push(("ship", ship_json(world, id)));
    }

    Json::object(fields)
}

fn ship_json(world: &World, id: EntityId) -> Json {
    let mut fields: Vec<(&str, Json)> = Vec::new();
    if let Some(info) = world.ships.get(&id) {
        fields.push(("ship_type", debug_str(&info.ship_type)));
        fields.push(("speed", Json::Num(info.speed)));
    }
    if let Some(cargo) = world.cargo.get(&id) {
        fields.push((
            "cargo",
            Json::object([
                ("capacity", Json::Num(cargo.capacity as f64)),
                ("current_load", Json::Num(cargo.current_load as f64)),
                ("contents", sorted_amounts(&cargo.contents)),
            ]),
        ));
    }
    if let Some(ai) = world.civilian_ai.get(&id) {
        fields.push(("home_base", Json::Int(ai.home_base as i64)));
        // the state enum's debug form is deterministic and carries route,
        // cargo target, and mining progress.
        fields.push(("state", debug_str(&ai.state)));
    }
    if let Some(route) = world.mining_routes.get(&id) {
        fields.push(("mining_route", debug_str(route)));
    }
    if let Some(order) = world.move_orders.get(&id) {
        fields.push((
            "move_order",
            Json::object([("x", Json::Num(order.x)), ("y", Json::Num(order.y))]),
        ));
    }
    Json::object(fields)
}

fn sorted_amounts<K: Debug + Ord>(map: &HashMap<K, f32>) -> Json {
    let mut entries: Vec<(&K, &f32)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    Json::Object(
        entries
            .into_iter()
            .map(|(k, v)| (format!("{k:?}"), Json::Num(*v as f64)))
            .collect(),
    )
}

fn debug_str(value: &impl Debug) -> Json {
    Json::Str(format!("{value:?}"))
}

/// minimal JSON value with insertion-ordered objects; callers are responsible
/// for supplying keys in a deterministic order.
enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn object<'a>(fields: impl IntoIterator<Item = (&'a str, Json)>) -> Json {
        Json::Object(fields.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }

    fn write(&self, out: &mut String, indent: usize) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Int(i) => write!(out, "{i}").unwrap(),
            // rust's float display is the shortest round-trip form, so it is
            // stable across runs. non-finite values have no JSON form.
            Json::Num(n) if n.is_finite() => write!(out, "{n}").unwrap(),
            Json::Num(_) => out.push_str("null"),
            Json::Str(s) => write_str(out, s),
            Json::Array(items) if items.is_empty() => out.push_str("[]"),
            Json::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    newline(out, indent + 1);
                    item.write(out, indent + 1);
                }
                newline(out, indent);
                out.push(']');
            }
            Json::Object(fields) if fields.is_empty() => out.push_str("{}"),
            Json::Object(fields) => {
                out.push('{');
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    newline(out, indent + 1);
                    write_str(out, key);
                    out.push_str(": ");
                    value.write(out, indent + 1);
                }
                newline(out, indent);
                out.push('}');
            }
        }
    }
}

fn newline(out: &mut String, indent: usize) {
    out.push('\n');
    for _ in 0..indent {
        out.push_str("  ");
    }
}

fn write_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_ticks_produce_identical_dumps() {
        let first = run(42, 500);
        let second = run(42, 500);
        assert_eq!(first, second);
    }

    #[test]
    fn dump_reflects_simulation_progress() {
        let start = run(7, 0);
        let later = run(7, 500);
        assert!(start.contains("\"ticks\": 0"));
        assert!(later.contains("\"ticks\": 500"));
        assert_ne!(start, later);
    }

    #[test]
    fn dump_lists_entities_sorted_by_id() {
        let (world, _) = seeded_world(Some(3));
        let output = dump(&world, 3, 0);
        let ids: Vec<u32> = output
            .lines()
            .filter_map(|line| line.trim().strip_prefix("\"id\": "))
            .map(|id| id.trim_end_matches(',').parse().unwrap())
            .collect();
        assert!(!ids.is_empty());
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
        assert_eq!(ids.len(), world.entities.len());
    }

    #[test]
    fn escapes_json_strings() {
        let mut out = String::new();
        write_str(&mut out, "a\"b\\c\n");
        assert_eq!(out, "\"a\\\"b\\\\c\\n\"");
    }
}
