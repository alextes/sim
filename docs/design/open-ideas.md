# open ideas

this doc parks ideas that are promising but not ready for implementation design. promote an idea into a focused design doc when it starts blocking real work.

## economy

- civilian freighters buy where local prices are low and sell where prices are high.
- mining ships eventually use contracts, renewals, and sleeping states instead of single-trip behavior.
- gas mining stations become the main supplier for gas giant resources once built.
- orbital mining bases exploit resource-rich uninhabited bodies without consuming ship fuel.
- imperial policy includes taxes, tariffs, subsidies, embargoes, and convoy escorts.

## energy

- energy should eventually be modeled as generation serving ongoing demand rather than as an ordinary stockpiled resource.
- limited storage can buffer short consumption spikes and temporary generation gaps, but should not bank years of surplus to cover sustained overconsumption.
- sustained generation deficits should have explicit consequences. the behavior when the short-term buffer reaches zero, including load shedding or reduced production, still needs design.

## research

- research grants, breakthroughs, and captured data cores can accelerate progress.
- research pillars start with industry, logistics, and commerce.
- unlocks can include improved drills, automated cranes, futures contracts, sensor sweeps, tariff harmonization, and asteroid smelters.

## military

- ship frames may include frigates, destroyers, cruisers, carriers, fighters, bombers, explorers, colony ships, and construction ships.
- ship design may include hull size, hardpoints, internal slots, power, heat, and template cloning.
- weapons may include kinetic, laser, plasma, missile, ion, particle, wave, and gravitic families.
- zoomed-out strategic view may group ships into fleets while tactical view shows individual weapons.

## population

- migration can respond to jobs, housing, planet quality, amenities, and imperial policy.
- bombardment can reduce population and destroy infrastructure.
- armies can defend or conquer planets, drawing strength from local population.

## events

- random crises and opportunities can change economic or military priorities.
- difficulty can adapt to player choices and the current power balance.

## code cleanup candidates

- unify the current building and infrastructure concepts before adding more infrastructure types.
- decide whether orbital infrastructure is represented as separate entities, planet-owned records, or both.
- keep deterministic ordering for all overview lists and build option lists so tests can assert ui state.
