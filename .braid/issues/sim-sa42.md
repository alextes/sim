---
schema_version: 9
id: sim-sa42
title: simplify startup menu
priority: P2
status: done
deps: []
owner: null
created_at: 2026-06-24T21:44:33.948921Z
started_at: 2026-06-24T21:44:37.836287Z
completed_at: 2026-06-24T21:45:32.236289Z
acceptance:
- app starts at the main menu without the intro splash
- main menu shows only play and quit buttons without an egui window frame
- play enters the game and quit exits
---

replace the framed startup window/splash with a plain centered play and quit menu using dark gray, space-like button styling.