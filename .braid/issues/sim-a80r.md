---
schema_version: 9
id: sim-a80r
title: add infrastructure definition catalog
priority: P2
status: done
deps:
- sim-cw48
- sim-r1kx
owner: null
created_at: 2026-06-30T22:11:54.470629Z
started_at: 2026-08-05T10:14:52.606388Z
completed_at: 2026-08-05T10:21:09.920795Z
acceptance:
- catalog defines every buildable InfrastructureType in deterministic order
- definitions include name, ground/orbit domain, category, ordered costs, capacity use, and effect
- current labels and costs are migrated to the catalog
- ResearchLab is added as ground research infrastructure
- build menu quotes and build command cost checks use catalog costs instead of ad hoc HashMaps
- tests cover deterministic ordering and representative definitions
---

add an ordered infrastructure definition catalog in the active infrastructure module. the catalog should own labels, domain, category, ordered costs, capacity use, and simple effects for all buildable infrastructure, including the new ResearchLab.