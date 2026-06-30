# planetary development

this doc organizes owned planet overview, body traits, and infrastructure. it is intentionally broader than the current code so the next design issues can choose small implementation slices.

## maturity legend

- decided v1: direction ready to design or implement soon.
- candidate: likely direction, but still needs design work.
- maybe later: interesting idea, intentionally out of scope for the next slice.
- open question: decision needed before implementation.

## decided v1

owned bodies:

- an owned body is any player-controlled celestial body.
- the first overview should include owned planets, moons, and gas giants if they are player-controlled.
- the overview should sort owned bodies deterministically, probably by system, then body type priority, then name or entity id.

planet overview:

- the player needs a scrollable overview of owned bodies.
- each row or detail panel should show name, body kind, ownership, population, civilian credits, resource yields, stocks, and infrastructure.
- selecting a body from the overview should make it the active body for inspection and actions.
- actions should include opening the build flow and opening an associated shipyard when one exists.

planet traits:

- each developed body should eventually have a body profile separate from transient stocks and population.
- the profile should include body type, size grade, gravity, atmosphere, and capacity.
- size should provide the main infrastructure capacity budget.
- body type should modify infrastructure efficiency and habitability.

infrastructure:

- infrastructure is unit-based rather than fixed building slots.
- every infrastructure definition should have a domain, category, cost, capacity use, and effect.
- domains start as ground and orbit.
- first categories are energy, mining, research, shipbuilding, and manufacturing.
- manufacturing is the v1 home for refining raw materials unless a later design splits refining into its own category.

## candidate

body types:

- greenhouse, barren, volcanic, oceanic, and gas giant are early useful types.
- moons may use the same type system plus lower capacity.
- asteroids and belts can be represented later as smaller, low-capacity mining bodies or resource sites.

size grades:

- tiny, small, medium, large, and giant are enough for early design.
- size should affect infrastructure capacity, population potential, and the strategic value of a body.

infrastructure examples:

- energy: ground power plants, orbital solar panels.
- mining: mines, orbital mining bases, gas mining stations.
- research: labs and observatories.
- shipbuilding: shipyards and related orbital facilities.
- manufacturing: construction factories, refineries, and material processors.

orbital build capability:

- a planet should be able to own or coordinate things built in orbit.
- orbital infrastructure should not be hidden inside ground infrastructure, because it affects logistics and shipyard access differently.
- solar panels are the first obvious orbital energy infrastructure.

## maybe later

- districts, jobs, housing, amenities, and detailed workforce allocation.
- terraforming and long-term atmosphere or gravity modification.
- special planet features or unique landmarks.
- separate civilian, state, and military ownership for infrastructure.
- habitats at lagrange points as fully separate colonies.

## open questions

- should the first overview be opened from a keybind, a hud button, or both?
- should overview selection replace the current world selection or mirror it?
- should orbital infrastructure live on the planet record, on separate orbital entities, or both?
- how visible should exact efficiency modifiers be in the v1 ui?
- should capacity be one shared budget or separate budgets for ground, orbit, and launch/logistics?
