Backlog for vNext
=================



Step Mode
---------

Simple idea: Only advance the game simulation when the Player can make a move.
This allows players to take their time planning each move without pressure.

* [x] Initial implementation of Step Mode.\
* [x] Add Step Mode toggle to SaveData and Options menu.\
* [ ] Fix replay desync when using Step Mode.\

The desync is caused by clearing the input buffering when stepping the simulation.
When replaying, the buffered inputs are not cleared, causing extra moves to be processed.



Assist Mode
-----------

Assist Mode provides visibility to hidden dangers and items in the level.
When enabled, hidden things fade in when the Player is nearby.
It also enables the Set Warp Point feature in the Pause menu.

* [x] Entities show when hidden behind Terrain and Blocks.\
* [x] Fire shows when hidden behind Blocks.\
* [x] FakeBlueWall fades when Player is nearby.\
* [x] Disable Set Warp Point when Assist Mode is off.\



Hidden entities
---------------

Refactor how hidden entities work to make them more flexible and easier to use.
Hidden entities should be invisible, including when put inside walls, locks, or other occluding terrain.
Items should also be hidden when placed on top of Dirt terrain.
When the Player gets close enough, hidden entities should fade in smoothly showing their presence.

* [x] Refactor hidden state tracking.\
* [x] Track hidden objects in FxState.\
* [x] Render hidden entities without depth testing.\
* [x] Fade hidden entities when Player is nearby.\



Validate CCLP2 levels
---------------------

Play through all CCLP2 levels and check the levels can be reasonably completed.

* [x] Add CCLP2 replay tests.\
* [ ] Identify and fix levels that cannot be completed.\



[You Can't Teach an Old Frog New Tricks](https://wiki.bitbusters.club/You_Can%27t_Teach_an_Old_Frog_New_Tricks)
--------------------------------------

This famous level is known to not work correctly in my implementation.

* [ ] Walker room: Timing of Tanks does not line up with official solution.\
* [x] Glider room: Teeth stuck in Trap should retain momentum when released instead of turning.\
* [ ] Timing of Fireballs is precise due to higher tickrate, making splitting them up harder.\



Player GameOver one tick earlier
--------------------------------

The Player moves ever 12 frames, but level finish triggers one extra tick later, let's fix this.

* [ ] Trigger GameOver one tick earlier when the Player reaches the exit.\
* [ ] Fix replay validation to allow an additional tick of input.\
* [ ] Player Death sprite should appear after move animation completes.\



Miscellaneous
-------------

* [x] Improve WaterHazard terrain graphics.\
* [ ] Fix crash when spamming Gliders into ForceRandom and Teleport grid.\
* [ ] DAT convert: Put monsters not in the monster list on Traps.\
* [x] Editor: Show current tool and grid coordinates on screen.\
* [ ] Editor: Center camera on level after level load.\



Intermission Screens
--------------------

Display an intermission screen before or after certain levels to provide story context.

* [ ] Add intermission entries to the LevelSetDto.levels structure.\
* [ ] Design and implement intermission screen layout and style.\



Display Keybinds
----------------

Players do not read the readme, so they are often unaware how to navigate the menus.

* [ ] Add 'Enter' and 'Back' menu options to the Unlock Level menu.\
* [ ] Support glyph-based fonts rendering.\
* [ ] Display keybinds somewhere in the Game itself.\
* [ ] Display keybinds somewhere in the Editor itself.\



Startup Performance
-------------------

Loading the audio assets causes a long delay before the game starts.

* [ ] Load audio assets in the background after initial startup.\



Reprocess the levelsets
-----------------------

The levelsets were converted from DAT files a long time ago, and the conversion tool has improved since then.
Reprocess the levelsets using the latest conversion tool to ensure consistency and take advantage of any improvements.

* [ ] Reprocess levelset CC1.\
* [ ] Reprocess levelset CCLP1.\
* [ ] Reprocess levelset CCLP2.\
* [ ] Reprocess levelset CCLP3.\
* [ ] Reprocess levelset CCLP4.\
* [ ] Reprocess levelset CCLP5.\



Editor: Undo/Redo System
------------------------

Implement an undo/redo system in the level editor to allow users to revert or reapply changes made during level editing.
To keep things simple just snapshot the entire level state after certain actions.
Keep a limited history of snapshots to manage memory usage effectively.

* [ ] Implement a simple history stack to store level state snapshots.\
* [ ] Add undo and redo commands to the editor interface.\
* [ ] Add 'Are you sure you want to quit?' dialog when there are unsaved changes.\



Editor: Implement Toolbox Menu
------------------------------

Create a toolbox menu in the level editor that displays icons for different editing tools on the left side of the screen.
This will allow users to easily select and switch between various tools for level editing.

* [ ] Tool: Autotiling for Force and Ice terrain.



Editor: Implement custom rendering for LevelDto
-----------------------------------------------

Currently the level editor reuses the game's FxState to render levels, which implements mechanics like hidden entities that are not desired in the editor.
There are already workarounds for other gameplay graphics features which complicate FxState.
Simplify the rendering logic by implementing a custom visualization for LevelDto that directly draws the level without gameplay mechanics.
