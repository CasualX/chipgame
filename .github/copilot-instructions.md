ChipGame – AI coding guide
==========================

Purpose: Give AI agents the minimum, project-specific context to be productive in this Rust monorepo.

Architecture (crates)
---------------------
- chipcore: deterministic simulation only (`chipcore/src/*`: entities, physics, input, gamestate). Emits `GameEvent` from `GameState::tick`.
- chipgame: rendering/UI layer (OpenGL). `fx/` mirrors `GameState` and consumes `GameEvent` → emits `FxEvent`; `menu/` (stack menus) → `MenuEvent`; `play/` wires menus+fx, persists saves, emits `PlayEvent`.
- chipplay: the app. Translates OS input → `chipcore::Input`, loads assets, plays audio, manages window (`chipplay/src/main.rs`).
- chipedit: editor UI using same rendering stack (`chipedit/src/main.rs`, `chipgame/src/editor/mod.rs`).
- chipty: shared DTOs/contracts (level, replay, terrain, savedata, soundfx).
- chipdat: DAT parsing/utilities for CC1 levelsets.

Data/event flow
---------------
- Input → `chipcore::Input` bits
- Sim: `GameState::tick` mutates state, pushes `GameEvent`
- Fx: `FxState::sync` consumes `GameEvent` → updates visuals/effects; emits `FxEvent::{PlaySound,PlayMusic,GameWin,GameOver,...}`
- Menus: `menu::*` emits `MenuEvent` via `KeyState::w(prev,cur)` transitions
- Orchestration: `PlayState::sync` consumes `MenuEvent` + `FxEvent`, updates saves, (re)loads level, pushes `PlayEvent`
- App: `chipplay` consumes `PlayEvent` and actually plays sfx/music or quits

FxState vs RenderState
----------------------
- FxState (`chipgame/src/fx/fxstate.rs`) knows about the game. It holds `GameState`, drives the `PlayCamera`, consumes `GameEvent`s via `sync()`, and translates them into renderable state by creating/updating entries in `render::ObjectMap`, tweaking animations, and enqueuing transient VFX. It also emits `FxEvent` for audio and high-level play-state changes.
- RenderState (`chipgame/src/render/renderstate.rs`) is generic. It has no game rules; it stores draw-time data only: `RenderField {width,height,terrain}`, an `ObjectMap<Object>` with sprites/models/anim, a clock (`time, dt, framecnt`), and a list of simple `Effect`s. Its `update()` advances object animations and prunes effects; its `draw()` issues draw calls with shaders/textures. Implement new visual effects here without introducing game logic.
- Handlers (`chipgame/src/fx/handlers.rs`) are the glue: per-`GameEvent` functions that mutate `FxState` and its `render::RenderState` (e.g., spawn/remove objects, change terrain tiles, trigger effects).

Build/run/test
--------------
- Play: `cargo run --bin chipplay`
- Edit: `cargo run --bin chipedit`
- Tests (replays): `cargo test -p chipcore` (see `chipcore/tests/replays.rs`; expects `levelsets/*/lv/` and `chipcore/tests/replays/**`)

Assets/filesystem
-----------------
- `chipgame::FileSystem` reads from `data/` (dev) or `data.paks` (packed)
- Shaders: `pixelart.*`, `ui.*`, `color.*`; textures: `tileset/*.png`, `effects.png`; audio in `data/sfx*` and `data/music/`
- Tiles are 32×32; world rendering in `chipgame/src/render/`; camera in `chipgame/src/fx/camera/`

Conventions and gotchas
-----------------------
- Strict layering: core (no gfx/audio) → fx/menu/play (no sim mutation except via `tick`/parse) → app (IO only)
- Input transitions via `menu::KeyState::w(prev,cur)`; `PlayState` caches previous input
- Levels are 1-based in UI; saves/replays live under `save/<Levelset>/`
- Music alternates per level number; persisted options live in `PlayState::save_data`
- Use `eprintln!` to trace `GameEvent`/`FxEvent`/`MenuEvent`

Common changes (where)
----------------------
- New gameplay features: go in `chipcore` first (simulation and rules). Emit `GameEvent` as needed from `GameState::tick`; do not couple to rendering/audio.
- New graphics techniques: implement in `chipgame/src/render/*` (generic draw-time only). Add draw-time data/structures and shader usage without introducing game logic.
- New visual effects derived from game state: add in `chipgame/src/fx/*`. Consume `GameEvent` in handlers to update `RenderState` (spawn objects, animations, transient `Effect`s) and push `FxEvent` for audio/gameflow.

- New entity: implement behavior in `chipcore/src/entities/*` and integrate with `GameState`; emit appropriate `GameEvent::{EntityCreated,EntityRemoved,EntityStep,...}`. For visuals, add per-event handlers in `chipgame/src/fx/handlers.rs` to create/update/remove `render::Object`s and set `data::SpriteId`/`AnimationId`. Add any new sprite/model ids under `chipgame/src/data/*`.
- New terrain/tile: add terrain type in `chipty::terrain` and handle rules in `chipcore` (including `TerrainUpdated` events when it changes). Map terrain to graphics in render: update tileset mapping in `chipgame/src/render/` (e.g., tile drawing and `TileGfx`). If the terrain has dynamic visuals (e.g., toggle walls, fire), add handlers in `fx/handlers.rs` to create/remove helper objects and animate as events arrive.
- New sound/music: add ids in `chipty`, load in `chipplay/src/main.rs` (`AudioPlayer::load_*`), and trigger via `GameEvent::SoundFx` (consumed in `FxState` → `FxEvent::PlaySound`). Music selection remains orchestrated by play-state logic.
- New menu/screen: add under `chipgame/src/menu/*`, wire into the `Menu` enum and dispatch; menus produce `MenuEvent` on input transitions via `KeyState::w(prev,cur)`.

Quick examples
--------------
- Start level: `PlayState::play_level(n)` → parses JSON → `FxState::parse_level` → posts `PlayEvent::PlayLevel`
- Save replay: `fx.gs.save_replay` → JSON; tests decode with `chipty::decode` in `chipcore/tests/replays.rs`
