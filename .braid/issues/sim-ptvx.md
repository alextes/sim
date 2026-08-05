---
schema_version: 9
id: sim-ptvx
title: make deliveries transactional and throughput limited
priority: P2
status: open
deps:
- sim-h8g3
- sim-4mvc
- sim-aro7
owner: null
created_at: 2026-08-04T16:45:26.760023Z
acceptance:
- delivery debits the buyer and credits the ship home civilian economy without minting credits
- accepted cargo is bounded by wanted quantity, funds, free storage, and remaining dock throughput
- unaccepted cargo remains aboard and ships can enter a waiting-to-unload state
- ship home base and sell destination are independent
- multiple arrivals are processed in deterministic order
---

replace unconditional ship sales with partial atomic unloading constrained by demand, buyer funds, storage capacity, and dock throughput.

canonical design: `docs/design/storage-procurement-economy.md`.
