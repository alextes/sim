---
schema_version: 9
id: sim-2x3a
title: add a deterministic procurement economy scenario
priority: P2
status: open
deps:
- sim-80p2
- sim-ac90
- sim-dz95
- sim-ptvx
- sim-1zfp
- sim-4jdi
- sim-f83n
owner: null
created_at: 2026-08-04T16:45:46.943188Z
acceptance:
- a layer-local construction shortage creates a bounded purchase opportunity
- a civilian ship mines, travels, unloads, and transfers conserved credits
- construction resumes after delivery and the opportunity closes at its target
- storage, account, cargo, infrastructure, and entity-reference invariants hold every tick
- the scenario produces identical results for the same seed
---

exercise the complete shortage-to-investment-to-delivery-to-construction loop over many world updates with invariant checks on every tick.

canonical design: `docs/design/storage-procurement-economy.md`.
