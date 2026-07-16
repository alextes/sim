//! winit input handling and screen->world entity picking.
//!
//! replaces the old sdl `event_handling` + `input` modules. stage 2 wires the
//! camera (pan/zoom) and single-click selection so the rendered world is
//! navigable. the menu-opening keys (b/s/r), box-select, and overlays return in
//! stage 3 alongside the egui ui.

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::app::{BuildMenuMode, GameState, MiningRouteMenuMode};
use crate::command::Command;
use crate::control_state::ControlState;
use crate::location::PointF64;
use crate::ships::ShipType;
use crate::viewport::Viewport;
use crate::world::types::{EntityType, InfrastructureType};
use crate::world::{EntityId, World};

/// world distance panned per arrow-key press at zoom 1.0.
const KEY_PAN_AT_ZOOM_1: f64 = 0.25;
/// mouse-wheel zoom step.
const WHEEL_ZOOM_FACTOR: f64 = 1.2;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InputOutcome {
    pub request_redraw: bool,
}

impl InputOutcome {
    fn redraw() -> Self {
        Self {
            request_redraw: true,
        }
    }
}

/// handle one winit window event that egui did not consume.
pub fn handle_window_event(
    event: &WindowEvent,
    viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
    game_state: &mut GameState,
) -> InputOutcome {
    // cursor position and modifier state are tracked regardless of game state.
    match event {
        WindowEvent::ModifiersChanged(modifiers) => {
            let state = modifiers.state();
            controls.ctrl_down = state.control_key() || state.super_key();
            controls.shift_down = state.shift_key();
            return InputOutcome::default();
        }
        WindowEvent::CursorMoved { position, .. } => {
            return handle_cursor_moved(*position, viewport, controls);
        }
        _ => {}
    }

    // escape transitions work in every state (menus, pause, quit).
    if let WindowEvent::KeyboardInput { event: key, .. } = event {
        if key.state == ElementState::Pressed
            && !key.repeat
            && matches!(key.physical_key, PhysicalKey::Code(KeyCode::Escape))
        {
            handle_escape(game_state, controls);
            return InputOutcome::default();
        }
    }

    // the rest are gameplay bindings, only active while playing.
    if *game_state != GameState::Playing {
        return InputOutcome::default();
    }

    match event {
        WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
            handle_keydown(event, viewport, world, controls, game_state)
        }
        WindowEvent::MouseInput { state, button, .. } => {
            handle_mouse_button(*state, *button, viewport, world, controls);
            InputOutcome::default()
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let scroll = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y as f64,
                MouseScrollDelta::PixelDelta(p) => p.y,
            };
            if let Some(pos) = controls.last_mouse_pos {
                if scroll > 0.0 {
                    viewport.zoom_at(WHEEL_ZOOM_FACTOR, pos);
                    return InputOutcome::redraw();
                } else if scroll < 0.0 {
                    viewport.zoom_at(1.0 / WHEEL_ZOOM_FACTOR, pos);
                    return InputOutcome::redraw();
                }
            }
            InputOutcome::default()
        }
        _ => InputOutcome::default(),
    }
}

fn handle_cursor_moved(
    position: PhysicalPosition<f64>,
    viewport: &mut Viewport,
    controls: &mut ControlState,
) -> InputOutcome {
    let new = (position.x, position.y);
    let prev = controls.last_mouse_pos;
    controls.last_mouse_pos = Some(new);

    // middle-drag or ctrl+left-drag pans the camera.
    if controls.middle_mouse_dragging || controls.ctrl_left_mouse_dragging {
        if let Some((px, py)) = prev {
            let scale = viewport.world_tile_pixel_size_on_screen();
            viewport.anchor.x -= (new.0 - px) / scale;
            viewport.anchor.y -= (new.1 - py) / scale;
            return InputOutcome::redraw();
        }
    }
    InputOutcome::default()
}

fn handle_keydown(
    event: &KeyEvent,
    viewport: &mut Viewport,
    world: &World,
    controls: &mut ControlState,
    game_state: &mut GameState,
) -> InputOutcome {
    let PhysicalKey::Code(code) = event.physical_key else {
        return InputOutcome::default();
    };
    let pan = KEY_PAN_AT_ZOOM_1 / viewport.zoom.max(0.01);

    match code {
        KeyCode::ArrowUp => {
            viewport.anchor.y -= pan;
            InputOutcome::redraw()
        }
        KeyCode::ArrowDown => {
            viewport.anchor.y += pan;
            InputOutcome::redraw()
        }
        KeyCode::ArrowLeft => {
            viewport.anchor.x -= pan;
            InputOutcome::redraw()
        }
        KeyCode::ArrowRight => {
            viewport.anchor.x += pan;
            InputOutcome::redraw()
        }
        KeyCode::Equal | KeyCode::NumpadAdd => {
            viewport.zoom_in();
            InputOutcome::redraw()
        }
        KeyCode::Minus | KeyCode::NumpadSubtract => {
            viewport.zoom_out();
            InputOutcome::redraw()
        }
        KeyCode::Tab if !event.repeat => {
            cycle_entity_focus(world, controls, controls.shift_down);
            InputOutcome::default()
        }
        KeyCode::F4 if !event.repeat => {
            controls.debug_enabled = !controls.debug_enabled;
            InputOutcome::default()
        }
        KeyCode::KeyF if !event.repeat => {
            if !controls.selection.is_empty() {
                controls.track_mode = !controls.track_mode;
            }
            InputOutcome::default()
        }
        KeyCode::KeyB if !event.repeat => {
            open_build_menu(world, controls, game_state);
            InputOutcome::default()
        }
        KeyCode::KeyS if !event.repeat => {
            open_shipyard_menu(world, controls, game_state);
            InputOutcome::default()
        }
        KeyCode::KeyR if !event.repeat => {
            open_mining_menu(world, controls, game_state);
            InputOutcome::default()
        }
        KeyCode::KeyO if !event.repeat => {
            open_planet_overview(world, controls, game_state);
            InputOutcome::default()
        }
        KeyCode::Space if !event.repeat => {
            controls.paused = !controls.paused;
            InputOutcome::default()
        }
        KeyCode::Backquote if !event.repeat => {
            controls.sim_speed = match controls.sim_speed {
                1 => 2,
                2 => 3,
                _ => 1,
            };
            InputOutcome::default()
        }
        _ => InputOutcome::default(),
    }
}

/// escape: context-dependent menu/pause transition (ported from the old global
/// escape handler).
fn handle_escape(game_state: &mut GameState, controls: &mut ControlState) {
    match game_state {
        GameState::MainMenu => controls.quit_requested = true,
        GameState::Playing => {
            *game_state = GameState::GameMenu;
            controls.paused = true;
        }
        GameState::GameMenu => {
            *game_state = GameState::Playing;
            controls.paused = false;
        }
        GameState::BuildMenu { .. }
        | GameState::ShipyardMenu
        | GameState::ShipyardMenuError { .. }
        | GameState::PlanetOverview { .. }
        | GameState::MiningRouteMenu { .. } => *game_state = GameState::Playing,
    }
}

/// (o) open the owned-body planet overview.
fn open_planet_overview(world: &World, controls: &ControlState, game_state: &mut GameState) {
    let bodies = world.owned_body_overview_entities();
    let selected = controls
        .selection
        .first()
        .copied()
        .filter(|entity| bodies.contains(entity))
        .or_else(|| bodies.first().copied());

    *game_state = GameState::PlanetOverview { selected };
}

/// (b) open the build menu if the selection is a player-controlled body.
fn open_build_menu(world: &World, controls: &ControlState, game_state: &mut GameState) {
    if let Some(&id) = controls.selection.first() {
        if world.is_player_controlled(id)
            && world.get_entity_type(id) == Some(EntityType::Planet)
            && world.infrastructure.contains_key(&id)
        {
            *game_state = GameState::BuildMenu {
                mode: BuildMenuMode::Main,
            };
        }
    }
}

/// (s) open the shipyard menu if a single player-controlled body has a shipyard.
fn open_shipyard_menu(world: &World, controls: &ControlState, game_state: &mut GameState) {
    if controls.selection.len() != 1 {
        return;
    }
    let id = controls.selection[0];
    if world.is_player_controlled(id) {
        if let Some(infrastructure) = world.infrastructure.get(&id) {
            if infrastructure.get_count(InfrastructureType::Shipyard) > 0 {
                *game_state = GameState::ShipyardMenu;
            }
        }
    }
}

/// (r) open the mining-route menu if a single mining ship is selected.
fn open_mining_menu(world: &World, controls: &ControlState, game_state: &mut GameState) {
    if controls.selection.len() != 1 {
        return;
    }
    let id = controls.selection[0];
    if let Some(info) = world.ships.get(&id) {
        if info.ship_type == ShipType::MiningShip {
            *game_state = GameState::MiningRouteMenu {
                ship_id: id,
                mode: MiningRouteMenuMode::SelectTarget,
            };
        }
    }
}

fn handle_mouse_button(
    state: ElementState,
    button: MouseButton,
    viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
) {
    let Some((x, y)) = controls.last_mouse_pos else {
        return;
    };

    match (state, button) {
        (ElementState::Pressed, MouseButton::Left) => {
            if controls.ctrl_down {
                controls.ctrl_left_mouse_dragging = true;
            } else {
                match get_entity_id_at_screen_coords(x, y, viewport, world) {
                    Some(id) => controls.selection = vec![id],
                    None => {
                        // empty space: clear selection and begin a drag box.
                        controls.selection.clear();
                        controls.track_mode = false;
                        controls.selection_box_start = Some((x, y));
                    }
                }
            }
        }
        (ElementState::Released, MouseButton::Left) => {
            controls.ctrl_left_mouse_dragging = false;
            if let Some(start) = controls.selection_box_start.take() {
                let rect = ScreenRect::from_corners(start, (x, y));
                // ignore tiny drags (those were really just empty-space clicks).
                if rect.w > 2.0 && rect.h > 2.0 {
                    let entities = get_entities_in_screen_rect(rect, viewport, world);
                    apply_box_selection(controls, world, &entities);
                }
            }
        }
        (ElementState::Pressed, MouseButton::Middle) => controls.middle_mouse_dragging = true,
        (ElementState::Released, MouseButton::Middle) => controls.middle_mouse_dragging = false,
        (ElementState::Pressed, MouseButton::Right) => {
            // move-order the selected ships to the clicked cell center.
            let precise = viewport.screen_to_world_coords(x, y);
            let destination = PointF64 {
                x: precise.x.floor() + 0.5,
                y: precise.y.floor() + 0.5,
            };
            let ships: Vec<EntityId> = controls
                .selection
                .iter()
                .copied()
                .filter(|id| world.ships.contains_key(id))
                .collect();
            for ship_id in ships {
                world.add_command(Command::MoveShip {
                    ship_id,
                    destination,
                });
            }
        }
        _ => {}
    }
}

/// cycle the single selection forward (or backward with shift) through entities.
fn cycle_entity_focus(world: &World, controls: &mut ControlState, reverse: bool) {
    if world.entities.is_empty() {
        controls.selection.clear();
        return;
    }
    let count = world.entities.len();
    let current = controls
        .selection
        .first()
        .and_then(|id| world.entities.iter().position(|e| e == id));

    let next = if reverse {
        match current {
            Some(0) => count - 1,
            Some(i) => i - 1,
            None => count - 1,
        }
    } else {
        match current {
            Some(i) => (i + 1) % count,
            None => 0,
        }
    };
    controls.selection = vec![world.entities[next]];
}

/// resolve a box selection: prefer ships (select all), otherwise a single body
/// by type priority star > gas giant > planet > moon (ported from the old sdl
/// logic).
fn apply_box_selection(controls: &mut ControlState, world: &World, entities: &[EntityId]) {
    let of_type = |ty: EntityType| -> Vec<EntityId> {
        entities
            .iter()
            .copied()
            .filter(|id| world.get_entity_type(*id) == Some(ty))
            .collect()
    };

    let ships = of_type(EntityType::Ship);
    if !ships.is_empty() {
        controls.selection = ships;
        return;
    }
    for ty in [
        EntityType::Star,
        EntityType::GasGiant,
        EntityType::Planet,
        EntityType::Moon,
    ] {
        let bodies = of_type(ty);
        if bodies.len() == 1 {
            controls.selection = bodies;
            return;
        }
    }
}

/// a screen-space rectangle in physical pixels.
#[derive(Debug, Clone, Copy)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl ScreenRect {
    /// build the axis-aligned rect spanning two corner points.
    pub fn from_corners(a: (f64, f64), b: (f64, f64)) -> Self {
        Self {
            x: a.0.min(b.0),
            y: a.1.min(b.1),
            w: (a.0 - b.0).abs(),
            h: (a.1 - b.1).abs(),
        }
    }

    /// true if the two rects overlap (touching edges do not count).
    pub fn intersects(&self, other: &ScreenRect) -> bool {
        let ax2 = self.x + self.w;
        let ay2 = self.y + self.h;
        let bx2 = other.x + other.w;
        let by2 = other.y + other.h;
        self.x < bx2 && ax2 > other.x && self.y < by2 && ay2 > other.y
    }
}

/// the entity id at the given screen coordinates, if any.
pub fn get_entity_id_at_screen_coords(
    screen_x: f64,
    screen_y: f64,
    viewport: &Viewport,
    world: &World,
) -> Option<EntityId> {
    let clicked_world_coords: PointF64 = viewport.screen_to_world_coords(screen_x, screen_y);

    let clicked_world_tile_x_i32 = clicked_world_coords.x.floor() as i32;
    let clicked_world_tile_y_i32 = clicked_world_coords.y.floor() as i32;

    world.iter_entities().find_map(|entity_id| {
        world.get_location(entity_id).and_then(|loc| {
            if loc.x == clicked_world_tile_x_i32 && loc.y == clicked_world_tile_y_i32 {
                Some(entity_id)
            } else {
                None
            }
        })
    })
}

/// all entities whose on-screen tile overlaps the given screen rectangle.
pub fn get_entities_in_screen_rect(
    rect: ScreenRect,
    viewport: &Viewport,
    world: &World,
) -> Vec<EntityId> {
    let mut entities = Vec::new();

    let world_tile_size_on_screen = viewport.world_tile_pixel_size_on_screen();

    for entity_id in world.iter_entities() {
        if let Some(pos) = world.get_location_f64(entity_id) {
            let screen_coords = viewport.world_to_screen_px(pos.x, pos.y);
            let size = (world.get_render_size(entity_id) * world_tile_size_on_screen).max(2.0);

            let entity_rect = ScreenRect {
                x: screen_coords.0 - size / 2.0,
                y: screen_coords.1 - size / 2.0,
                w: size,
                h: size,
            };

            if rect.intersects(&entity_rect) {
                entities.push(entity_id);
            }
        }
    }
    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;
    use crate::viewport::Viewport;
    use crate::world::World;

    #[test]
    fn test_get_entity_at_screen_coords() {
        let mut world = World::default();
        let entity_pos = Point { x: 10, y: 20 };
        let entity_id = world.spawn_star("test_star".to_string(), entity_pos);

        let viewport = Viewport {
            anchor: PointF64 {
                x: entity_pos.x as f64,
                y: entity_pos.y as f64,
            },
            zoom: 1.0,
            screen_pixel_width: 800,
            screen_pixel_height: 600,
        };

        // click on the center of the screen, which should be where the entity is
        let result = get_entity_id_at_screen_coords(400.0, 300.0, &viewport, &world);
        assert_eq!(result, Some(entity_id));

        // click somewhere else
        let result_none = get_entity_id_at_screen_coords(0.0, 0.0, &viewport, &world);
        assert_eq!(result_none, None);
    }

    #[test]
    fn test_screen_rect_intersects() {
        let a = ScreenRect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        };
        let overlapping = ScreenRect {
            x: 5.0,
            y: 5.0,
            w: 10.0,
            h: 10.0,
        };
        let disjoint = ScreenRect {
            x: 20.0,
            y: 20.0,
            w: 5.0,
            h: 5.0,
        };
        assert!(a.intersects(&overlapping));
        assert!(!a.intersects(&disjoint));
    }

    #[test]
    fn test_subpixel_drag_pans_camera() {
        let mut viewport = Viewport::default();
        let mut controls = ControlState::new(vec![]);
        controls.middle_mouse_dragging = true;
        controls.last_mouse_pos = Some((10.25, 20.25));

        let outcome = handle_cursor_moved(
            PhysicalPosition::new(10.75, 20.75),
            &mut viewport,
            &mut controls,
        );

        assert!(outcome.request_redraw);
        assert!((viewport.anchor.x - -(0.5 / 9.0)).abs() < 1e-9);
        assert!((viewport.anchor.y - -(0.5 / 9.0)).abs() < 1e-9);
        assert_eq!(controls.last_mouse_pos, Some((10.75, 20.75)));
    }

    #[test]
    fn open_planet_overview_uses_selected_owned_body() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let earth_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 1.0);
        let mars_id = world.spawn_planet("mars".to_string(), star_id, 12.0, 0.0, 1.0);
        world.set_player_controlled(earth_id);
        world.set_player_controlled(mars_id);
        let controls = ControlState::new(vec![mars_id]);
        let mut game_state = GameState::Playing;

        open_planet_overview(&world, &controls, &mut game_state);

        assert_eq!(
            game_state,
            GameState::PlanetOverview {
                selected: Some(mars_id)
            }
        );
    }

    #[test]
    fn open_planet_overview_falls_back_to_first_owned_body() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let earth_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 1.0);
        let ship_id = world.spawn_frigate("frigate".to_string(), Point { x: 1, y: 1 });
        world.set_player_controlled(earth_id);
        let controls = ControlState::new(vec![ship_id]);
        let mut game_state = GameState::Playing;

        open_planet_overview(&world, &controls, &mut game_state);

        assert_eq!(
            game_state,
            GameState::PlanetOverview {
                selected: Some(earth_id)
            }
        );
    }

    #[test]
    fn escape_closes_planet_overview() {
        let mut controls = ControlState::new(vec![]);
        let mut game_state = GameState::PlanetOverview { selected: Some(42) };

        handle_escape(&mut game_state, &mut controls);

        assert_eq!(game_state, GameState::Playing);
    }
}
