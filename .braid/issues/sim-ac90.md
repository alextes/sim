---
schema_version: 9
id: sim-ac90
title: add multi-tick economy integration tests
priority: P2
status: open
deps:
- sim-xagw
- sim-80p2
owner: null
created_at: 2026-07-08T14:30:26.994521Z
acceptance:
- 'test drives world.update() over many ticks and asserts the full mining loop: idle -> travel -> mine -> return -> sell with correct credits and stocks'
- check_invariants runs every tick during integration tests
- tests are deterministic (seeded or builder-constructed worlds only)
---
