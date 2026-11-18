# Bevy assets, GLB and animation

Baseline: Bevy 0.20.1. The symbols here were checked against that version's crate source; this repository's windowless ECS example does not verify GLB or GPU rendering.

To create or fix source models, materials, rigs and animation, use [Blender and MCP](blender-mcp.md) on the source asset and export it, then bring it into Bevy with this page.

## Loading and instancing are different stages

`AssetServer::load` returns a handle immediately; that does not mean the asset has finished loading. `get_load_states` separates the root asset, direct dependencies and recursive dependencies; when the whole dependency tree must be ready, use `is_loaded_with_dependencies`. A failed load should show an error or a fallback, not stay on "Loading" forever.

Loading a GLB sub-scene in 0.20.1:

```rust
commands.spawn(WorldAssetRoot(
    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/hero.glb")),
));
```

Put this in a system that has `Commands` / `Res<AssetServer>`, with the GLTF and rendering plugins/features enabled. `Scene(0)` is only an example; the real index or name comes from the target file. A GLTF may depend on external textures or a bin file, which must also ship with the game.

Wait for `On<WorldInstanceReady>` before searching the instance's descendants for entities, meshes and animation players. Root asset loading, scene child creation, animation graph binding and the first GPU frame each have their own readiness condition. Do not replace them with "wait 32 frames".

## Root assets, shared handles and local changes

When you need several scenes or animations from one file, hold a `Handle<Gltf>`, and once `Assets<Gltf>::get` succeeds, take handles from `scenes` / `named_scenes` and `animations` / `named_animations`. Keep the root handle, or the strong handles you actually use, to keep the assets alive. Do not issue the same load again every frame.

Entities that use the same mesh or material handle share the asset. Changing that handle's value in `Assets<StandardMaterial>` affects everyone who uses it. When only one character should be tinted, clone the material value, `materials.add` it once, and swap that instance's `MeshMaterial3d`; do not create materials every frame. A preview and the main character can share immutable assets, but need their own scene entities, bone poses and playback state.

When switching characters repeatedly, clean up the entities owned by each instance root and drop resource handles or cache entries that are no longer used. Hiding an entity does not unload its assets, and unloading the GLB root does not guarantee nothing else holds strong references to its sub-assets. Judge lifetime by entity, asset and active-camera counts in the same state, not by one rise in working-set memory.

## Make the animation actually play on this model

Wiring animation in 0.20.1 involves:

1. Pass the target `Handle<AnimationClip>` to `AnimationGraph::from_clip` to get the graph and an `AnimationNodeIndex`; the index belongs to that graph and cannot be replaced by an arbitrary animation-array index.
2. Add the graph to `Assets<AnimationGraph>` and keep the returned handle.
3. Find the `AnimationPlayer` among the descendants of the instance that `WorldInstanceReady` refers to; it is not necessarily on the root entity you spawned.
4. Insert `AnimationGraphHandle` on that player entity, then call `player.play(index).repeat()`. For switching and blending, see `AnimationTransitions`.

This wiring does not retarget arbitrary skeletons. Animation target names/paths, skeleton hierarchy and the exported bind pose must match. For a T-pose, check the clip, graph, player and target binding first; for sliding feet, check whether the animation's root motion and the code's movement are applied twice. Do not hide errors by writing hard-coded global bone poses every frame.

## How assets affect the result

After importing a model, check size, axes, origin, normals, alpha mode and texture colour space; fix the clear problems first, rather than changing camera, exposure and materials all at once. When no assets in the target style exist, prepare them with the host's available generation tools or assets the user provides, and check them in the game. Placeholder geometry can prove a mechanic, but does not reach the visual style the user asked for.

API and wiring sources: [GLTF API](https://docs.rs/bevy/0.20.1/bevy/gltf/index.html), [AssetServer](https://docs.rs/bevy/0.20.1/bevy/asset/struct.AssetServer.html), and that version's official `examples/animation/animated_mesh.rs` and `animated_mesh_control.rs`. Use the [version locator](../scripts/inspect_bevy.py) to find the matching local examples instead of relying on the ever-changing main branch.
