---
schema_version: 9
id: sim-4jdi
title: gate civilian mining ship investment on projected payback
priority: P2
status: open
deps:
- sim-1zfp
owner: null
created_at: 2026-08-04T16:45:46.71031Z
acceptance:
- investment requires sufficient civilian credits, materials, and shipyard capability
- the best visible route must meet a configurable payback threshold
- cooldown and pending-build checks prevent duplicate build commands
- delivery income funds later civilian investment
- no mining ship is commissioned when all opportunities have nonpositive expected return
---

replace threshold-only civilian ship construction with an opportunity-aware investment decision using body civilian savings.

canonical design: `docs/design/storage-procurement-economy.md`.
