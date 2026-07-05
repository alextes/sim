---
schema_version: 9
id: sim-r1kx
title: organize planetary development design docs
priority: P2
status: done
deps: []
owner: null
created_at: 2026-06-30T18:40:22.665797Z
started_at: 2026-06-30T18:41:35.155757Z
completed_at: 2026-06-30T18:42:51.943551Z
acceptance:
- docs/game_design.md links to the new planetary development docs and stays readable as the main design index
- planet overview, planet traits, infrastructure units, orbital infrastructure, launch capacity, space elevators, mass drivers, spaceports, and landing economics are captured in focused docs
- each focused doc separates decided v1 direction from maybe-later ideas
- all prose follows repo lowercase style
---

split the main game design doc into a clearer linked design structure for planetary development. keep docs/game_design.md as the high-level index and move detailed planet overview, infrastructure, and surface-orbit logistics notes into focused docs. mark ideas by maturity: decided, candidate, maybe later, and open questions. default doc layout: docs/game_design.md remains the index; docs/design/planetary-development.md covers planet overview, ownership, stats, planet size/type, and infrastructure units; docs/design/surface-orbit-logistics.md covers orbital vs surface stockpiles, launch capacity, space elevators, mass drivers, spaceports, and landing economics; docs/design/open-ideas.md parks promising ideas that are not ready for implementation design.