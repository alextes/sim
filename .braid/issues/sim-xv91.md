---
schema_version: 9
id: sim-xv91
title: wire infrastructure categories and effects
priority: P2
status: open
deps:
- sim-a80r
- sim-r1kx
owner: null
created_at: 2026-06-30T22:12:09.809745Z
acceptance:
- resource production uses catalog effects or categories for mine, farm, and fuel/refining behavior without changing existing outputs intentionally
- construction capacity uses the catalog construction effect
- energy, mining, research, shipbuilding, manufacturing, agriculture, and construction categories are visible through definitions for ui/tests
- tests cover category/effect lookup and preserved representative production behavior
- surface/orbit logistics, launch capacity, and separate stock layers remain out of scope
---

use infrastructure catalog categories and effects where practical while preserving current gameplay behavior. resource production and construction capacity should read from infrastructure definitions instead of matching hard-coded infrastructure names where that is straightforward.