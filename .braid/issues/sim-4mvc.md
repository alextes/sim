---
schema_version: 9
id: sim-4mvc
title: enforce bounded resource deposits
priority: P2
status: open
deps:
- sim-csgb
- sim-h8g3
owner: null
created_at: 2026-08-04T16:45:26.507558Z
acceptance:
- mining, farming, refining, initialization, and deliveries cannot raise a stockpile above capacity
- partial deposits report or retain the remainder instead of silently destroying it
- production pauses or truncates deterministically when storage is full
- inventory invariants assert nonnegative stock and used capacity not exceeding total capacity
---

apply stockpile capacity to every resource-producing and cargo-depositing path while preserving resources that cannot be deposited.

canonical design: `docs/design/storage-procurement-economy.md`.
