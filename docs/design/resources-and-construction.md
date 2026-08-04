# resources and construction

this doc records the agreed mechanics-first v1 resource and construction loop. it describes the foundation in code and keeps unselected transport parameters out of the design.

## maturity legend

- decided v1: implemented direction or direction ready for the next small implementation slice.
- candidate: likely direction, but still needs a concrete design decision.
- later: explicitly outside the first playable progression.

## decided v1

anchor entities:

- stars, planets, and moons are the core anchor entities.
- solid bodies, including planets and moons, support surface and orbit construction layers.
- gas giants support upper-atmosphere and orbit construction layers.
- stars support orbit construction only.

resources and refining:

- the active raw resources are metals, organics, crystals, and volatiles.
- the broader raw-resource catalogue can remain in code for later expansion, but generated v1 bodies expose only the active resources.
- volatiles plus metals produce fuel cells. the existing fuel-cell recipe and its role as standard in-system fuel remain active.
- metals plus crystals produce one universal construction material.
- organics produce food.
- construction factories currently refine construction material as the smallest extension of the existing infrastructure model.

local construction:

- a body's primary stockpile is on its surface for a solid body and in its upper atmosphere for a gas giant. orbit is a separate stockpile.
- every infrastructure type resolves to a construction layer for its anchor environment.
- construction material is consumed continuously from that exact layer as work advances. material in another layer cannot advance the project.
- queueing a project does not reserve or teleport material. a project without enough local material remains queued and stalled.
- initial sol provides a small stock of construction material already delivered to earth orbit so the existing orbital build flow remains usable before transport commands exist.

construction capacity:

- biological population provides construction throughput at a simple initial conversion rate.
- robots can add explicit robotic construction throughput, including on bodies without biological population.
- capacity and local material jointly limit progress. construction factories are refiners rather than the source of construction capacity.
- this is a tested foundation, not a complete workforce, jobs, automation, or robot-production system.

## candidate

- interlayer transport must be explicit and physics-informed. gravity, atmosphere, direction of travel, infrastructure, payload, and fuel are relevant inputs.
- the first transport implementation still needs decisions for throughput units, fuel costs, eligible infrastructure, and command ownership.
- infrastructure footprint limits and construction throughput are separate concepts. body size may constrain the former while population and robots provide the latter.

## later

- planetary belts and system belts can become anchor-entity types after the core body model is stable.
- the kuiper belt and oort cloud are later regional or system constructs, not v1 anchor bodies.
- detailed workforce allocation, robot manufacturing, construction jobs, and project priorities.
