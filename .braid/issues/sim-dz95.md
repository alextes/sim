---
schema_version: 9
id: sim-dz95
title: derive procurement demand from local construction
priority: P2
status: open
deps:
- sim-csgb
- sim-aro7
owner: null
created_at: 2026-08-04T16:45:26.685799Z
acceptance:
- queued construction contributes construction-material demand only at its exact construction layer
- procurement covers a bounded horizon of upcoming work rather than the full queue lifetime
- existing continuous local construction-material consumption and pause behavior remain unchanged
- current stock and expected near-term consumption determine the outstanding construction purchase quantity
- build UI distinguishes lifetime cost from the current procurement horizon
---

derive bounded construction-material procurement from the existing layer-local construction loop. do not replace the implemented continuous consumption model or reintroduce raw-resource build charging.

canonical design: `docs/design/storage-procurement-economy.md`.
