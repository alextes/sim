---
schema_version: 9
id: sim-aro7
title: add procurement policies and economic accounts
priority: P2
status: done
deps:
- sim-csgb
owner: null
created_at: 2026-08-04T16:45:26.595235Z
started_at: 2026-08-06T09:47:05.525403Z
completed_at: 2026-08-06T09:52:15.842454Z
acceptance:
- recurring consumption rates are not mutated while the simulation runs
- procurement policy supports reserve target, maximum unit price, enabled state, and optional periodic spend cap
- purchase quantity is bounded by shortage, free storage, and buyer funds
- price increases monotonically with shortage and no quote exists at target or when storage is full
- account debit and credit operations conserve credits and refinery input demand no longer accumulates each tick
---

separate recurring consumption from procurement intent, derive bounded purchase opportunities, and introduce explicit player-treasury and body-civilian account transfers.

canonical design: `docs/design/storage-procurement-economy.md`.
