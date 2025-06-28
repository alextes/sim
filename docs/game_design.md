# game design

lay out game mechanics and features. used to ideate and plan with assistant.

## vision

sim is a top-down 2d space empire builder. the player begins in the sol system and shares the galaxy with one ai-controlled empire. victory is achieved either by military conquest or by reaching a target prestige score first.

## celestial bodies

### planets

planets come in greenhouse, barren, volcanic, oceanic and other classes. each has a size grade (tiny, small, medium, large, giant) that defines the number of building slots. every planet is assigned three resources from the global pool (see economy) plus a yield grade (poor, average, rich, ultra). building slot counts and resource yields together shape planetary specialization.

#### gas giants

gas giants mostly supply volatile and exotic gases and hold more slots for orbital infrastructure.

### moons

moons are small bodies orbiting planets. they have fewer slots but cheaper build costs.

### asteroids & belts

asteroids appear in belts and clouds. most are low-yield but cheap to exploit.

## colonization

planets and moons can be settled with colony ships or conquered from rivals. players may also build orbital structures (shipyards, habitats, defense stations, solar collectors) at lagrange points and around gas giants. each structure provides unique bonuses and costs upkeep.

## economy v1 – three-resource mercantilism

1. resource spread  
   • each celestial body holds exactly three resource types pulled from the pool (metals, volatiles, organics, rare exotics, crystals, microbes, isotopes, dark matter, etc.) and a yield grade.
   • sol worlds skew toward metals + volatiles; the rival empire owns more organics + exotics, ensuring asymmetrical demand.
   • monthly extraction = population × infrastructure × yield grade.

2. planetary supply & demand  
   • supply index = stock / (population consumption × 3). if ≥1 the colony is self-sufficient; if <1 it becomes an importer.  
   • demand index = 1 / supply index (capped). higher values raise local prices. prolonged shortages reduce happiness and production.

3. galactic exchange rate  
   • every resource has a base credit value by rarity tier (common, uncommon, scarce, exotic).  
   • exchange modifier = (buyer demand – seller supply) ×0.3  
   • final price = base × (1 + modifier) clamped to 0.5-2.5× base. prices recalculate daily.

4. trade ships  
   • cargo hull sizes: 200, 500, 1000 t. bigger hulls need stronger drives and burn more fuel.  
   • loading cost = quantity × local price + 5 % port fee (paid upfront).  
   • routes may use rival transit lanes (safe but taxed) or open space (free but pirate-prone).  
   • selling auto-auctions cargo on arrival (2 % harbour commission). profit = sale – purchase – transit costs – optional insurance.

5. credit flow  
   • imperial treasury income:  
    – tariffs (4 % of cross-border sales, adjustable by policy)  
    – corporate tax (15 % of private profit)  
    – state-owned trade fleets (100 % profit)  
   • treasury expenses: construction, research grants, military upkeep, subsidies.

6. strategic levers  
   • broker licences: allow placing pre-orders, reducing price risk.  
   • trade treaties: lower customs fees both ways.  
   • subsidies: pay a monthly stipend to guarantee minimum imports.  
   • embargo: block rival access to a resource.  
   • convoy escorts: assign warships to routes to reduce pirate raids.

7. optional complications  
   • price bubbles, black-market goods, dynamic tariffs, pirate king events.

## research

knowledge points (kp) are produced by laboratory districts:  
kp/day = labs × researchers × efficiency modifiers.

research tree pillars  
– industry (mining & refining)  
– logistics (drives, cargo-handling, sensors)  
– commerce (markets, tariffs, finance)

queue up to three techs: first slot gets 100 % kp, second 50 %, third 25 %.

speed-ups  
• research grants (+25 % kp, costs credits)  
• breakthroughs (random, instantly add 20 % progress)  
• captured data cores (skip prerequisites)

sample unlocks  
– improved drills: +20 % extraction yield  
– automated cranes: –30 % loading time  
– futures contracts: buy resources on margin  
– sensor sweeps: +10 % pirate detection  
– tariff harmonization: +2 % treasury income  
– asteroid smelters: upgrade common ore → rare alloy

## military

### units

frames: frigate, destroyer, cruiser, carrier, fighter, bomber, explorer, colony ship

### civilian ships

the civilian economy operates its own fleets for various purposes, independent of direct player control. these ships are built and managed autonomously by planetary civilian sectors.

- **mining ships**: extract raw resources from planets, moons, and asteroids. they form the backbone of the civilian industrial base.
- **trade ships (freighters)**: transport goods and resources between star systems, creating a galactic marketplace. they respond to supply and demand, seeking profitable trade routes.
- **migration transports**: move populations between worlds, seeking better opportunities or fleeing poor conditions. this is a key driver of demographic shifts in the galaxy.
- **tourism vessels**: generate income by ferrying tourists to and from planets with high appeal (e.g., unique features, high amenities).

### ship design

hull size, hardpoints, internal slots, power & heat budgets; player can clone templates or design new hulls. ai uses default templates.

### weapons

kinetic, laser, plasma, missile, ion, particle, wave, gravitic (placeholder icons: -, ---, o, >, +, \*, ), >)

## Propulsion

the vast distances between star systems necessitate two distinct modes of travel, each with its own engine and fuel type.

### in-system drive (sub-light)

the standard engine for maneuvering within a star system.

- **function**: used for travel between planets, moons, and stations within a single star system. it is also the only drive capable of engaging in combat and performing the final "last-mile" approach to celestial bodies and structures.
- **fuel**: consumes standard `fuel cells`, which are readily produced from common resources.
- **velocity**: provides constant thrust, allowing for acceleration and deceleration. top speed is limited, making interstellar travel impractical.

### interstellar drive (warp drive)

a specialized engine for traversing the void between stars.

- **function**: enables near-instantaneous jumps between star systems. the drive requires time to charge before a jump and has a cooldown period afterward.
- **fuel**: consumes `warp cores`, a rare and expensive fuel type synthesized from exotic resources.
- **usage within a system**: the warp drive can be used for short-range jumps within a system. however, safety protocols prevent activating the drive too close to significant gravity wells. this means ships cannot warp directly to a planet, starport, or star. a certain "safe distance" must be maintained, after which the ship must use its in-system drive for the final approach. this makes it practical for quickly crossing a system, but not for precise maneuvering.

## population

population is the lifeblood of an empire, providing the workforce for industry and research, and the soldiers for its armies. managing population growth, happiness, and migration is key to success.

### population growth

population growth is primarily biological, though future advances may unlock cybernetic or synthetic populations with different characteristics.

natural growth is influenced by several factors:

- **base growth rate**: each species has a natural growth rate. for humans, this is a slow and steady increase.
- **planet quality**: the habitability of a planet affects growth. factors include gravity, atmosphere, and temperature. terraforming can improve these conditions.
- **food availability**: a surplus of food boosts growth rates, while shortages can halt it or even cause population decline.
- **amenities**: buildings like hospitals, entertainment complexes, and parks increase happiness and, in turn, population growth. these are unlocked through research.
- **policies**: imperial policies can encourage or discourage population growth.

for initial implementation, a simple constant annual growth rate is applied to all populated worlds.

### migration

pops will automatically migrate between worlds within an empire to seek better living conditions. the primary drivers for migration are:

- **job availability**: pops will move to planets with open jobs that match their skills.
- **housing**: a lack of housing on a world will push pops to emigrate.
- **planet quality & amenities**: higher quality worlds with better amenities will attract migrants.
- **imperial policies**: migration can be encouraged or restricted through policies.

migration will be a future mechanic and is not in the initial implementation.

### military interaction

population is directly affected by military actions.

- **bombardment**: orbital bombardment by enemy fleets will reduce a planet's population and destroy infrastructure. it also reduces a planet's defense capabilities, making it easier to invade.
- **armies**: planets can recruit and garrison ground armies. these armies are used for planetary defense against invasions and for conquering enemy worlds. army strength is drawn from the local population.

this mechanic is planned for a later stage of development.

## strategic scaling & fleets

ships scale visually with zoom. tactical view shows individual weapons; strategic view bundles ships into fleets. unassigned ships orbit their origin body; fleets take selection priority on zoomed-out layers.

## building

shipyards build spaceframes; other structures increase mining, research, trade capacity, or defense.

## the list

- events
  - random crises and opportunities
  - difficulty adapts to player choices and power balance
- strategic scaling when zooming
- gas resources in gas giants
- many resource types with varied rarity
- complex civilian economy with incentives and subsidies
- lagrange-point structures
- float vs grid discussion for movement precision
- population growth, migration, and military interaction mechanics
