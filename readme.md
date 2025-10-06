Chip's Challenge Remake
=======================

A passion project born from nostalgia of the classic puzzle game *Chip’s Challenge*, rewritten from scratch in Rust.

Features
--------

- Inspired by the classic *Chip’s Challenge* released for Windows
- Smooth and responsive controls for a more enjoyable experience
- Retro aesthetic: 2D graphics displayed as 3D models and sprites
- Built-in level editor for creating and modifying levels
- Includes community level packs: CCLP1, CCLP3, CCLP4, and CCLP5
- Supports additional levelsets in the game's native .DAT level format

Downloads
---------

Precompiled binaries for Windows are available on the [Releases](https://github.com/CasualX/chipgame/releases) page.

Other platforms can be built from source (see below).

Getting Started
---------------

### Download and Play

1. Download and extract the latest release.
2. Run `chipplay.exe` to start the game.
3. Select a levelset from the menu to begin playing.
4. Place additional levelset files in the `levelsets` directory to play them.
5. Create your own levels with `chipedit.exe`.

### Build from Source

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Clone this repository.
3. Run the game: `cargo run --release --bin chipplay`
4. The level editor: `cargo run --release --bin chipedit`

### Publishing

* Install [Makurust](https://github.com/fromgodd/makurust).

Controls
--------

### Keyboard Controls

* Arrow keys, WASD: Move Chip
* Space: Select menu item
* Backspace: Close the current menu
* Enter: Open the Pause menu
* M: Toggle music

### Controller Bindings

* D-pad: Move Chip
* A: Select menu item
* B: Close the current menu
* Start: Open the Pause menu

Level Editor
------------

The game includes a very bare-bones level editor that can be used to create custom levels. Launch it by dragging a level onto the `chipedit.exe` executable.

* F2: Load level
* F5: Save level
* Arrow keys, WASD: Move cursor
* Left click: Select Tile or Entity, Place tile, Select Entity, Place Connection
* Right click: Rotate Entity
* Delete: Remove Entity
* T: Select Terrain Tool
* E: Select Entity Tool
* C: Select Connection Tool

Community Resources
-------------------

* [Bit Busters Wiki](https://wiki.bitbusters.club/Main_Page)
* [Community levelsets](https://sets.bitbusters.club/)
* [Andrew E.'s Chip's Challenge Page](https://tilde.town/~magical/chip/)
* [pieguy's site](https://davidstolp.com/old/chips/)

License & Credits
-----------------

### Chip's Challenge

*Chip's Challenge* was created by Chuck Sommerville and is owned by Niffler Ltd.  
This project is an independent, open-source remake and is not affiliated with Niffler Ltd.

### Source Code

This remake is released under the GNU General Public License v3.0.  
See the [license](license.md) file for more information.  

### Graphics

Default tileset: *Kayu's Enhanced Interface*

- Sourced from [https://tilde.town/~magical/chip/#kayu](https://tilde.town/~magical/chip/#kayu)

Additional graphics: *tileworld*

- Sourced from [retrofw/tileworld](https://github.com/retrofw/tileworld)

### Music

Music files written by chaozz of gp32x.com and am-fm's music is used under Creative Commons License.

* Sourced from [retrofw/tileworld](https://github.com/retrofw/tileworld)

### Sound Effects

The sound effects included in this distribution were created by Brian Raiter, with assistance from SoX. Brian Raiter has explictly placed these files in the public domain.

- Sourced from [SicklySilverMoon/tworld](https://github.com/SicklySilverMoon/tworld)

### Community Level Packs

This remake is distributed with several community-created level packs.

These packs were designed and maintained by the Chip’s Challenge community and are distributed freely for use with compatible engines such as Tile World.

* [Chip's Challenge Level Pack 1](https://wiki.bitbusters.club/Chip%27s_Challenge_Level_Pack_1)
* [Chip's Challenge Level Pack 3](https://wiki.bitbusters.club/Chip%27s_Challenge_Level_Pack_3)
* [Chip's Challenge Level Pack 4](https://wiki.bitbusters.club/Chip%27s_Challenge_Level_Pack_4)
* [Chip's Challenge Level Pack 5](https://wiki.bitbusters.club/Chip%27s_Challenge_Level_Pack_5)
