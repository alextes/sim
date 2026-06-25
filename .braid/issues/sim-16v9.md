---
schema_version: 9
id: sim-16v9
title: add toggle for autonomous civilian AI in debug menu
priority: P2
status: open
type: design
deps: []
owner: null
created_at: 2026-01-27T16:46:02.626595Z
---

the `enable_civilian_ai` flag on `World` controls whether mining ships without explicit routes use random AI behavior. currently it defaults to `false` with no way to change it at runtime.

## goal

allow toggling this flag from the debug menu so players/developers can enable or disable autonomous civilian ship behavior on demand.

## considerations

- where in the debug menu should this toggle appear?
- what label/description makes the behavior clear?
- should the current state be visible (on/off indicator)?
- keybind within debug menu vs clickable option?