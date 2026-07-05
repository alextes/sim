---
schema_version: 9
id: sim-adcj
title: add planet overview state and input
priority: P2
status: done
deps:
- sim-miel
- sim-r1kx
owner: null
created_at: 2026-06-30T20:42:48.864909Z
started_at: 2026-06-30T23:13:19.448729Z
completed_at: 2026-06-30T23:14:54.881077Z
acceptance:
- GameState has a PlanetOverview state with optional selected body id
- pressing o while playing opens the planet overview
- escape closes the overview back to Playing
- tests cover the new input/state transitions where existing input tests make this practical
---

add the planet overview game state and gameplay input entry point. the overview should open from gameplay with the o key, carry an optional selected body id, close on escape like other modal states, and reuse ControlState.selection when choosing a body.