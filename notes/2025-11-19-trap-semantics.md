# Trap semantics

This change updates trap behavior and reintroduces momentum for entities that enter traps while sliding or on force floors.
When a trap is opened, it remains open until the entity leaves the trap tile; it does not close again while the entity is still inside.

Entities that enter a trap from ice or force floors retain both their facing direction and their movement momentum while in the trap.
When released, they are forced out of the trap in the same direction they were facing when they entered, regardless of any subsequent input.

Entities that were not forced into the trap via ice or force floors do not preserve any special momentum.
When such an entity is released from the trap, it can move out in any valid direction, based on normal player input or AI behavior.

[x] Reintroduce momentum for all entities entering traps from ice or force floors.\
[x] Blocks can be moved off traps once their button is pressed once.\

It all seems to work but is hacked together, Ideally the trap code should run independently of the other terrain interaction code.
