# Player Chip Clones

Allow the player to control multiple player Chips simultaneously by sharing their inputs across several entities.
This requires moving the player control logic out of the single Chip entity and into the PlayerState so multiple Chips can respond to the same commands.
Each Chip should be independently affected by terrain and hazards, creating interesting puzzles that require coordinating several Chips at once.

[ ] Update player chasing logic to target the nearest player Chip.\
[ ] Move forced_move flag to player entity flags.\
[ ] Refactor player control logic to support multiple player Chips.\
[ ] Update input handling to broadcast commands to all player Chips.\
[ ] Test with levels designed for multiple player Chips.\
