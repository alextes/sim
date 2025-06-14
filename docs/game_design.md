# game design

lay out game mechanics and features. used to ideate and plan with assistant.

## high level

sim is a top down 2d game. the goal is explore, expand,

## celestial bodies

### planets

have different types, greenhouse, barren, volcanic, etc. different size. different resources.
some buildings are slotted, a planet has a limited number of slots based on size.
other buildings are of a certain type and may increase mining output, or other resources. they specialize the planet as it were.
different types of planets and geographic features mean some planets are better for certain types of buildings.

#### gas giants

gas giants are a type of planet. they often offer a different type of resource, and are often larger.

### moons

moons are smaller celestial bodies that orbit planets. they have a limited number of slots for buildings.

### asteroids

asteroids are small celestial bodies. they appear mostly in clouds, where only few are fit for mining. clouds are inserted at sound physical layers, like just beyond the last large planetary body (kuiper belt) or at the gravitational rim of a solar system (oort cloud). they are also inserted in the gaps between planets, where they are not fit for mining.

## building

some structures enable building units. mainly shipyards.

## units

- frigate, a small agile ship with a small weapon.
- destroyer, a larger ship with a larger weapon.
- cruiser, a larger ship with a larger weapon.
- explorer, a small ship with a small weapon.
- colony ship, a large ship with a large weapon.
- carrier, a large ship with a large weapon and the ability to launch fighters.
- fighter, a small ship with a small weapon.
- bomber, a large ship with a large weapon.

## ships

ships follow a similar format to planets. they have some size and slot types which constrains what you can build on them. the game comes with decent default designs, but allows the player to create their own.

## weapons

- kinetic -
- laser ---
- plasma o
- missile >
- ion +
- high-energy particle \*
- wave )
- gravitic >

## strategic scaling

we want to give a feeling of controlling a vast empire with many ships, in many fleets. for this we both want to enable the player to zoom in all the way and see the individual ships in battle, firing individual weapons, and allow the player to zoom out and control the fleets. one should effortless be able to zoom in and select a single civilian ship for escort, or select three ships in a fleet for repairs, or set a fleet to go refuel at a station.

### fleets

when ships are built they are not yet assigned to a fleet, although the player may configure a fleet template and request ships be built from the nearest shipyard. unassigned ships simply orbit their shipyard host body. when zooming out, fleets take priority over individual ships. i.e. when a single tile contains both fleets and individual ships, the fleets will be visible / selected when clicking the tile. when drawing a box, all ships in the box will be selected. fleet ships grouped by fleet, the rest in an ungrouped list.

## economy

the player does not control everything. some industry capacity or specialization can only be incentivized by the player but it is the civilian economy that has to pick up the incentive. the player can order shipyards built, but if a shipyard lacks a certain resource, the player can either build military trade / transport fleets and manage them themselves, or the player can set subsidies for the civilian economy to build these fleets, or even pick up refining locally.

technological progress impacts the civilian economy but only develops slowly when left alone. most progress needs to be incentivized by the player.

## the list

- events
  - throw random challenges and opportunities at the player
  - challenge the player by making things easier or harder depending on their choices
  - challenge the player by making things easier or harder depending on their relative position to other players
- scale. on normal zoom level where both celestial bodies and ships are visible, ships would be too tiny to individually select. we use strategic scaling to make ships visible. a player can zoom in all the way to see individual ships.
- gas resources mostly present in gas giants.
- many different types of resources, some are rare, some are common, depending on the celestial body and what type it is.
- complex civilian economy. i.e. the player does not control everything. some industry capacity or specialization can only be incentivized by the player but it is the civilian economy that has to pick up the incentive. the player can order shipyards built, but if a shipyard lacks a certain resource, the player can either build military trade / transport fleets and manage them themselves, or the player can set subsidies for the civilian economy to build these fleets, or even pick up refining locally.
- lagrange point special orbital structures.
