Chip's Challenge Remake
=======================

A passion project born from fond memories of the classic puzzle game *Chip’s Challenge*, rewritten from scratch in Rust.

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

### Playing

1. Download the latest release.
2. Run `chipplay.exe` to start the game.
3. Place additional levelset files in the `levelsets` directory to play them.
4. Create your own levels with `chipedit.exe`.

### Building from Source

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Clone this repository.
3. Run the game: `cargo run --release --bin chipplay`
4. The level editor: `cargo run --release --bin chipedit`

### Publishing

* Install [Makurust](https://github.com/fromgodd/makurust).

Community Resources
-------------------

* [Bit Busters Wiki](https://wiki.bitbusters.club/Main_Page)
* [Chip's Challenge 1 Levels](https://sets.bitbusters.club/)
* [Andrew E.'s Chip's Challenge Page](https://tilde.town/~magical/chip/)
* [pieguy's site](https://davidstolp.com/old/chips/)

License & Credits
-----------------

### Source Code

This remake is released under the GNU General Public License v3.0.  
See the [license](license.md) file for more information.  

### Chip's Challenge

*Chip's Challenge* was created by Chuck Sommerville and is owned by Niffler Ltd.  
This project is an independent, open-source remake and is not affiliated with Niffler Ltd.

### Graphics

Default tileset: **Kayu's Enhanced Interface**

- Sourced from [https://tilde.town/~magical/chip/#kayu](https://tilde.town/~magical/chip/#kayu)

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
