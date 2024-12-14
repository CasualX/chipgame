Chip's Challenge Remake
=======================

Organization
------------

### Gameplay: [./chipcore](./chipcore)

Platform independent core gameplay implementation.

Just the core game logic, to simulate the game without rendering or audio.

### Game: [./chipgame](./chipgame)

Platform independent game implementation.

Includes menus, game logic, rendering, audio, input handling, etc.

### Data: [./data](./data)

Game data files.

### Desktop application: [./chipplay](./chipplay)

Uses winit, glutin for windowing, shade for OpenGL rendering, soloud for audio.

### Level editor: [./chipedit](./chipedit)

References
----------

* https://wiki.bitbusters.club/Main_Page
* https://tilde.town/~magical/chip/

Makurust to render markdown files:

* https://github.com/fromgodd/makurust

License
-------

GPLv3 or later. See [license.md](./license.md).
