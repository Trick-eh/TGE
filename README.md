# Trick's Game Engine (Rust)

A 2D game engine written in Rust, scriptable via Lua or native Rust code, with
interchangeable OpenGL and Vulkan rendering backends. Built as a learning
project.

> Status: actively under construction. See [Project Status](#project-status)
> below for what's implemented vs. planned.

## Features

- **Two render backends, one trait.** `OpenGLRenderer` and `VulkanRenderer`
  both implement the same `Renderer` trait in `engine-renderer`, selected via
  mutually exclusive Cargo features (`opengl` / `vulkan`).
- **Lua or Rust scripting.** Write your game logic in Lua (via `mlua`,
  Lua 5.4, vendored) with hot-reload in dev mode, or implement the `App`
  trait directly in Rust for a fully native game.
- **ECS-based world.** Entities and components via `hecs`, wrapped in a
  data-oriented `World` with built-in `Transform2D`, `Sprite`, `Velocity`,
  and other components.
- **Sprite batching.** Draw calls are batched per-frame to keep draw counts
  low even with many sprites on screen.
- **Input.** Polling-based keyboard/mouse/gamepad state (via `gilrs`) with
  an action-mapping layer (`bind_action` / `is_action_held`, etc.) so games
  don't hardcode physical keys.
- **Audio.** Sound effects and music playback via `kira`, with separate
  master/SFX/music volume controls.
- **Fixed-timestep simulation** with interpolated rendering (`alpha`
  blending between the last two physics snapshots) to keep motion smooth
  independent of frame rate.
- **Hot reload (Lua dev mode).** Edit `main.lua`, save, and the running game
  reloads — with an explicit `persist()` / `get_persisted()` API for state
  that should survive the reload.
- **Save data.** Simple key/value persistence to disk across process
  restarts (`save_data` / `load_data` in Lua).

## Workspace layout

```
engine-core      the runtime: window/event loop, game loop, Lua bindings, save data
engine-math      Vec2/Mat4 (via glam), Transform2D, Camera2D, AABB
engine-input     KeyCode/MouseButton/GamepadButton state machine, action bindings
engine-ecs       World, Entity, and built-in components (thin wrapper over hecs)
engine-audio     AudioManager / AudioAssets (via kira)
engine-renderer  Renderer trait + OpenGL and Vulkan implementations
xtask            cargo-xtask style build/run/scaffold tooling
game-example     example 'game' using the native Rust App trait
snake-clone      example game
```

Crate boundaries were decided up front so that
`engine-renderer` stays a pure trait crate — nothing else in the engine
depends on OpenGL or Vulkan types directly.

## Getting started

This project uses the [`xtask`](https://github.com/matklad/cargo-xtask)
pattern instead of a Makefile.

```bash
# build just the engine core
cargo xtask build

# run a game crate with the OpenGL backend (default)
cargo xtask run <game-crate-name>

# run with Vulkan instead
cargo xtask run <game-crate-name> vulkan

# run cargo check + clippy across the whole workspace
cargo xtask check
```

### Starting a new game

```bash
# scaffold a Lua-scripted game in the current crate
cargo xtask init-lua-project

# scaffold a Rust-native game in the current crate
cargo xtask init-rust-project
```

`init-lua-project` writes out a `src/main.lua` with the lifecycle stubs
(`on_start`, `on_update`, `on_fixed_update`, `on_render`, `on_stop`,
`on_resize`) plus a `.luarc.json` and Lua type definitions
(`definitions_engine.lua`) so editors get autocomplete against the real
engine API.

### Toolchain requirements

- Rust (stable)
- For the Vulkan backend: the Vulkan SDK and `glslc` on your `PATH` (shaders
  are compiled to SPIR-V at build time in `engine-renderer`'s `build.rs`)
- For Vulkan validation layers in debug builds: `VK_LAYER_KHRONOS_validation`
  installed (optional — the engine warns and continues without it if
  missing)

## Scripting API (Lua)

Games can be written entirely in Lua against the `engine` global — spawning
entities, binding input actions, playing audio, drawing text/rects, and
reading/writing persistent or save-file data. See
[`definitions_engine.lua`](./definitions_engine.lua) for the full annotated
API surface (spawn/despawn, transform and velocity accessors, sprite sheets,
fonts, input polling, audio, save/persist).

## Project status

This game engine is still under development and is not a final product nor
something ready a for a crazy 2D game idea since is very limited. That said
feel free to try it and tweak it as you desire.

## Contributing

This is currently a solo learning project, so there's no formal contributing
process yet. If you spot a bug or have a suggestion, opening an issue is the
best way to reach out.
