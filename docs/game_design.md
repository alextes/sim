# game design

this is the main design index for sim. keep broad vision and links here; move detailed mechanics into focused docs once they are large enough to need their own home.

## vision

sim is a top-down 2d space empire builder. the player begins in the sol system and shares the galaxy with one ai-controlled empire. victory is achieved either by military conquest or by reaching a target prestige score first.

## design map

- [planetary development](design/planetary-development.md): owned planet overview, planet traits, infrastructure units, and ground/orbit infrastructure.
- [surface orbit logistics](design/surface-orbit-logistics.md): surface and orbital stockpiles, launch capacity, space elevators, mass drivers, spaceports, and landing economics.
- [open ideas](design/open-ideas.md): promising ideas that are not ready for implementation design.

## current priority areas

- make player-owned celestial bodies easier to inspect from a planet overview.
- move infrastructure toward unit-based planetary development with explicit surface and orbital domains.
- keep shipyards and build flows simple until the menu model is redesigned.
- separate worked-out direction from maybe-later ideas so agents can safely pick up focused work.

## economy

the economy should feel alive: populated bodies create demand, local stockpiles influence prices, and civilian ships respond to profitable mining and trade opportunities.

decided direction:

- populated celestial bodies generate local demand from population, body character, and infrastructure.
- local resource prices should respond to stock relative to demand, with clamps to avoid extreme swings.
- civilian mining ships should prefer profitable mining opportunities and sell back to populated worlds.
- fuel cells are the standard in-system fuel and are produced by refining raw resources.

candidate direction:

- civilian freighters compare prices between worlds and move goods where profit remains after fuel, tariffs, and purchase costs.
- imperial finance eventually exposes taxes, tariffs, subsidies, embargoes, and state-owned fleets as strategic levers.

## research

research should unlock better industrial, logistics, and commerce tools over time.

candidate direction:

- knowledge points are produced by research infrastructure and modified by population, efficiency, and policy.
- the research tree starts with industry, logistics, and commerce pillars.
- sample unlocks include improved drills, automated cranes, futures contracts, sensor sweeps, tariff harmonization, and asteroid smelters.

## military and ships

military and civilian ships share the space economy but serve different control models.

candidate direction:

- military frames include frigates, destroyers, cruisers, carriers, fighters, bombers, explorers, colony ships, and construction ships.
- civilian ships include mining ships, freighters, migration transports, and tourism vessels.
- ship design eventually includes hull size, hardpoints, internal slots, power, and heat budgets.

## propulsion

ships use different travel modes for in-system movement and interstellar travel.

candidate direction:

- in-system drives use fuel cells and handle normal travel, combat maneuvering, and final approach.
- interstellar drives use warp cores and cannot safely activate too close to significant gravity wells.
- final approach economics should eventually connect with atmosphere, gravity, spaceports, and landing-capable ship modules.

## population

population provides workforce, research capacity, civilian economic activity, and military manpower.

candidate direction:

- initial population growth can stay as a simple annual rate.
- later growth should depend on planet quality, food, amenities, and policy.
- migration is a future mechanic driven by jobs, housing, planet quality, and imperial policy.
- bombardment and armies should eventually affect population and infrastructure.
