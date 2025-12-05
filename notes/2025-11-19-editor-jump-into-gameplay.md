# Jump into gameplay from editor

Allow chipedit to start a play session for the currently open level using the same simulation and rendering pipeline as chipplay.
The editor’s level representation needs a reliable path into gameplay, so a test run behaves exactly like campaign play.
Gameplay in this mode should reuse the standard input handling and Fx/Play event pipeline instead of inventing a separate “editor” simulation.

Introduce a clean state transition between editor and a test-play sub-mode so the player can move back and forth without losing their place.
When a test run ends through death, victory, or user abort, control returns to the editor with camera, selection, and other editor state restored.
From a user perspective this should feel like a quick “Play from here” loop, not a separate application.

[x] Initial implementation of editor play mode.\
[x] Support controllers in editor play mode.\
[x] Add sound effects and music to editor play mode.\
[ ] Implement Scout and Pause functionality in editor play mode.\
