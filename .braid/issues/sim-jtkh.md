---
schema_version: 9
id: sim-jtkh
title: enforce infrastructure capacity
priority: P2
status: done
deps:
- sim-je9s
- sim-a80r
- sim-r1kx
owner: null
created_at: 2026-06-30T22:12:01.278993Z
started_at: 2026-08-05T11:00:55.302598Z
completed_at: 2026-08-05T11:04:07.196811Z
acceptance:
- completed plus queued infrastructure capacity use is computed from the catalog
- build command processing rejects over-capacity infrastructure builds
- build menu quotes show or disable over-capacity projects before queueing
- capacity rejection does not deduct resources or mutate construction queues
- tests cover accepted builds, rejected over-capacity builds, and queued capacity counting
---

enforce v1 shared infrastructure capacity using body profile capacity and catalog capacity use. capacity should include completed infrastructure and queued construction so players cannot overbook a body.