ChipGame level editor
=====================

Simple level editor for ChipGame.

Controls
--------

Duplicate the _'template.json'_ file and rename it to your level name. Fill in some details about the level. The width and height cannot be changed later! To start the editor, run the following command:

```
cargo run --bin edit -- data/cc1/levelN.json
```

Then use the following controls:

* `T`: Terrain Tool

  - Left mouse button: Place a terrain tile
  - Right mouse button: Sample the terrain tile under the cursor

* `E`: Entity Tool

  - Left mouse button: Place/Select/Move an entity
  - Right mouse button: Select/Rotate the entity under the cursor
  - Delete: Delete the selected entity

* `C`: Connection Tool

  - Left mouse button: Create a connection between two tiles
  - Right mouse button: Delete a connection

* `F5`: Save the level

Click on the terrain samples on the left to change to the Terrain Tool. Click on the entity samples up top to change to the Entity Tool.

Don't forget to place the player entity!
