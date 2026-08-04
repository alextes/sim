# pirate factions

this doc describes the intended identity and gameplay loop for pirate factions. the direction is concrete enough to preserve, but it remains in the design phase and is not yet an implementation plan.

## maturity legend

- decided direction: part of the intended pirate identity.
- candidate mechanic: a promising way to realize that identity, but still subject to design work.
- maybe later: interesting extension outside the first useful version.
- open question: decision needed before implementation.

## design goals

- make lightly protected economic activity create an opportunity rather than trigger a random event.
- make pirates an asymmetric threat instead of a smaller territorial empire.
- let players reduce piracy through escorts, patrols, surveillance, and investigation as well as direct combat.
- connect pirate decisions to the same economic logic as civilian activity, extended with risk, incomplete information, and escape probability.
- make pirate losses, bases, recruitment, and operating range persistent enough that player action has lasting consequences.

## decided direction

identity:

- pirate factions do not own planets, govern conventional territory, or pursue galactic conquest.
- each pirate faction is a network of hidden bases, members, ships, and known opportunities.
- a pirate base can occupy a concealed orbital position around a gas giant or hide on or around a remote moon without granting ownership of the body or system.
- pirate population represents active membership and available manpower rather than a normal civilian population. it changes through recruitment, departure, defection, and casualties rather than planetary population growth.
- pirate ships use distinct designs that favor speed, concealment, cargo capture, and escape over winning a stand-up fleet battle.

raiding:

- pirates look for valuable civilian traffic on routes that appear weakly protected.
- escorts protect the ships they accompany, while patrols and recent military presence raise the perceived danger of operating on a route.
- that protection should become less certain as time passes without a visible military presence, allowing neglected routes to become attractive again.
- a pirate attacks only when its estimate of the reward, chance of success, chance of escape, and cost of losing ships and crew makes the opportunity worthwhile.
- fast in-system engines and fast warp drives can make escape a viable strategy. later player technologies such as warp interdiction can change that calculation.
- pirate knowledge is imperfect. pirates act on observations and remembered activity, not authoritative access to the player's fleet positions or response plan.

persistence:

- pirate ships that appear on a route should ultimately originate from a base, travel within an operating range, and return somewhere to repair, refuel, unload, or hide.
- destroyed ships and exposed bases are real losses. replacements require time, resources, recruitment, or captured material.
- this persistent layer can remain coarsely simulated when pirates are outside player observation; it does not require every hidden movement to be rendered.

## candidate mechanics

opportunity evaluation:

- estimate expected loot from observed cargo, ship vulnerability, and the pirate's ability to capture or disable it.
- estimate interception risk from escorts, patrol recency, nearby bases, known military response times, sensors, and route traffic.
- estimate escape probability from relative speed, warp readiness, local gravity wells, damage risk, and available interdiction technology.
- compare the resulting risk-adjusted return with fuel, ammunition, repair, crew, and opportunity costs.
- allow pirate factions to have different risk tolerance, preferred prey, and willingness to accept casualties.

route security:

- track a legible security or danger estimate for important trade lanes rather than treating a single patrol as permanent protection.
- let escorts create strong local protection and patrols create broader protection that decays with time and distance.
- let pirate observation lag behind reality so a sudden convoy, hidden response fleet, or bait ship can surprise them.
- surface enough information for the player to understand why a route is attracting attacks without exposing the pirate's exact calculation.

base discovery:

- an undiscovered base is not targetable and may be represented only through clues such as raid patterns, sightings, emissions, or traffic returning toward a region.
- a ship with sufficient observational capability can search a plausible location and accumulate discovery progress over time.
- discovery time depends on sensor strength, concealment, range, local terrain, and whether the search can remain in place uninterrupted.
- partial intelligence can narrow the search area before the exact base is revealed.
- once discovered, a base can be attacked, blockaded, monitored, or used to trace other pirate activity.

suppression and recovery:

- destroying or blockading a base lowers local pirate reach and replacement capacity instead of instantly removing piracy everywhere.
- surviving pirates may disperse, defect, join another network, or establish a replacement base after a long recovery period.
- recruitment can respond to local poverty, instability, pirate success, faction policy, and the perceived safety of pirate life.

## maybe later

- multiple pirate factions that compete, cooperate, merge, or sell information about one another.
- ransom, smuggling, protection payments, privateering, and state sponsorship.
- boarding, stolen ship conversion, prisoner exchange, and pirate reputation.
- pirate havens inside nominally controlled territory when local authorities tolerate or support them.
- diplomacy with pirate factions, including temporary truces or paid passage, without turning them into ordinary empires.

## open questions

- how many pirate factions should exist, and can new ones form dynamically?
- what resources do bases consume, and how do pirates obtain replacement ships and modules?
- does a raid steal cargo, destroy the target, capture the ship, demand ransom, or choose among those outcomes?
- how should patrol coverage and its decay be represented to the player?
- does discovery progress persist, decay, or convert into regional clues when the observing ship leaves?
- can a pirate faction be permanently eradicated, or should the underlying conditions eventually create new piracy?
- how much hidden pirate simulation is necessary before coarse strategic accounting becomes indistinguishable to the player?
