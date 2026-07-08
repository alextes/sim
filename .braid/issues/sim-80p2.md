---
schema_version: 9
id: sim-80p2
title: add world-builder test fixture and invariant checker
priority: P2
status: open
deps: []
owner: null
created_at: 2026-07-08T14:30:19.124151Z
acceptance:
- WorldBuilder composes test worlds (star, owned planet with data+infra, ships) without hand-inserting hashmaps
- 'check_invariants(&World) asserts: stocks never negative, infra counts within body capacity, ship home bases and orbital anchors exist'
- at least one existing hand-assembled test is migrated to the builder
---
