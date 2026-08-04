# minor factions

this doc preserves the early direction for city-state-like independent factions. the concept remains intentionally open: these factions should matter to the simulation without all sharing one economic role or behaving like reduced versions of major factions.

## maturity legend

- decided direction: part of the intended minor faction identity.
- candidate role: a promising capability that some minor factions may have.
- maybe later: interesting extension outside the first useful version.
- open question: decision needed before implementation.

## design goals

- add independent powers whose objectives are not galactic conquest or automatic hostility toward the player.
- give them enough agency, scarcity, and relationships to affect strategic choices rather than exist only as shops or quest dispensers.
- let specialized local capabilities create trade, diplomatic, technological, and logistical opportunities.
- make their influence emerge through the same physical economy as other factions, including stocks, prices, storage, ships, and transport capacity.
- keep their scope bounded enough that they remain distinct from major factions.

## decided direction

identity:

- major factions are the player-equivalent competitors that expand, colonize, and pursue galaxy-scale power.
- minor factions are sovereign or semi-sovereign actors with limited holdings and no default drive for continuous territorial expansion.
- minor does not mean passive or irrelevant. a minor faction has interests, relationships, assets, red lines, and some ability to respond when those are threatened.
- minor factions are not inherently pirates and should not all begin hostile to the player. cooperation, competition, neutrality, and hostility can emerge from incompatible interests and diplomacy.
- different minor factions can specialize in different roles; no single capability below needs to be universal.

bounded power:

- a minor faction may control a planet, moon, station, habitat, local cluster, or specialized mobile operation.
- its economic and military reach is limited by population, infrastructure, storage, transport, technology, and diplomatic access.
- it may develop its existing holdings and replace losses without necessarily colonizing new systems or becoming a major faction.
- its continued independence should matter. conquest, protection, isolation, or economic dependence can change the surrounding balance of power.

## candidate roles

specialized technology:

- a minor faction may possess a distinctive technology, research tradition, data archive, or manufacturing technique.
- access could take the form of technology trade, licenses, research cooperation, captured knowledge, or products that embed the capability without transferring it.
- willingness to share should depend on relationships, strategic risk, price, and what rival factions may learn as a result.

resource producer:

- a mining or industrial minor faction may produce resources that nearby major factions cannot obtain as cheaply or reliably.
- production should use real deposits, infrastructure, labor, and stocks rather than generate unlimited offers.
- trade volume is constrained by extraction, storage, local consumption, transport, and route safety.

independent trade hub:

- a minor faction with diplomatic access to otherwise hostile powers can act as an intermediary between markets that cannot trade directly.
- for example, it may buy goods from a major faction hostile to the player, hold them in its own storage, and resell a limited quantity to the player.
- this is a physical chain rather than an abstract bypass: the intermediary needs relationships with both sides, available capital, storage space, transport capacity, and safe routes.
- prices should include the intermediary's margin, delay, risk, scarcity, and the diplomatic cost of continuing the trade.
- either major faction may pressure, embargo, blockade, subsidize, or threaten the intermediary, making the hub a source of diplomacy and conflict rather than guaranteed market access.

service provider:

- a minor faction may sell repair, refueling, transport, intelligence, navigation, banking, mercenary, or scientific services.
- service quality and capacity should come from actual assets and expertise that can be disrupted, improved, monopolized, or lost.

diplomatic connector:

- a minor faction can carry messages, broker agreements, exchange prisoners, or provide a neutral meeting place when direct relations are poor.
- acting as an intermediary can strain its relationships or neutrality, so access should never be automatic or consequence-free.

## candidate behavior

- minor factions pursue a small set of motives such as independence, prosperity, security, religious duty, scientific discovery, or preservation of a location or way of life.
- decisions should follow those motives and their imperfect knowledge rather than a universal expansion score.
- relationships with major factions can affect market access, technology sharing, basing rights, defense agreements, and willingness to mediate.
- they may ask for protection, align with a stronger power, balance competing powers, or deny access to preserve autonomy.
- finite capacity creates choices: transport used for intermediary trade cannot simultaneously serve every other market, and scarce storage or military assets cannot satisfy every partner.

## maybe later

- federations or leagues of minor factions that coordinate without becoming a major empire.
- a minor faction growing into a major faction after extraordinary economic, military, or political change.
- internal politics, coups, succession crises, and factions within a minor polity.
- cultural influence, migration attraction, pilgrimage, tourism, and unique prestige sources.
- player-created protectorates, client states, autonomous regions, or released colonies.

## open questions

- what is the minimum simulated state that makes a minor faction feel like an actor rather than a market node?
- which holdings can minor factions control in the first version: planets, moons, stations, habitats, or some subset?
- can they colonize locally, and what prevents ordinary development from turning every successful minor faction into a major one?
- can the player conquer or annex them, and what diplomatic or prestige consequences follow?
- how are diplomacy and relationships represented before a full diplomacy system exists?
- how transparent should their stocks, capacity, motives, and relationships be to the player?
- can intermediary trade continue when one endpoint explicitly forbids resale, and how are evasion and enforcement modeled?
- how many specialized roles should one minor faction be allowed to combine?
