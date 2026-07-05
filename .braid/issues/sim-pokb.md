---
schema_version: 9
id: sim-pokb
title: design planet overview v1
priority: P2
status: done
type: design
deps:
- sim-r1kx
owner: null
created_at: 2026-06-30T18:40:31.794543Z
started_at: 2026-06-30T18:43:10.288141Z
completed_at: 2026-06-30T20:43:01.548768Z
---

design a first planet overview experience for player-controlled celestial bodies. include an owned-body list with deterministic ordering, a scrollable stats view for each body, selection from the overview, a build action entry point, and an action to open the associated shipyard when the selected body has one built in orbit or otherwise available. the design should converge on a simple planet overview ui model that lists player-controlled celestial bodies and exposes allowed actions. later implementation tests should cover player-owned body filtering, deterministic overview ordering, and entry points into existing build and shipyard flows.

## design

### options considered

1. modal overview state

add a `GameState::PlanetOverview` modal opened from gameplay. it renders over the world like the existing build, shipyard, and mining-route menus. selecting a row updates `ControlState.selection`, and action buttons reuse the existing build and shipyard states.

trade-offs: this is the smallest consistent change and keeps the world visible, but it is still modal and not a full management screen.

2. persistent hud panel

add a side panel that can stay open while playing and update continuously as the simulation runs. it would behave more like a management dashboard than a modal.

trade-offs: better long-term ergonomics for empire management, but it raises layout, focus, and input questions before the menu rework is ready.

3. full planet management screen

replace the current in-world overlays with a dedicated management screen for owned bodies, stats, construction, shipyards, and logistics.

trade-offs: this is likely where the design should go eventually, but it is too large for v1 and would force infrastructure/logistics decisions that are still under design.

### recommendation

use option 1: a modal planet overview state.

the v1 goal is not to solve all planet management. it should give the player one reliable place to see owned bodies, inspect key stats, select a body, and jump into existing build or shipyard flows. this lets the infrastructure and logistics design issues continue without blocking a useful overview.

### scope

- add `GameState::PlanetOverview { selected: Option<EntityId> }`.
- add an `o` gameplay keybind to open the overview. escape closes it like other modal states.
- add a testable world query for overview bodies, filtered to player-controlled `Planet`, `Moon`, and `GasGiant` entities that have `celestial_data` and `buildings`. exclude stars and ships for v1.
- sort overview bodies deterministically by home star id, body type priority (`GasGiant`, `Planet`, `Moon`), name, then entity id.
- render a resizable or fixed-width egui window with a scrollable body list and a scrollable/detail area for the selected body.
- show existing stats only: name, body kind, population, civilian credits, yields, stocks, infrastructure, and construction queue.
- selecting a body in the overview updates `ControlState.selection` to that body.
- the build action opens `GameState::BuildMenu { mode: BuildMenuMode::Main }` for selected owned bodies with buildings.
- the shipyard action is enabled when the selected body has `BuildingType::Shipyard`; clicking it opens `GameState::ShipyardMenu`.
- do not add planet type, size, ground/orbit stock separation, launch capacity, or orbital shipyard association in this issue. those belong to the infrastructure and logistics design issues.

### planned implementation issues

1. add owned body overview query

create the deterministic world-side query and unit tests for filtering and sorting owned overview bodies.

2. add planet overview state and input

add the `PlanetOverview` game state, the `o` keybind, escape handling, and basic state transitions.

3. implement planet overview ui

render the overview window with owned-body list, selected-body stats, build action, shipyard action, and tests for action state transitions where practical.
