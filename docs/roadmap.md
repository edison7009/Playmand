# Playmand roadmap

Playmand is the skill an AI agent uses to make games and visual work in Rust. Bevy is the default engine, but the skill is not tied to it: the agent picks the right tool for the task from a verified set, and always follows the user's choice of engine.

The goal is not to list the most engines. It is to be the Rust creation skill AI agents use best, so every tool that joins must arrive **with a runnable example, a pinned version and a check in CI**. A tool with only prose about it does not get a row in the selector.

## Three areas

| Area | What it covers | Default |
| --- | --- | --- |
| **Game** | Real-time games, 2D and 3D, ECS, input, UI, physics, audio | Bevy |
| **Motion** | Recorded animation and video, generative visuals, effects inside web pages and Tauri apps | Bevy with a fixed clock; Rust → WebAssembly + WebGL2 for page effects |
| **Simulation** | Many interacting bodies, particles, physics-heavy scenes | Bevy ECS + a physics engine |

Assets (Blender through MCP) and encoding (FFmpeg, AVFoundation) serve all three.

## Rules for adding a tool

1. **A runnable example** that checks observable behaviour, headless where possible, under `skills/playmand/examples/`.
2. **A pinned version** that works with the skill's Bevy baseline, or a clear note that it stands alone.
3. **CI** builds and tests it on Windows and Linux.
4. **A reference page** in English with a Chinese copy, loaded only when the task needs it.
5. **No cross-engine coupling**: a Macroquad project never pulls in Bevy, a page effect never pulls in an engine.
6. **An evaluation task** in `docs/evaluation.md` that shows the skill does better with the page than without it.

## Phases

### Phase 1 — Playmand as the entry point (done)

- [x] Rust → WebAssembly + WebGL2 effects for web pages and Tauri apps, with the live card pack at <https://playmand.com/> and a guide (`references/web-wasm.md`).
- [x] Reposition the skill and READMEs: "Rust games and visual creation, Bevy by default", not "a Bevy skill".
- [x] An **engine selector** in `SKILL.md`: pick by task, respect the user's engine.
- [x] **Physics with Avian** (Bevy-native ECS physics): a guide and a headless example with collisions, sensors and determinism. Rapier is covered as the alternative.

### Phase 2 — the pieces most games need (done)

- [x] **Audio with Kira** (music, sound effects, crossfades), tested through a mock sound card.
- [x] **egui** for debug panels and in-game tools, with real input events in tests.
- [x] **Macroquad** for small 2D games and light web builds (434 KB of WebAssembly, 150 KB gzipped, for the example game).
- [x] **wgpu** for custom render passes and GPU compute, run in CI on software adapters.

### Alongside: craft

- [x] **Game feel and feedback**: tiered rewards, squash and stretch, impact timing, safe cancellation and reduced motion, from the pet level-up effects; an engine-free example tests the beats and the player.

### Phase 3 — only when a real demo needs it

Fyrox (full 3D editor workflow), nannou (generative art; check its maintenance status first), Lightyear (multiplayer), Lyon and Vello (vector graphics). Each arrives with the demo that needed it.

## Not planned

- Bundling several engines into one project or one runtime.
- Migrating an existing project to another engine without the user asking.
- Adding a tool to the selector before it has an example and a CI check.
