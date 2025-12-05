# Structural level operations

Add structural tools that let the editor expand and shrink the rectangular playfield without manual data surgery.
Expansion should let the user choose which edges to grow and what default terrain or contents fill the new area, so options like “add wall border” or “empty floor” are possible.
Shrinking should remove rows or columns at the edges but must consider what happens to entities that would fall outside the new bounds.

Make sure the underlying level and field representations stay flexible and can handle resizing without corrupting data.
Integrate these operations into the editor as menus or dialogs that use the new UI infrastructure, giving users a straightforward “Resize field” entry point.
The overall feel should be closer to resizing a canvas in an image editor than editing raw dimensions.

[x] Fix glitchy resize behavior when shrinking levels.\
[x] Fix crash when resizing to zero width/height.\
[x] Implement level expansion and shrinking in terms of `LevelBrush`.\

## Selection, move/copy, and fill

Introduce a selection mode for rectangular regions so users can manipulate areas instead of single tiles.
Click-drag should define a rectangle with a clear visual highlight, and there should be simple ways to cancel or adjust the selection.
Once a region is selected, tools can operate on it as a unit instead of requiring repetitive single-tile edits.

On top of selection, support moving and copying blocks of terrain and entities, with defined rules when pasted areas overlap existing content.
Add a terrain flood-fill tool to quickly change connected regions, making large-scale terrain adjustments practical.

[x] Introduce `LevelBrush` to paste level chunks.\
[x] Flood fill terrain tool.\
