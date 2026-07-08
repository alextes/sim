---
schema_version: 9
id: sim-40hd
title: fix orbit ring translation jitter
priority: P2
status: done
deps: []
owner: null
created_at: 2026-07-05T20:28:19.452263Z
started_at: 2026-07-05T20:28:22.644541Z
completed_at: 2026-07-05T20:30:07.196913Z
acceptance:
- orbit rings centered on moving bodies use precise f64 anchor positions
- the moon orbit ring around earth translates smoothly with earth instead of snapping by tile
- add or update a focused regression test for fractional anchor positions where practical
---

orbit rings are centered using rounded world positions in the line batch. nested rings, such as the moon orbit around earth, therefore translate in tile-sized snaps while the anchored body sprite uses precise f64 coordinates. investigate whether nested orbital update ordering also contributes, but start with making ring centers use precise coordinates.