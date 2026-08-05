---
schema_version: 9
id: sim-1zfp
title: score civilian mining opportunities by expected return
priority: P2
status: open
deps:
- sim-ptvx
owner: null
created_at: 2026-08-04T16:45:46.623686Z
acceptance:
- route estimates include sale revenue, wanted quantity, cargo capacity, mining yield, travel time, and configured operating costs
- ships ignore nonpositive opportunities and prefer expected profit per unit time
- manual routes remain authoritative
- ties resolve deterministically by entity and resource ordering
- ships re-evaluate opportunities after completing a trip
---

replace random and yield-only mining selection with deterministic profit-per-cycle scoring over current procurement opportunities.

canonical design: `docs/design/storage-procurement-economy.md`.
