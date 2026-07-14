---
schema_version: 9
id: sim-wq51
title: maintain deterministic orbital iteration order
priority: P2
status: done
deps:
- sim-ltwr
owner: null
created_at: 2026-07-14T10:06:33.402536Z
started_at: 2026-07-14T10:06:38.04769Z
completed_at: 2026-07-14T10:08:44.180686Z
acceptance:
- location system maintains parent-before-child deterministic orbital order on mutation
- simulation updates and rendering do not sort orbitals on hot paths
- orbital ordering invariants are covered by tests
---

store a deterministic orbital ID order alongside the location hashmap so frequent simulation and render reads avoid repeated sorting.