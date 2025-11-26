# Explicit gameplay phases

In some levels, the order in which monsters spawn affects gameplay significantly.
I would prefer this not to be the case, but fixing this is a slow and difficult process.

Implement gameplay in three distinct phases:

1. Process all entity movement
2. Process all entity actions (killing player, pickup items)
3. Process all terrain effects (traps, buttons; water, fire hazards for monsters)

[ ] Refactor game loop to have explicit phases for movement, actions, terrain effects.\
[ ] Split physics code into separate movement and terrain effect functions.\

[ ] Fix broken replays after this change.\

This doesn't resolve all issues, but it should help a lot.
In particular, movement order and collisions can still interact in an order-dependent way, so also:

[ ] Visualize monster order in the editor, with a little number next to each monster indicating its spawn order.\
[ ] Allow reordering monsters in the editor by dragging these numbers up and down.\
