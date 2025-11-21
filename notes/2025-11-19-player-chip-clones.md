# Player Chip Clones

Allow the player to control multiple player Chips simultaneously by sharing their inputs across several entities.
This requires moving the player control logic out of the single Chip entity and into the PlayerState so multiple Chips can respond to the same commands.
Each Chip should be independently affected by terrain and hazards, creating interesting puzzles that require coordinating several Chips at once.

Player inventory and stats are shared between all player Chips, so collecting keys or items with one Chip benefits all controlled Chips.
Game over when the master player Chip is killed, game win when any player Chip reaches the exit.
Player chips are solid and cannot pass through each other, implement using block push logic.
Camera should focus on the 'master' player Chip, to keep the camera logic simple.

Monsters like Teeth should focus on the nearest player Chip, TBD what happens if multiple Chips are equidistant.

[x] Update player chasing logic to target the nearest player Chip.\
[x] Move forced_move flag to player entity flags.\
[x] Refactor game logic to support multiple player Chips.\
[ ] Introduce the master player Chip and adjust win/lose conditions.\
[ ] Test with levels designed for multiple player Chips.\
