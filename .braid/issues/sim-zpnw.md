---
schema_version: 9
id: sim-zpnw
title: plan bounded storage and procurement implementation
priority: P2
status: done
type: design
deps: []
owner: null
created_at: 2026-08-04T16:43:33.76802Z
started_at: 2026-08-04T16:43:53.713932Z
completed_at: 2026-08-04T16:46:17.275568Z
---

turn the canonical storage and procurement design into small implementation issues with explicit dependencies and acceptance criteria.

## canonical game design

the mechanic is specified in `docs/design/storage-procurement-economy.md` and indexed from `docs/game_design.md`. update those docs when the design changes; this issue records implementation planning only.

## implementation issues

1. `sim-csgb`: encapsulate existing layer-aware stocks behind the inventory component without changing behavior.
2. `sim-h8g3`: add storage and dock infrastructure effects.
3. `sim-4mvc`: enforce bounded deposits throughout production and logistics.
4. `sim-aro7`: split recurring consumption from procurement policy and add explicit accounts.
5. `sim-dz95`: derive bounded procurement demand from local construction work.
6. `sim-ptvx`: make deliveries transactional and throughput-limited.
7. `sim-1zfp`: score civilian mining opportunities by expected return.
8. `sim-4jdi`: gate civilian mining-ship investment on projected payback.
9. `sim-f83n`: charge infrastructure maintenance and track arrears.
10. `sim-4n64`: expose storage, procurement, docks, construction blockage, and upkeep in the UI.
11. `sim-2x3a`: add an end-to-end deterministic procurement economy scenario.

## dependency alignment

- stock layers and future transfers align with `sim-ax70`.
- infrastructure effects build on `sim-a80r`; storage construction capacity also uses `sim-jtkh` and catalog-driven behavior aligns with `sim-xv91`.
- deterministic economy validation builds on `sim-80p2` and `sim-ac90`.

## approval

approved by the user on 2026-08-04, including the recommended componentized model and all listed defaults.
