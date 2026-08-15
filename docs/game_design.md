# game design

this is the main design index for sim. keep broad vision and links here; move detailed mechanics into focused docs once they are large enough to need their own home.

## vision

sim is a top-down 2d space empire builder. the player begins in the sol system and initially shares the galaxy with one ai-controlled major faction. victory is achieved either by military conquest or by reaching a target prestige score first.

the broader faction model distinguishes three kinds of independent power:

- major factions are player-like, expansionist powers that grow territory, population, industry, technology, and military strength.
- pirate factions are non-territorial predatory networks supported by hidden bases and risk-aware raiding fleets.
- minor factions are bounded independent actors with their own interests, capabilities, and relationships, but no default goal of galactic expansion.

## design map

- [planetary development](design/planetary-development.md): owned planet overview, planet traits, infrastructure units, and ground/orbit infrastructure.
- [resources and construction](design/resources-and-construction.md): v1 raw resources, refining recipes, local construction material, layers, and capacity.
- [surface orbit logistics](design/surface-orbit-logistics.md): surface and orbital stockpiles, launch capacity, space elevators, mass drivers, spaceports, and landing economics.
- [storage and procurement economy](design/storage-procurement-economy.md): bounded stockpiles, staged construction demand, procurement policy, dock throughput, trade accounting, maintenance, and civilian investment.
- [pirate factions](design/pirate-factions.md): hidden bases, risk-aware raids, route security, discovery, and suppression.
- [minor factions](design/minor-factions.md): bounded independent powers, specialized capabilities, diplomacy, and intermediary trade.
- [open ideas](design/open-ideas.md): promising ideas that are not ready for implementation design.

## current priority areas

- make player-owned celestial bodies easier to inspect from a planet overview.
- move infrastructure toward unit-based planetary development with explicit surface and orbital domains.
- keep shipyards and build flows simple until the menu model is redesigned.
- separate worked-out direction from maybe-later ideas so agents can safely pick up focused work.

## interface presentation

- the interface uses a bundled monospace pixel font for a consistent retro technical character across platforms.
- the base interface palette is catppuccin mocha: crust, mantle, and base provide near-black neutral backgrounds; text and subtext provide high-contrast lettering; brighter colors are reserved for state and resource accents.
- the intro remains barebones and logo-led: an outlined planet, one tilted orbit, and a single moon sit above compact transparent play and quit controls with clear borders.
- selecting an owned body should produce a compact map-attached foldout with current stats and direct shortcut actions.
- dense management controls belong in small independent windows so the player can arrange and compare overview, logistics, and procurement information without entering one monolithic modal.
- index windows such as owned bodies should remain sparse launchers, showing only enough information to choose what to inspect next.

## economy

the economy should feel alive: populated bodies create demand, local stockpiles influence prices, and civilian ships respond to profitable mining and trade opportunities.

decided direction:

- populated celestial bodies generate local demand from population, body character, and infrastructure.
- surface and orbital stockpiles have finite capacity supplied by infrastructure.
- construction creates bounded, staged procurement demand rather than requesting its complete lifetime material cost at once.
- local purchase prices respond to shortage, procurement limits, free storage, dock throughput, and available buyer funds.
- civilian mining ships prefer opportunities with positive expected profit per cycle time and sell through credit-conserving delivery transactions.
- infrastructure has fixed credit maintenance, with arrears suspending effects without destroying stored goods.
- fuel cells are the standard in-system fuel and are produced by refining raw resources.
- the active v1 raw resources are metals, organics, crystals, and volatiles; the wider catalogue is reserved for later progression.
- metals and crystals refine into universal construction material, which must be present at the build location as work advances.

see [storage and procurement economy](design/storage-procurement-economy.md) for the detailed model.

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
