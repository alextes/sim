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

### ship design

hull size, hardpoints, internal slots, power & heat budgets; player can clone templates or design new hulls. ai uses default templates.

### weapons

kinetic, laser, plasma, missile, ion, particle, wave, gravitic (placeholder icons: -, ---, o, >, +, \*, ), >)

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
