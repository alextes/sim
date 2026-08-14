use std::collections::BTreeMap;

use crate::world::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyDialog {
    Overview,
    Logistics,
    Procurement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BodyDialogs {
    pub overview: bool,
    pub logistics: bool,
    pub procurement: bool,
}

impl BodyDialogs {
    fn set_open(&mut self, dialog: BodyDialog, open: bool) {
        match dialog {
            BodyDialog::Overview => self.overview = open,
            BodyDialog::Logistics => self.logistics = open,
            BodyDialog::Procurement => self.procurement = open,
        }
    }

    fn any_open(self) -> bool {
        self.overview || self.logistics || self.procurement
    }
}

#[derive(Debug, Default)]
pub struct UiState {
    pub owned_bodies_open: bool,
    owned_body_cursor: Option<EntityId>,
    body_dialogs: BTreeMap<EntityId, BodyDialogs>,
}

impl UiState {
    pub fn toggle_owned_bodies(&mut self, bodies: &[EntityId], selected: Option<EntityId>) {
        if self.owned_bodies_open {
            self.owned_bodies_open = false;
        } else {
            self.open_owned_bodies(bodies, selected);
        }
    }

    pub fn open_owned_bodies(&mut self, bodies: &[EntityId], selected: Option<EntityId>) {
        self.owned_bodies_open = true;
        self.owned_body_cursor = selected
            .filter(|body| bodies.contains(body))
            .or_else(|| bodies.first().copied());
    }

    pub fn owned_body_cursor(&self) -> Option<EntityId> {
        self.owned_body_cursor
    }

    pub fn set_owned_body_cursor(&mut self, body: EntityId) {
        self.owned_body_cursor = Some(body);
    }

    pub fn normalize_owned_body_cursor(&mut self, bodies: &[EntityId]) {
        if !self
            .owned_body_cursor
            .is_some_and(|body| bodies.contains(&body))
        {
            self.owned_body_cursor = bodies.first().copied();
        }
    }

    pub fn move_owned_body_cursor(&mut self, bodies: &[EntityId], backwards: bool) {
        self.normalize_owned_body_cursor(bodies);
        let Some(current) = self.owned_body_cursor else {
            return;
        };
        let Some(index) = bodies.iter().position(|&body| body == current) else {
            return;
        };
        let next = if backwards {
            index.checked_sub(1).unwrap_or(bodies.len() - 1)
        } else {
            (index + 1) % bodies.len()
        };
        self.owned_body_cursor = Some(bodies[next]);
    }

    pub fn open_body_dialog(&mut self, body: EntityId, dialog: BodyDialog) {
        self.body_dialogs
            .entry(body)
            .or_default()
            .set_open(dialog, true);
    }

    pub fn body_dialogs(&self) -> Vec<(EntityId, BodyDialogs)> {
        self.body_dialogs
            .iter()
            .map(|(&body, &dialogs)| (body, dialogs))
            .collect()
    }

    pub fn update_body_dialogs(&mut self, body: EntityId, dialogs: BodyDialogs) {
        if dialogs.any_open() {
            self.body_dialogs.insert(body, dialogs);
        } else {
            self.body_dialogs.remove(&body);
        }
    }

    pub fn has_open_windows(&self) -> bool {
        self.owned_bodies_open || self.body_dialogs.values().any(|dialogs| dialogs.any_open())
    }

    pub fn close_all(&mut self) {
        self.owned_bodies_open = false;
        self.body_dialogs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_dialogs_are_independent_and_pruned_when_closed() {
        let mut state = UiState::default();
        state.open_body_dialog(7, BodyDialog::Overview);
        state.open_body_dialog(7, BodyDialog::Logistics);

        let (_, mut dialogs) = state.body_dialogs()[0];
        assert!(dialogs.overview);
        assert!(dialogs.logistics);
        assert!(!dialogs.procurement);

        dialogs.overview = false;
        dialogs.logistics = false;
        state.update_body_dialogs(7, dialogs);

        assert!(state.body_dialogs().is_empty());
    }

    #[test]
    fn owned_body_cursor_prefers_selection_and_wraps() {
        let mut state = UiState::default();
        let bodies = [3, 7, 11];

        state.open_owned_bodies(&bodies, Some(7));
        assert_eq!(state.owned_body_cursor(), Some(7));

        state.move_owned_body_cursor(&bodies, false);
        assert_eq!(state.owned_body_cursor(), Some(11));
        state.move_owned_body_cursor(&bodies, false);
        assert_eq!(state.owned_body_cursor(), Some(3));
        state.move_owned_body_cursor(&bodies, true);
        assert_eq!(state.owned_body_cursor(), Some(11));
    }

    #[test]
    fn owned_body_cursor_recovers_when_body_is_no_longer_owned() {
        let mut state = UiState::default();
        state.open_owned_bodies(&[3, 7], Some(7));

        state.normalize_owned_body_cursor(&[3]);

        assert_eq!(state.owned_body_cursor(), Some(3));
    }
}
