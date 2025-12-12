Backlog for vNext
=================

Unrefined ideas:

* Editor: Use icons on left of the screen to use tools
* Editor: Display X, Y grid coordinate on screen
* Editor: Add undo/redo system
* Editor: Are you sure you want to quit? dialog
* Editor: When loading a level in the editor, put camera on the center
* Editor: Custom FxState for rendering the LevelDto directly
* Intermezzo screens



Step Mode
---------

Simple idea: Only advance the game simulation when the Player can make a move.
This allows players to take their time planning each move without pressure.

* [x] Initial implementation of Step Mode.
* [x] Add Step Mode toggle to SaveData and Options menu.



Hidden entities
---------------

Refactor how hidden entities work to make them more flexible and easier to use.
Hidden entities should be invisible, including when put inside walls, locks, or other occluding terrain.
Items should also be hidden when placed on top of Dirt terrain.
When the Player gets close enough, hidden entities should fade in smoothly showing their presence.

* [x] Refactor hidden state tracking.\
* [ ] Track hidden objects in FxState.\
* [ ] Render hidden entities without depth testing.\
* [ ] Fade hidden entities when Player is nearby.\



Miscellaneous
-------------

* [x] Improve WaterHazard terrain graphics.\
* [ ] Fix crash when spamming Gliders into ForceRandom and Teleport grid.\
* [ ] DAT convert: Put monsters not in the monster list on Traps.\



Startup Performance
-------------------

Loading the audio assets causes a long delay before the game starts.

* [ ] Load audio assets in the background after initial startup.\
