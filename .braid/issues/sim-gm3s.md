---
schema_version: 9
id: sim-gm3s
title: implement planet overview ui
priority: P2
status: done
deps:
- sim-adcj
- sim-r1kx
owner: null
created_at: 2026-06-30T20:42:56.729441Z
started_at: 2026-06-30T23:16:00.518768Z
completed_at: 2026-06-30T23:16:59.704856Z
acceptance:
- overview shows owned bodies from the shared query in deterministic order
- selecting a body updates the overview selection and ControlState.selection
- detail area shows name, body kind, population, civilian credits, yields, stocks, infrastructure, and construction queue when present
- build action opens BuildMenu Main for selected owned bodies with buildings
- shipyard action is enabled only when the selected body has a Shipyard and opens ShipyardMenu
- no planet type, capacity, surface/orbit stock, or orbital shipyard model is introduced in this issue
---

render the planet overview modal using the owned body query and planet overview state. show a scrollable owned-body list, a detail area for the selected body, existing stats only, and actions that open the current build and shipyard flows when allowed.