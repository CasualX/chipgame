# Checkpoints in chipplay

Add a runtime checkpoint system for chipplay that makes long or difficult levels less punishing without modifying base level files.
During a run, the player should be able to create checkpoints. These checkpoints live only while playing and can be resumed at any time during play.

Resuming from a checkpoint must mark the run as checkpoint-assisted so progression rules stay fair.
Players should be allowed to finish the level, but record and trophy updates should be disabled for runs that used checkpoints.

Checkpoints should be easy to manage during play, with clear UI for creating and resuming.
The system should feel like a quality-of-life improvement that encourages experimentation without trivializing the challenge of completing levels.

[x] Initial checkpoint save and load system implemented.\
[x] Disable records and trophies for checkpoint-assisted runs.\
[ ] Improve the UX when a level is completed with checkpoints used.\
