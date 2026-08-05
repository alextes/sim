---
schema_version: 9
id: sim-4n64
title: expose storage and procurement controls in the planet overview
priority: P2
status: open
deps:
- sim-4mvc
- sim-aro7
- sim-dz95
- sim-ptvx
- sim-f83n
owner: null
created_at: 2026-08-04T16:45:46.86863Z
acceptance:
- overview shows primary-layer and orbital stock usage, capacity, and free space
- overview shows wanted quantity, current purchase price, and configured reserve, maximum price, and spend cap
- player can enable or disable procurement and edit its limits
- overview shows dock throughput, waiting ships, construction material blockage, upkeep, and arrears
- resource and body ordering remains deterministic
---

show the bounded logistics economy in the planet overview and let the player configure automatic procurement limits.

canonical design: `docs/design/storage-procurement-economy.md`.
