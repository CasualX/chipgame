# UI/menu infrastructure refactor

Refactor the shared game/editor menus into a cleaner UI system so custom UIs are easy to define, compose, and reuse.
Clarify and extend the `chipgame::menu` stack model, and make the relationship between editor overlays and world rendering explicit.
This may include a simple layout abstraction for sidebars and overlays, so new UI panels don’t require bespoke layout hacks.

## Editor UI rework and tool palette

Replace in-world templates with a menu-driven tool palette that feels like a proper toolbox.
The palette should support browsable categories such as terrain, entities, and templates, and show visual previews in a side panel or overlay.
Common tools and elements should have shortcut keys so experienced users can work quickly without excessive menu diving.

Build the palette on top of the refactored menu system by defining a palette menu type that supports scrolling lists, icons, and selection callbacks.
Palette selection should feed directly into the editor’s paint tools so choosing an item updates the active brush, with feedback such as a highlighted tool or ghost tile under the cursor.
This shifts the editor from “place from weird in-world objects” to a more standard, discoverable tool-based workflow.
