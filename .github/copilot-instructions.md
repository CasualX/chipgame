ChipGame – AI coding guide
==========================

Purpose: Crisp, project-specific breadcrumbs so GPT-style agents can contribute immediately without re-deriving architecture.

Crate map (who owns what)
-------------------------
- chipcore: deterministic sim only (`gamestate`, `entities`, `movement`, `terrain`). `GameState::tick` emits `GameEvent` and must stay pure/portable.
- chipgame: presentation stack. `fx/` mirrors `GameState` + consumes `GameEvent` → `FxEvent`; `menu/` implements stack-driven UI → `MenuEvent`; `play/PlayState` glues menus+fx, persists saves, emits `PlayEvent`.
- chipplay: desktop app wrapper. Translates OS input to `chipcore::Input`, owns `AudioPlayer`, window, and data loading.
- chipedit: level editor binary that reuses chipgame’s renderer/editor widgets (see `chipgame/src/editor`).
- chipty: shared DTOs for tiles, entities, saves, sound ids; use when data crosses crate boundaries.
- chipdat: DAT/packfile helpers for legacy CC1 assets.

Data + event loop
-----------------
- OS input → `chipcore::Input` → `GameState::tick` (mutates sim, queues `GameEvent`).
- `FxState::sync` (in `chipgame/src/fx/fxstate.rs`) consumes those events, updates render objects/camera, and pushes `FxEvent::{PlaySound,PlayMusic,GameWin,...}`.
- Menu stack (`chipgame/src/menu`) watches `KeyState::w(prev,cur)` to emit `MenuEvent` for focus/selection.
- `PlayState::sync` (render/play layer) ingests `MenuEvent + FxEvent`, reloads levels, updates saves, posts `PlayEvent` to `chipplay`.
- `chipplay` reacts to `PlayEvent` (actual audio playback, quitting, switching levelsets).

Render layering contract
------------------------
- `FxState` knows gameplay and can mutate `render::ObjectMap` plus queue transient VFX; never talk to OS/audio here.
- `render::RenderState` (`chipgame/src/render/renderstate.rs`) is dumb draw-time data (`RenderField`, `Object`s, time). Keep visual tech (animations, shaders) here without mixing rules.
- Handlers live in `chipgame/src/fx/handlers.rs`: each `GameEvent` spawns/despawns objects, swaps terrain tiles, and enqueues `Effect`s. Add new visuals by extending these handlers plus relevant sprite/animation ids from `chipty`.

Developer workflow
------------------
- Run game: `cargo run --bin chipplay`. Editor: `cargo run --bin chipedit`.
- Determinism tests: `cargo test -p chipcore` (exercises `chipcore/tests/replays.rs`; expects `levelsets/*/lv/` + `chipcore/tests/replays/**`).
- Assets resolve via `chipgame::FileSystem` (prefers `data/` during dev, `data.paks` when packed). `tileset/*.png`, `effects.png`, shaders in `data/shaders`, audio under `data/sfx*` + `data/music`.

Conventions + gotchas
---------------------
- Strict layering: chipcore (no gfx/audio), chipgame fx/menu/play (no direct sim mutation except via `GameState`/parsers), chipplay (IO only). Respect to keep tests deterministic.
- Terrain + entities expect 32×32 tiles; render camera helpers live in `chipgame/src/fx/camera`.
- Level numbers are 1-based; saves/replays live in `save/<Levelset>/replay`. `PlayState::save_data` persists options and music options.
- Use `eprintln!` tracing hooks already sprinkled through Fx/Menu/Play when debugging event flow.

Common modifications
--------------------
- New gameplay features: go in `chipcore` first (simulation and rules). Emit `GameEvent` as needed from `GameState::tick`; do not couple to rendering/audio.
- New graphics techniques: implement in `chipgame/src/render/*` (generic draw-time only). Add draw-time data/structures and shader usage without introducing game logic.
- New visual effects derived from game state: add in `chipgame/src/fx/*`. Consume `GameEvent` in handlers to update `RenderState` (spawn objects, animations, transient `Effect`s) and push `FxEvent` for audio/gameflow.

- New entity: implement behavior in `chipcore/src/entities/*` and integrate with `GameState`; emit appropriate `GameEvent::{EntityCreated,EntityRemoved,EntityStep,...}`. For visuals, add per-event handlers in `chipgame/src/fx/handlers.rs` to create/update/remove `render::Object`s and set `chipty::SpriteId`/`AnimationId`.
- New terrain/tile: add terrain type in `chipty::terrain` and handle rules in `chipcore` (including `TerrainUpdated` events when it changes). Map terrain to graphics in render: update tileset mapping in `chipgame/src/render/` (e.g., tile drawing and `TileGfx`). If the terrain has dynamic visuals (e.g., toggle walls, fire), add handlers in `fx/handlers.rs` to create/remove helper objects and animate as events arrive.
- New sound/music: add ids in `chipty`, load in `chipplay/src/main.rs` (`AudioPlayer::load_*`), and trigger via `GameEvent::SoundFx` (consumed in `FxState` → `FxEvent::PlaySound`). Music selection remains orchestrated by play-state logic.
- New menu/screen: add under `chipgame/src/menu/*`, wire into the `Menu` enum and dispatch; menus produce `MenuEvent` on input transitions via `KeyState::w(prev,cur)`.

Quick reference flows
---------------------
- Level boot: `PlayState::play_level(n)` → parses JSON (level + save) → `FxState::new` primes render objects → posts `PlayEvent::SetTitle` for chipplay to switch.
- Replay/save: `fx.game.save_replay` dumps JSON; deterministic tests reload via `chipty::decode` inside `chipcore/tests/replays.rs`.
