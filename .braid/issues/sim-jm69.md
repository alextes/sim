---
schema_version: 9
id: sim-jm69
title: improve game testability for agent-driven development
priority: P2
status: done
type: design
deps: []
owner: null
created_at: 2026-01-26T13:45:55.721053Z
started_at: 2026-07-08T14:19:09.346756Z
completed_at: 2026-07-08T14:30:32.535869Z
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

## design

### current state (2026-07-08)

the architecture is already in good shape for state-based testing; parts of this
issue have been overtaken by events since it was filed:

- `World::update(dt)` is fully headless: no rendering dependency, fixed
  timestep, dt-driven. the sim core can be driven from any test.
- player intent flows through the `Command` queue, so actions are plain data
  and easy to inject in tests.
- 41 passing unit tests cover spawning, locations, resources, command
  processing, economy edge cases, infrastructure, viewport, and input.
- `populate_initial_galaxy` takes `&mut impl Rng`; map generation tests
  already use seeded `StdRng`.

### gaps

1. **determinism leaks** — `add_sol_system` (map_generation.rs) and
   `App::new` (app.rs) call `rand::rng()` internally, so a full game start is
   not reproducible from a seed.
2. **no shared test fixtures** — every test hand-assembles `celestial_data`,
   `infrastructure`, etc. repetitive now, brittle as world state grows.
3. **no multi-tick integration tests** — nothing runs `world.update()` over
   many ticks and asserts a full behavior loop (e.g. mining ship: idle →
   travel → mine → return → sell, with correct credits/stocks at the end).
   highest-value gap: exactly what an agent needs to verify a change didn't
   break the economy.
4. **no invariant checks** — e.g. stocks never negative, infra count within
   body capacity, ship home bases exist, orbital anchors exist.

### options considered

- **lean slice (chosen)** — fix determinism, add a world-builder fixture,
  add multi-tick integration tests, add an invariant-check helper called
  inside those tests. no new dependencies.
- **full menu** — lean slice plus proptest property testing and insta
  snapshot testing. rejected: snapshot tests churn on unrelated changes and
  world state is small enough for direct assertions; an invariant helper
  gives ~80% of proptest's value here without a new dependency.
- **integration tests only** — rejected: without seedable generation and
  fixtures the tests are noisy to write and nondeterministic to debug.

additionally a **headless scenario runner** is included (human-approved):
`sim --headless --ticks N --seed S` runs the real game world without a
window and dumps world state as text/JSON. gives agents an observability
channel for actual runs (not just tests) and helps debug civilian ai
behavior (relevant to sim-16v9).

### scope

- `src/map_generation.rs` — thread `&mut impl Rng` through `add_sol_system`
  and any other internal `rand::rng()` call sites
- `src/app.rs` — accept an optional seed, construct `StdRng` from it
- new `src/world/test_support.rs` (or `#[cfg(test)]` module) — `WorldBuilder`
  fixture + `check_invariants(&World)` helper
- new `tests/` or `src/world/integration_tests.rs` — multi-tick economy
  loop tests
- `src/main.rs` — headless mode arg parsing and run path

### planned implementation issues

1. seedable world generation (determinism plumbing)
2. world-builder test fixture + invariant-check helper
3. multi-tick integration tests for the civilian economy loop (deps: 1, 2)
4. headless scenario runner CLI (deps: 1)