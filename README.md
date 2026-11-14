# sim

![sim logo](sim.png)

## building from source

you need rust and cargo installed, preferably via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# follow the on-screen instructions, then ensure the stable toolchain is default:
rustup default stable
```

## agent screenshots

render a deterministic game frame directly to a PNG and exit:

```bash
cargo run -- --seed 42 --start playing --ticks 300 \
  --screenshot target/agent-captures/game.png --width 800 --height 600
```

the capture uses the native wgpu and egui renderers, advances the simulation by
the requested number of fixed ticks, and does not depend on browser rendering.
use `--start main-menu` without `--ticks` to capture the main menu.
use `--start planet-overview` to capture the owned-bodies launcher and selected-body foldout.
run `cargo run -- --help` for all options.

## headless scenario runs

advance the real seeded world without creating a window and print its final
state as JSON on stdout:

```bash
cargo run -- --headless --seed 42 --ticks 6000 > target/world-42.json
```

the dump covers player credits and every entity sorted by ID: name, type,
position, body credits, population, stocks, infrastructure, and ship type,
cargo, civilian AI state, routes, and move orders. it uses the same world setup
and fixed-tick advance as `--screenshot`, so the same seed and tick count always
produce a byte-identical dump that can be diffed between builds. logs go to
stderr. `--seed` defaults to 0.

## trying the economy inspector

run `cargo run`, start the game, and press `o` to toggle the owned-bodies launcher.
click a body name in the launcher, or use the up/down arrows and enter, to select
it and open its overview. select an owned body on the map to show its attached
foldout. use `i`, `l`, and `p` to open independent overview, logistics, and
procurement windows for the selected body. the same actions are available as
labeled foldout buttons.

press `f4` to show the debug overlay and toggle autonomous civilian AI. select a
civilian mining ship to inspect its active route, live market quote, expected
travel and mining time, operating costs, and profit per cycle.
