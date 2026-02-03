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
* [x] Identify and fix levels that cannot be completed.\


Validate CCLP3 levels
---------------------

Play through all CCLP3 levels and check the levels can be reasonably completed.

* [ ] Reprocess CCLP3 levelset to take advantage of new conversion features.\
* [ ] Add CCLP3 replay tests.\
* [ ] Identify and fix levels that cannot be completed.\


[You Can't Teach an Old Frog New Tricks](https://wiki.bitbusters.club/You_Can%27t_Teach_an_Old_Frog_New_Tricks)
--------------------------------------

This famous level is known to not work correctly in my implementation.

* [x] Walker room: Timing of Tanks does not line up with official solution.\
* [x] Glider room: Teeth stuck in Trap should retain momentum when released instead of turning.\
* [x] Timing of Fireballs is precise due to higher tickrate, making splitting them up harder.\



Player GameOver one tick earlier
--------------------------------

The Player moves ever 12 frames, but level finish triggers one extra tick later, let's fix this.

* [x] Trigger GameOver one tick earlier when the Player reaches the exit.\
* [x] Fix replay validation to allow an additional tick of input.\
* [x] Player Death sprite should appear after move animation completes.\



Visual indicator for BearTrap terrain state
-------------------------------------------

Playtesters have reported being very confused about the state of BearTraps.

* [x] Improve readability for Inactive BearTraps.\



Texture shimmering
------------------

Tiling floor textures shimmer at the edges and have visible hard edges.
This is caused because the padding around the tiles is using Clamp mode, instead use Repeat mode to fill in the padding.

* [x] Update texture padding to use Repeat mode instead of Clamp mode for specific tiles.\



Miscellaneous
-------------

* [x] Improve WaterHazard terrain graphics.\
* [x] Improve Paramecium graphics by offsetting the sprite slightly.\
* [x] Fix crash when spamming Gliders into ForceRandom and Teleport grid.\
* [ ] DAT convert: Put monsters not in the monster list on Traps.\
* [x] Editor: Show current tool and grid coordinates on screen.\
* [x] Editor: Center camera on level after level load.\



Prologue and Epilogue screens
-----------------------------

Display story screens before or after certain levels to provide story context.

* [x] Add story entries to the LevelSetDto structure.\
* [ ] Design and implement story screen layout and style.\
* [ ] Add story to CCLP1 levelset.\
* [x] Add story to CCLP2 levelset.\
* [ ] Add story to CCLP3 levelset.\
* [ ] Add story to CCLP4 levelset.\
* [ ] Add story to CCLP5 levelset.\



Display Keybinds
----------------

Players do not read the readme, so they are often unaware how to navigate the menus.

* [ ] Add 'Enter' and 'Back' menu options to the Unlock Level menu.\
* [x] Support glyph-based fonts rendering.\
* [ ] Display keybinds somewhere in the Game itself.\
* [ ] Display keybinds somewhere in the Editor itself.\



Startup Performance
-------------------

Loading the audio assets causes a long delay before the game starts.

* [ ] Load audio assets in the background after initial startup.\


Editor: Undo/Redo System
------------------------

Implement an undo/redo system in the level editor to allow users to revert or reapply changes made during level editing.
To keep things simple just snapshot the entire level state after certain actions.
Keep a limited history of snapshots to manage memory usage effectively.

* [x] Implement a simple history stack to store level state snapshots.\
* [x] Add undo and redo commands to the editor interface.\
* [x] Add 'Are you sure you want to quit?' dialog when there are unsaved changes.\



Editor: Implement Toolbox Menu
------------------------------

Create a toolbox menu in the level editor that displays icons for different editing tools on the left side of the screen.
This will allow users to easily select and switch between various tools for level editing.

* [x] Tool: Autotiling for Force and Ice terrain.
* [x] Tool: Refactor Tool State.
* [x] Tool: Implement EntOrder tool to change entity ordering.



Editor: Implement custom rendering for LevelDto
-----------------------------------------------

Currently the level editor reuses the game's FxState to render levels, which implements mechanics like hidden entities that are not desired in the editor.
There are already workarounds for other gameplay graphics features which complicate FxState.
Simplify the rendering logic by implementing a custom visualization for LevelDto that directly draws the level without gameplay mechanics.
