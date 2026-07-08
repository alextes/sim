---
schema_version: 9
id: sim-gmcp
title: add headless scenario runner
priority: P2
status: open
deps:
- sim-xagw
owner: null
created_at: 2026-07-08T14:30:27.027399Z
acceptance:
- sim --headless --ticks N --seed S runs the real world without creating a window
- dumps world state (entities, credits, stocks, ship states) as text or JSON on exit
- same seed and tick count produce identical dumps
---
