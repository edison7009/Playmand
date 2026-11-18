# Bevy verification and distribution

## Choose checks by the kind of change

| Change | Suitable check | What it does not cover |
| --- | --- | --- |
| Queries, commands, states, resource lifecycle | Register the real systems in a trimmed `App`, advance the schedule, and check the World afterwards; see [ecs-patterns](../examples/ecs-patterns/src/lib.rs) | Rendering and real window input |
| Frame-rate-dependent movement or cooldowns | Advance the simulation with a controlled timestep and check displacement, counts and pause/resume; fixed-step tests configure the time plugin and time strategy explicitly | Game feel |
| Keys, mouse, UI behaviour | Drive the project's input path and check state and feedback; in trimmed tests, input plugins clear one-shot state at frame boundaries | Synthetic input does not prove OS focus or a Chinese IME |
| GLB, animation, UI, cameras, materials | Use the real render path, wait for assets and the scene to be ready, and check images or a time series of motion | One screenshot does not prove a whole animation, every interaction or performance |
| Performance | Sample a release build with the same config, scene, viewport, foreground/background state and warm-up | Entity counts are not VRAM, and an offline encode's frame rate is not runtime FPS |

A trimmed `App` registers only the plugins the rule depends on; a game using `DefaultPlugins` does not mean logic tests must start the GPU. `MinimalPlugins` does not include every state, resource or rendering feature; add the plugins the actual systems need. Normal frame-update tests should run input and rules, not write the final score directly.

Pick these checks as the task needs and follow the project's required checks. `cargo check` does not run systems, so it cannot find every query initialization conflict, and `cargo test` passing does not prove the image is right. When the GPU cannot run, complete the available checks and state the visual verification gap clearly.

## Asset paths and player builds

`AssetServer` paths are relative to the asset source; do not write absolute paths from the development machine. The root of the default native file asset source depends on the launch environment and `AssetPlugin` settings; confirm it in that version's `bevy_asset::io::file` source rather than assuming it follows the shell's current folder. Fix the in-package asset layout and the launch entry point so development-time environment variables do not hide missing files.

A package includes the exe, the asset folder and the real dynamic library dependencies; external GLTF textures/bins, fonts and sounds are dependencies too. `dynamic_linking` is only for an explicitly chosen development configuration; player builds must handle real runtime library dependencies. Save files go to the user-data folder the project chose, not to a read-only install folder by default, and tests must not overwrite a player's existing saves.

Put this build into a separate distribution folder, launch it away from the source tree and from a different working folder, and check load logs, the image, input, and any save/exit paths this change touches. Generating a package and that package running are two different results.

Hand-off notes keep the launch command, the checks that actually passed and what is still unverified; for hand-over, use the project's existing records instead of producing a new report for every game task.
