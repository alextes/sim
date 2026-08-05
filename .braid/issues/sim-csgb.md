---
schema_version: 9
id: sim-csgb
title: encapsulate layer-aware stockpile access
priority: P2
status: open
deps:
- sim-ax70
owner: null
created_at: 2026-08-04T16:44:57.126851Z
acceptance:
- inventory operations address every layer available to solid bodies, gas giants, and stars
- inventory operations cover amount, used capacity, free capacity, bounded deposit, and withdrawal
- production, consumption, construction, shipbuilding, UI, and delivery code no longer mutate body stock maps directly
- existing representative economy behavior and deterministic ordering are preserved
---

build on the existing primary and orbital stock maps and `ConstructionLayer` accessors by introducing one inventory boundary keyed by anchor body and logistics layer. migrate direct stock reads and writes without intentionally changing current capacity or construction behavior.

canonical design: `docs/design/storage-procurement-economy.md`.
