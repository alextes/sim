//! input/interaction state, decoupled from the windowing backend.
//!
//! moved out of the old sdl `event_handling` module. the winit input handler
//! (`crate::input`) mutates this; the sim loop and ui read it.

use crate::world::EntityId;

#[derive(Debug)]
pub struct ControlState {
    pub selection: Vec<EntityId>,
    pub debug_enabled: bool,
    pub track_mode: bool,
    pub sim_speed: u32,
    pub paused: bool,
    pub middle_mouse_dragging: bool,
    pub ctrl_left_mouse_dragging: bool,
    pub ctrl_down: bool,
    pub shift_down: bool,
    /// last known cursor position in physical pixels. winit, unlike sdl, has no
    /// "current mouse position" query, so we track it on every `CursorMoved`.
    pub last_mouse_pos: Option<(i32, i32)>,
    /// box-select drag origin (set on empty-space left-press, cleared on release).
    pub selection_box_start: Option<(i32, i32)>,
    /// set by the ui / escape handling to request app exit; the event loop reads
    /// it (egui and the input handler can't call `event_loop.exit()` directly).
    pub quit_requested: bool,
}

impl ControlState {
    pub fn new(selection: Vec<EntityId>) -> Self {
        Self {
            selection,
            debug_enabled: false,
            track_mode: false,
            sim_speed: 1,
            paused: false,
            middle_mouse_dragging: false,
            ctrl_left_mouse_dragging: false,
            ctrl_down: false,
            shift_down: false,
            last_mouse_pos: None,
            selection_box_start: None,
            quit_requested: false,
        }
    }
}
