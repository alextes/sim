---
schema_version: 9
id: sim-cw48
title: migrate building model to infrastructure model
priority: P2
status: open
deps:
- sim-r1kx
owner: null
created_at: 2026-06-30T22:11:45.967559Z
acceptance:
- BuildingType is replaced by InfrastructureType as the active infrastructure unit identity
- EntityBuildings is replaced by an infrastructure-owned container such as EntityInfrastructure
- commands, ui, resources, map generation, and tests use infrastructure terminology
- src/infrastructure.rs is either the single active infrastructure module or removed so no duplicate model remains
- existing construction queue and resource production behavior is preserved
---

rename the active building model to infrastructure concepts and clean up the duplicate infrastructure path. preserve existing unit counts, construction queue behavior, build commands, resource production behavior, and tests while moving code vocabulary away from BuildingType and EntityBuildings.