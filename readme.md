Chip's Challenge Remake
=======================

Organization
------------

### Game: [./game](./game)

Platform independent game implementation.

Includes menus, game logic, rendering, audio, input handling, etc.

### Gameplay: [./gameplay](./gameplay)

Platform independent core gameplay implementation.

Just the game logic, to simulate the game without rendering or audio.

### Data: [./data](./data)

Game data files.

### Desktop application: [./play](./play)

Uses winit, glutin for windowing, shade for OpenGL rendering, soloud for audio.

### Level editor: [./editor](./editor)

License
-------

GPLv3 or later. See [license.md](./license.md).
