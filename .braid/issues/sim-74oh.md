---
schema_version: 9
id: sim-74oh
title: design planetary infrastructure model v1
priority: P2
status: done
type: design
deps:
- sim-r1kx
owner: null
created_at: 2026-06-30T18:40:31.804452Z
started_at: 2026-06-30T20:43:15.815571Z
completed_at: 2026-06-30T22:12:14.981789Z
---

design the first unit-based planetary infrastructure model. cover planet/body profiles with type, size, gravity, atmosphere, and capacity; infrastructure definitions with ground or orbit domain, category, cost, capacity use, and effects; planet type modifiers; and the first infrastructure categories: energy, mining, research, shipbuilding, and manufacturing. manufacturing is the v1 home for refining raw materials unless a later design splits refining into its own category. later implementation tests should cover planet size/type capacity, infrastructure availability, and deterministic build options.

## design

### options considered

1. metadata around existing `BuildingType`

keep `BuildingType` as the unit identity stored in `EntityBuildings`, but move labels, costs, domain, category, capacity use, and effects into ordered infrastructure definitions. add body profiles as a separate world component keyed by entity id.

trade-offs: this fits the current code and keeps implementation small, but the name `BuildingType` remains a little old-fashioned until a later rename.

2. replace `BuildingType` with a new `InfrastructureType`

rename the model to infrastructure everywhere and migrate queues, costs, resources, ui, and tests to the new type.

trade-offs: cleaner vocabulary, but it is a wider refactor and the repo already has an unused `src/infrastructure.rs` path that would make this easy to mix up.

3. separate ground and orbital infrastructure into separate storage now

store surface infrastructure and orbital infrastructure in different maps or separate entities immediately.

trade-offs: this lines up with the long-term logistics model, but it pulls surface-orbit ownership questions into v1 before `sim-ax70` has designed them.

### recommendation

use option 2, but keep it bounded.

v1 should move the code vocabulary to infrastructure now, before planet overview, logistics, and surface/orbit work add more references to the old building model. rename the active model from `BuildingType`/`EntityBuildings` to infrastructure concepts, clean up `src/infrastructure.rs` immediately, and introduce one ordered infrastructure catalog as the source of truth for labels, costs, domain, category, capacity use, and effects.

the bound is important: this is a vocabulary and metadata migration, not a logistics rewrite. keep the existing unit-count and construction-queue behavior, and do not split ground/orbit storage yet.

### model

body profile:

- add a world component such as `body_profiles: HashMap<EntityId, BodyProfile>`.
- `BodyProfile` has `class`, `size`, `gravity`, and `atmosphere`.
- `BodyClass` starts with `Greenhouse`, `Barren`, `Volcanic`, `Oceanic`, `Lunar`, and `GasGiant`.
- `BodySize` starts with `Tiny`, `Small`, `Medium`, `Large`, and `Giant`.
- `Atmosphere` starts with `None`, `Thin`, `Breathable`, `Dense`, and `Toxic`.
- capacity is derived from size for v1 instead of stored independently.
- initial capacity defaults: tiny 4, small 8, medium 16, large 28, giant 48.

infrastructure definitions:

- add ordered definitions for every buildable `InfrastructureType`.
- each definition has: type, name, domain, category, ordered costs, capacity use, and effect.
- `InfrastructureDomain` starts with `Ground` and `Orbit`.
- `InfrastructureCategory` starts with `Energy`, `Mining`, `Research`, `Shipbuilding`, `Manufacturing`, `Agriculture`, and `Construction`.
- `InfrastructureEffect` can be simple and descriptive for v1: `Energy`, `MineRaw`, `Research`, `Shipbuilding`, `Manufacture`, `Agriculture`, `ConstructionCapacity`, or `None`.
- costs should use ordered resource + quantity pairs, not `HashMap`, so build quotes are deterministic.

initial catalog:

- `Mine`: ground, mining, capacity 1, produces raw extraction effect.
- `FuelCellCracker`: ground, manufacturing, capacity 2, manufacturing/refining effect.
- `Farm`: ground, agriculture, capacity 1, food/agriculture effect.
- `Shipyard`: orbit, shipbuilding, capacity 4, shipbuilding effect.
- `ConstructionFactory`: ground, construction, capacity 2, construction capacity effect.
- `SolarPanel`: orbit, energy, capacity 1, energy effect.
- add `ResearchLab`: ground, research, capacity 2, research effect.

capacity rules:

- total used capacity is the sum of completed infrastructure units times capacity use.
- queued infrastructure should count against capacity at queue time, so players cannot overbook a planet by queueing many projects.
- build command processing remains authoritative: it should reject builds that exceed body capacity or lack resources.
- v1 uses one shared capacity budget across ground and orbit. separate budgets are deferred to logistics/infrastructure follow-up work.

planet type modifiers:

- v1 stores body class, gravity, and atmosphere, but only body class needs immediate modifiers.
- body class modifiers should be exposed through helper methods and tests, even if only a few effects consume them initially.
- keep modifier values conservative and simple: oceanic favors agriculture, barren favors solar/orbital energy, volcanic favors mining/manufacturing, gas giant favors orbital infrastructure, greenhouse penalizes ground habitability.
- do not implement population happiness, habitability, landing cost, or surface-orbit logistics here.

### scope

- add profile types and a `body_profiles` world component.
- assign profiles during map generation for sol bodies and generated bodies.
- replace `BuildingType` with `InfrastructureType` as the active infrastructure unit identity.
- replace `EntityBuildings` with an infrastructure-owned container such as `EntityInfrastructure`.
- rename or adapt `build_queue` to an infrastructure construction queue, while preserving existing queue behavior.
- turn `src/infrastructure.rs` into the real infrastructure definition module, or delete it if the real module lives elsewhere. do not leave a duplicate unused model behind.
- replace hard-coded building labels/costs with the ordered infrastructure definition catalog.
- update build menu quotes and command processing to use ordered definitions and capacity checks.
- keep existing resource production behavior mostly unchanged, except reading effects/categories where it is straightforward.
- do not implement surface/orbit stock separation, launch capacity, space elevators, mass drivers, or landing economics.

### planned implementation issues

1. add body profiles and generated defaults

introduce body profile types, store them on `World`, assign profiles in spawning/map generation, and test default profile assignment.

2. migrate building model to infrastructure model

rename the active building types and containers to infrastructure concepts, update commands, ui, resources, map generation, and tests, and clean up the duplicate `src/infrastructure.rs` path so only one infrastructure model remains.

3. add infrastructure definition catalog

create ordered infrastructure definitions in the active infrastructure module for the current buildable infrastructure plus `ResearchLab`, and migrate labels and costs to that catalog.

4. enforce infrastructure capacity

compute completed plus queued capacity use, reject over-capacity build commands, disable or flag over-capacity build quotes in the menu, and test accepted/rejected capacity cases.

5. wire infrastructure categories and effects

update resource/construction systems to use the definition catalog where practical, preserving current behavior while making energy, mining, research, shipbuilding, manufacturing, agriculture, and construction categories visible to ui/tests.
