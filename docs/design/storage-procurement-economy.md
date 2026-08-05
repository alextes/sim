# storage and procurement economy

this doc defines how finite storage, construction demand, procurement prices, deliveries, infrastructure maintenance, and civilian investment form one economic loop. it is the canonical design for these mechanics.

## maturity legend

- decided v1: direction ready to implement.
- candidate: likely follow-up after the v1 loop is working.
- maybe later: intentionally outside the current implementation plan.
- balance question: tune through deterministic scenarios rather than redesigning the model.

## goals

- make resource storage a constructed and maintained capability rather than an unlimited property of a body.
- make large projects generate sustained logistics activity without demanding all lifetime materials immediately.
- let prices and physical capacity create backpressure on civilian mining and transport.
- let the player shape automatic procurement without manually scheduling every shipment.
- conserve credits when resources are traded.
- allow shortages, dock congestion, and temporarily stranded ships to emerge from understandable rules.

## economic loop

```text
recurring consumption and active construction
                    |
                    v
       bounded procurement opportunities
                    |
                    v
       civilian route and investment choices
                    |
                    v
 mining, transport, docking, and partial unloading
                    |
                    v
      replenished stock and resumed production
```

## decided v1

### layer-aware stockpiles

- every stockpile belongs to an anchor body and a logistics layer.
- solid bodies use surface and orbit, gas giants use upper atmosphere and orbit, and stars support orbit only.
- resources in a body's primary layer are not automatically available in orbit, and orbital resources are not automatically available in the primary layer.
- each layer uses shared storage capacity. one resource unit consumes one capacity unit in v1.
- resource-specific volume, refrigeration, hazardous handling, and spoilage are not part of v1.
- all deposits are bounded. a producer or delivery keeps or reports any amount that cannot fit instead of silently destroying it.

### storage and dock infrastructure

- ground warehouses provide surface capacity.
- upper-atmosphere storage provides primary capacity for gas giants.
- orbital depots provide orbital capacity.
- orbital docks provide unloading throughput and berth capacity.
- storage capacity and dock throughput are independent: a body may have room but unload slowly, or unload quickly but have nowhere to keep cargo.
- storage and docks use ordinary infrastructure capacity, construction materials, and maintenance credits.
- settled starting bodies receive enough explicit or settlement-core capacity to hold their seeded inventory and avoid a bootstrapping deadlock.

### recurring consumption and procurement

- recurring consumption is a rate describing what population or infrastructure uses over time. it is not itself a purchase order.
- procurement policy describes what a buyer is willing to acquire.
- each resource policy supports an enabled state, reserve target, maximum unit price, and optional periodic spend cap.
- construction contributes automatic procurement demand for a bounded staging horizon.
- the player can override procurement limits, but does not need to create each construction order manually.

the wanted quantity is bounded by:

- the difference between target and available stock;
- free capacity in the destination layer;
- remaining dock throughput when delivery occurs;
- the buyer's remaining spend cap;
- the buyer's current credits.

no purchase opportunity exists when the target is filled, storage is full, procurement is disabled, or the buyer cannot pay for any unit.

### staged construction

- queueing a project records its lifetime cost but does not deduct every material immediately.
- only a bounded horizon of upcoming construction work contributes to the current procurement target.
- construction consumes universal construction material continuously from the exact layer where work advances.
- if local construction material is unavailable, construction remains queued and pauses without losing progress.
- ongoing consumption frees storage and allows fresh procurement demand to appear.
- the UI distinguishes lifetime project cost from construction material needed for the current staging horizon.

### price formation

- a purchase price is derived from current shortage rather than stored as an independent market truth.
- price rises monotonically as stock falls farther below target.
- price remains capped by the buyer's configured maximum.
- clamps prevent extreme values while balance is immature.
- price and wanted quantity are recalculated from current state when an opportunity is inspected or a delivery arrives.

exact curve constants, reserve defaults, and the staging horizon are balance questions.

### economic accounts

- government procurement on player-controlled bodies uses the global player treasury.
- each populated body has a civilian economy account used for civilian income and investment.
- delivery income is credited to the ship's home civilian economy, not to its current sell destination.
- a trade must debit one account and credit another by the same amount; delivery never mints credits.
- this account boundary can later expand to state treasuries for other empires without changing delivery semantics.

### transactional deliveries

when a ship attempts to unload, the accepted quantity is the minimum allowed by:

- cargo aboard;
- current wanted quantity;
- free destination storage;
- remaining dock throughput;
- affordable quantity at the current price.

the transaction debits the buyer, credits the seller's civilian economy, deposits accepted cargo, and leaves the remainder aboard. ships process in deterministic order when several arrive during the same interval.

home base and sell destination are separate concepts. a ship may wait to unload when a buyer remains interested but current throughput is exhausted. v1 may leave an oversupplied ship waiting rather than automatically finding a new buyer.

### civilian route selection

civilian mining ships compare visible procurement opportunities by expected profit per complete cycle:

```text
expected sale revenue
- travel operating cost
- mining operating cost
- ship maintenance
- docking and handling fees
= expected cycle profit
```

ships prefer positive expected profit per unit of cycle time. estimates include wanted quantity, cargo capacity, mining yield, distance, speed, and expected mining time. manual routes remain authoritative, and equal scores use deterministic entity and resource ordering.

### civilian investment

a civilian economy considers commissioning a mining ship only when:

- it has enough credits and construction materials;
- a usable shipyard exists;
- no equivalent build is already pending;
- the best visible route meets a configurable payback threshold;
- an investment cooldown has elapsed.

delivery earnings increase civilian savings and can fund later ships. the economy does not build a ship merely because its cash balance crosses a fixed threshold.

### infrastructure maintenance

- every completed infrastructure unit has a fixed periodic credit cost.
- storage maintenance is charged whether the capacity is empty or occupied, making very large stockpiles expensive to keep available.
- unpaid maintenance becomes arrears.
- delinquent production and construction infrastructure suspends its effect.
- delinquent storage remains physically present and preserves existing goods, but accepts no new deposits.
- maintenance failure never deletes inventory.

### player information and control

the planet overview should expose:

- primary-layer and orbital stock, capacity, and free space;
- reserve target, current wanted quantity, current purchase price, maximum price, and spend cap;
- dock throughput and ships waiting to unload;
- immediate construction material blockage and lifetime project cost;
- infrastructure upkeep, arrears, and active state.

## candidate follow-ups

- delivery reservations that reserve demand, buyer funds, storage space, and unloading allowance for an accepted trip.
- automatic rerouting when an arriving ship cannot make a worthwhile sale.
- civilian freight routes that buy at one body and sell at another.
- physical fuel-cell consumption instead of a credit operating-cost estimate.
- surface-orbit transfer orders constrained by spaceports, launch capacity, elevators, or mass drivers.
- maintenance inputs such as replacement materials and workforce.

## maybe later

- a full order book with independent bids and asks.
- delivery contracts with expiry, renewal, cancellation penalties, and reliability history.
- futures and long-term supply agreements.
- specialized solid, liquid, cryogenic, hazardous, and perishable storage.
- insurance, cargo loss, accidents, weather, and launch windows.
- ownership and balance sheets for individual civilian firms.

## balance questions

- how many economy intervals should one reserve target or construction staging horizon cover?
- how steeply should price rise as stock approaches zero?
- what maintenance rate makes resilience valuable without making spare capacity punishing?
- how strongly should expected dock waiting time reduce a route's estimated return?
- what payback period should trigger new civilian investment?

these values should be tuned with deterministic multi-tick scenarios after the complete loop exists.
