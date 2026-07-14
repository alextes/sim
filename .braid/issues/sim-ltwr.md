---
schema_version: 9
id: sim-ltwr
title: add deterministic game screenshot capture
priority: P2
status: done
deps:
- sim-xagw
owner: null
created_at: 2026-07-14T08:57:02.838277Z
started_at: 2026-07-14T08:57:06.341642Z
completed_at: 2026-07-14T09:47:53.70311Z
acceptance:
- sim --seed S --ticks N --screenshot PATH renders the real game world and exits
- the PNG includes both the world render and egui overlay
- capture dimensions and simulation state are deterministic from CLI inputs
---

add a one-shot native GPU readback path for agent visual feedback without requiring browser or OS-level screen capture permissions.