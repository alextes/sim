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

## Galactic Economy

this model aims to create a self-regulating civilian economy where supply, demand, and profitability guide autonomous economic activity. the goal is to make the economy feel alive and create interesting strategic decisions for the player.

### 1. Planetary Demand

- each populated celestial body (planets, moons) generates demand for a set of specific resources (e.g., 3-5 types).
- the base demand for each resource is determined by the planet's population and its type/class (e.g., an industrial world demands more `metals`, an agricultural world more `organics`).
- `demand volume (per month) = base rate × population × infrastructure modifiers`
- unfulfilled demand can lead to penalties over time (e.g., reduced happiness, lower production output).

### 2. Dynamic Pricing & Profitability

- the price of a resource on a planet is determined by its local supply relative to its demand.
- a `supply/demand ratio` is calculated: `ratio = local stockpile / (monthly demand × buffermonths)`. a buffer (e.g., 3 months) prevents wild price swings.
- `local price = base resource value × (1 / ratio)`. the price is clamped to a range (e.g., 0.25x to 4.0x base value).
- as mining ships and traders sell resources to a planet, the stockpile increases, the s/d ratio goes up, and the price paid for that resource goes down. this naturally makes mining less profitable as supply saturates demand.

### 3. Civilian Production (Mining)

The civilian sector of a populated body will consider building a new mining ship if its treasury allows. the decision is based on a profitability calculation for potential mining ventures.

`profitability score = (expected revenue - expected costs)`

- **Resource Selection**: the ai scans for unexploited or under-exploited raw resources within its star system.
- **Expected Revenue**:
  - `revenue = price at home base × cargo size`
  - the ai checks the _current_ local price. a high price signals high demand and high potential profit.
- **Expected Costs**:
  - `cost = fuel cost + ship upkeep (amortized)`
  - **Fuel Cost**: calculated for a round trip. for simplicity, the travel distance is estimated as `system radius`.
    - `fuel required = distance × fuel consumption rate`
    - `fuel cost = fuel required × price of fuel at home base`
  - the ai will also consider competition by checking how many other civilian ships are already targeting the same resource body. a simple modifier could be applied: `profitability score /= (1 + number of other miners)`.
- **Decision**: if the highest `profitability score` for any available resource exceeds a certain threshold, and the treasury can afford the `mining_ship_cost`, the ai queues a new mining ship for construction.

#### Mining Ship AI Lifecycle

to create more realistic and stable behavior, individual mining ships will follow a simple lifecycle or state machine.

- **seeking contract**: a new or idle ship scans its system for the most profitable resource to mine, using the `profitability score` calculation. the "contract" is a commitment to make at least three trips.
  - if a profitable opportunity is found, the ship begins the contract.
  - if no profitable opportunities are found anywhere in the system, a ship proceeds to the `sleeping` state.
- **fulfilling contract**: the ship executes three full round trips: travel to the mining site, mine until the cargo is full, return to its home base, and sell the resources.
- **contract renewal**: after the third trip, the ship re-evaluates the profitability of its current resource.
  - if it's still profitable, it automatically renews for another three trips.
  - if the price has dropped and it's no longer profitable, the ship returns to the `seeking contract` state to find a new, more lucrative resource to mine.
- **sleeping**: if a ship cannot find any profitable contracts, it will travel to the nearest friendly spaceport (or its home base) and enter a "sleep" mode.
  - during this time, it is docked and consumes no fuel and accrues no upkeep costs.
  - after a set period (e.g., 90 days), it "wakes up" and re-enters the `seeking contract` state to check if market conditions have improved.

### 4. Civilian Distribution (Trade)

Civilian freighters are autonomous agents that respond to the price signals created by the dynamic pricing system.

- **Trade Logic**: freighters will scan for profitable trade routes by comparing prices between different planets.
- they will buy a resource where `local price` is low (i.e., `supply/demand ratio` is high) and transport it to a planet where `local price` is high (i.e., `supply/demand ratio` is low).
- **Profit Calculation**: `profit = (sale price × quantity) - (purchase price × quantity) - fuel cost - tariffs`
- this activity helps to naturally balance resource distribution throughout the empire and between empires.

### 5. Fuel Production and Consumption

- **Sub-light Fuel (`Fuel Cells`)**:
  - standard fuel for in-system travel.
  - produced at planets with `fuel cracker` buildings. these structures convert specific raw gas resources (e.g., `volatiles`) into `fuel cells`.
  - `fuel production rate = building level × local gas stockpile modifier`
  - mining ships and other civilian vessels consume fuel cells when executing move orders. if a ship runs out of fuel, it stops moving until refueled (a future mechanic could involve distress calls or fuel deliveries).
- **Interstellar Fuel (`Warp Cores`)**:
  - a distinct, more advanced fuel for ftl jumps.
  - requires more advanced infrastructure and rarer resources (e.g., `rare exotics`, `dark matter`) to produce. (this aligns with the existing design).

### 6. Mining Operations & Targets

- **Mining Ships**: the primary mobile resource extractors for the civilian economy.
- **Orbital Mining Bases**:
  - static structures built in orbit of uncolonized bodies (barren planets, asteroids).
  - more efficient extraction rate than a single mining ship and don't consume fuel, but require a significant upfront investment and have upkeep costs. they act as a mid-game way to exploit resource-rich but inhospitable worlds.
  - freighters are required to transport resources from the mining base back to a populated world.
- **Gas Mining Stations**:
  - large, floating platforms built in the upper atmosphere of gas giants by `construction ships`.
  - provide a very high-yield, continuous extraction of all available gas resources on that giant.
  - have significant construction costs and monthly upkeep.
  - once operational, they become the primary supplier for gas resources in a system. the economic model should be tuned so it is more profitable for civilian freighters to service the station than for individual mining ships to mine gas, creating a natural shift in the civilian economy's behavior.
- **Targeting Rules**:
  - mining ships will _not_ target populated planets or moons for mining. it's assumed on-planet mining is always superior, so they seek untapped resources on uninhabited bodies.

### 7. Imperial Finance & Policy

This layer represents the player's high-level interaction with the galactic economy.

- **Credit Flow (Treasury Income)**:
  - **Corporate Tax**: a percentage (e.g., 15%, adjustable by policy) of all profits generated by civilian mining and trade operations is paid to the imperial treasury.
  - **Tariffs**: a tax (e.g., 4%, adjustable) on all goods traded with other empires. this applies to both imports and exports.
  - **State-Owned Fleets**: the player can build their own freighters which generate 100% of their profit for the treasury, but require direct management.
- **Strategic Levers (Economic Policy)**:
  - **Trade Treaties**: diplomatic agreements to lower or eliminate tariffs with a specific empire, encouraging mutual trade.
  - **Subsidies**: the player can pay a monthly stipend to a planet to artificially increase the price it pays for a specific resource, stimulating imports or local mining.
  - **Embargoes**: block all trade of a specific resource, or all trade entirely, with a rival empire.
  - **Convoy Escorts**: assign military ships to protect civilian freighters on dangerous routes, reducing losses to piracy.

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

frames: frigate, destroyer, cruiser, carrier, fighter, bomber, explorer, colony ship, construction ship

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
