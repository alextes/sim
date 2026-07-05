---
schema_version: 9
id: sim-je9s
title: add body profiles and generated defaults
priority: P2
status: done
deps:
- sim-r1kx
owner: null
created_at: 2026-06-30T22:11:40.008769Z
started_at: 2026-06-30T22:17:56.324077Z
completed_at: 2026-06-30T23:13:00.841646Z
acceptance:
- World stores body profiles keyed by entity id
- BodyProfile includes class, size, gravity, and atmosphere
- capacity is derived from body size using the design defaults
- sol bodies and generated planets, moons, and gas giants receive profiles
- tests cover profile assignment and capacity defaults
---

add body profile types and generated/default profile assignment for celestial bodies. profiles should cover body class, size, gravity, atmosphere, and derived v1 capacity defaults.