---
schema_version: 9
id: sim-f83n
title: charge infrastructure maintenance and track arrears
priority: P2
status: done
deps:
- sim-h8g3
- sim-aro7
owner: null
created_at: 2026-08-04T16:45:46.788591Z
started_at: 2026-08-06T11:06:58.550927Z
completed_at: 2026-08-06T11:13:27.892465Z
acceptance:
- every infrastructure definition has a maintenance credit rate
- periodic upkeep debits the correct owning account deterministically
- unpaid upkeep is recorded as arrears and affects infrastructure operation
- delinquent storage preserves existing inventory but accepts no new deposits
- UI-facing queries can explain active, inactive, and arrears states
---

add fixed periodic credit upkeep to every infrastructure definition and suspend delinquent effects without deleting stored resources.

canonical design: `docs/design/storage-procurement-economy.md`.
