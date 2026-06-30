# surface orbit logistics

this doc organizes the logistics ideas that sit between planets, orbital infrastructure, and ships. most of this is not ready for implementation yet, but the concepts should shape planetary development design.

## maturity legend

- decided v1: direction ready to design or implement soon.
- candidate: likely direction, but still needs design work.
- maybe later: interesting idea, intentionally out of scope for the next slice.
- open question: decision needed before implementation.

## decided v1

surface and orbit:

- resources available on a planet surface are not automatically available in orbit.
- resources delivered to an orbital station are not automatically available on the planet surface.
- a future logistics model should distinguish surface stock availability from orbital stock availability.

launch capacity:

- planets need some notion of capacity to move material from surface to orbit.
- early design can treat launch capacity as throughput rather than simulating individual launches.
- launch capacity should affect whether ground-produced resources can feed orbital construction and shipbuilding.

shipyard access:

- a planet overview should be able to open the associated shipyard when the body has a usable shipyard.
- shipyards should eventually have a clear location: ground, orbit, or a separate station.
- command processing should remain authoritative for whether a build can actually happen.

## candidate

spaceports:

- a ground spaceport makes landing and launch cheaper and more efficient.
- an orbital spaceport makes orbital transfer, docking, and shipbuilding easier.
- a body can have neither, one, or both, which changes the economics of moving resources and ships.

space elevators:

- a space elevator provides high-throughput surface-to-orbit transfer.
- elevator capacity should gate how quickly surface stocks become available in orbit and vice versa.
- elevator value depends on gravity, atmosphere, and the scale of orbital industry.

mass drivers:

- mass drivers are mining logistics infrastructure for large bodies where launching bulk raw material is more efficient than ship transport.
- they should be especially attractive for raw resources and less useful for fragile goods, population, or finished ships.
- mass drivers are likely ground infrastructure with orbital receiving requirements.

landing economics:

- gravity and atmosphere should influence landing and launch fuel costs.
- landing without a spaceport should be expensive.
- ships may eventually need special modules to land on bodies without proper ports.

## maybe later

- individual launch vehicles and scheduled surface-orbit cargo runs.
- ship components for aerobraking, heat shields, landing legs, shuttles, and cargo pods.
- accidents, launch windows, weather, and atmospheric hazards.
- blockades that separate surface and orbit economies during conflict.
- separate civilian logistics contracts for surface-orbit movement.

## open questions

- should surface and orbit stocks be separate maps, separate entities, or a stock map keyed by location layer?
- should early launch capacity be measured per day, per month, or per simulation tick?
- should ground shipyards be allowed to build ships directly, or should all ship construction happen in orbit?
- should mass drivers move only raw resources, or any nonliving cargo?
- how much of the logistics model should be visible in the first planet overview?
