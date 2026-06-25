---
schema_version: 9
id: sim-vgn5
title: simplify rendering setup boundaries
priority: P2
status: done
deps: []
owner: null
created_at: 2026-06-24T21:21:36.943824Z
started_at: 2026-06-24T21:21:39.581346Z
completed_at: 2026-06-24T21:24:48.26399Z
acceptance:
- GpuState only owns window, surface, device, queue, and surface config
- world sprite and line rendering live behind a separate renderer
- egui frame input, texture, buffer, and paint boilerplate is encapsulated in EguiLayer
- stale SDL dependency and README setup instructions are removed
---

separate raw wgpu setup from world rendering, hide egui frame plumbing behind the egui integration layer, and remove stale SDL setup references.