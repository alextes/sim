---
schema_version: 8
id: sim-jm69
title: improve game testability for agent-driven development
priority: P2
status: open
type: design
deps: []
owner: null
created_at: 2026-01-26T13:45:55.721053Z
---

design a testing strategy that makes the game thoroughly testable through game state verification, enabling AI agents to confidently make changes without visual verification.

## context

agents working on this codebase cannot verify visual output - they can only verify game state changes through tests. the game should be testable enough that state-based testing provides high confidence in correctness.

## areas to explore

- what game state changes should be testable (spawning, movement, economy, combat, etc.)
- how to structure world/systems to make state transitions easily assertable
- snapshot testing for complex state
- property-based testing for game rules and invariants
- integration tests that simulate multiple game ticks
- test fixtures and world builders for common scenarios
- mocking/stubbing strategies for time, randomness, and input

## expected output

- document current test coverage and gaps
- propose testing patterns suited to game state verification
- identify refactoring needed to improve testability
- create prioritized list of test areas to implement