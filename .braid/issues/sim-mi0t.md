---
schema_version: 9
id: sim-mi0t
title: fix ship build affordability and missing-resource reporting
priority: P2
status: done
deps: []
owner: null
created_at: 2026-06-25T08:35:03.107533Z
started_at: 2026-06-25T09:56:40.27734Z
completed_at: 2026-06-25T10:03:12.083704Z
acceptance:
- ship buildable definitions list all currently buildable ship types and their ordered resource costs in one place
- shipyard menu labels and affordability checks use the shared buildable definitions
- BuildShip command processing rechecks shipyard stocks and rejects unaffordable builds without deducting resources or spawning ships
- civilian ai no longer loses credits when a mining ship build is rejected for missing shipyard resources
- missing-resource reporting is deterministic when multiple resources are short
- tests cover rejected builds, deterministic missing-resource selection, and the civilian credit-drain regression
---

centralize ship build affordability around one ordered set of buildable ship definitions. ship costs should be expressed as resource + quantity pairs, checked by the shipyard menu before queueing and checked again by command processing before deducting resources or spawning ships. command processing remains the final authority and rejects builds that no longer have enough shipyard resources. missing-resource messages must be deterministic.