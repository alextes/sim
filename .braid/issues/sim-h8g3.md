---
schema_version: 9
id: sim-h8g3
title: add storage and dock infrastructure effects
priority: P2
status: done
deps:
- sim-a80r
- sim-jtkh
- sim-xv91
owner: null
created_at: 2026-08-04T16:45:26.418348Z
started_at: 2026-08-05T11:09:22.542055Z
completed_at: 2026-08-05T11:12:49.599419Z
acceptance:
- catalog definitions provide surface, upper-atmosphere, and orbital storage capacity plus orbital dock throughput effects
- storage and dock units consume ordinary infrastructure capacity and have deterministic costs and ordering
- starting Earth has enough explicit or settlement-core capacity for its seeded inventory
- tests cover representative capacity and throughput derivation
---

extend the infrastructure catalog with surface storage, gas-giant upper-atmosphere storage, orbital storage, and orbital unloading capacity, including starting capacity that avoids bootstrapping deadlocks.

canonical design: `docs/design/storage-procurement-economy.md`.
