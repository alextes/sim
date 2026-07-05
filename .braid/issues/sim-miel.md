---
schema_version: 9
id: sim-miel
title: add owned body overview query
priority: P2
status: done
deps:
- sim-r1kx
owner: null
created_at: 2026-06-30T20:42:42.727294Z
started_at: 2026-06-30T22:16:02.660088Z
completed_at: 2026-06-30T22:17:39.947794Z
acceptance:
- query returns only player-controlled Planet, Moon, and GasGiant entities with celestial data and buildings
- stars and ships are excluded even when player-controlled
- results sort deterministically by home star id, body type priority, name, then entity id
- unit tests cover filtering and sorting
---

add a world-side query for planet overview bodies. it should return player-controlled planets, moons, and gas giants that have celestial data and buildings, exclude stars and ships for v1, and sort deterministically by home star id, body type priority, name, then entity id.