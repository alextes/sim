---
schema_version: 9
id: sim-7pun
title: design simple shipyard menu build boundary
priority: P2
status: open
type: design
deps: []
owner: null
created_at: 2026-06-25T08:35:03.12059Z
---

propose a small boundary between shipyard menu presentation and world/domain build rules. the goal is to avoid duplicating ship build affordability logic while keeping the current menu simple until the larger menu rework happens. expected output: a recommended domain helper/API shape, scope boundaries for the menu, and one or more follow-up implementation issues if needed.

recommended starting direction: expose a small domain-side helper for available ship builds and affordability results that the menu can render, while `BuildShip` command processing stays authoritative for final validation and mutation.
