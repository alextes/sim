---
schema_version: 9
id: sim-xagw
title: make world generation seedable
priority: P2
status: open
deps: []
owner: null
created_at: 2026-07-08T14:30:12.157804Z
acceptance:
- add_sol_system and all map_generation internals take &mut impl Rng (no internal rand::rng())
- App::new accepts an optional seed and builds StdRng from it
- two runs with the same seed produce identical world state
---
