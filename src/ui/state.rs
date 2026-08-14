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
    body_dialogs: BTreeMap<EntityId, BodyDialogs>,
}

impl UiState {
    pub fn toggle_owned_bodies(&mut self) {
        self.owned_bodies_open = !self.owned_bodies_open;
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
}
