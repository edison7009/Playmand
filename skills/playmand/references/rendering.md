# Bevy rendering, UI and performance

Baseline: Bevy 0.20.1. Deal with the game's actual image first, and check fields against the project's version. This is a source-checked technical index; GPU results must be verified by running the target game.

## Wiring 2D and 3D

2D starts with `Camera2d` plus `Sprite` or `Mesh2d`; 3D uses `Camera3d`, `Mesh3d`, `MeshMaterial3d<StandardMaterial>` and `Transform`. In 0.20 `Camera2d` defaults to `Tonemapping::Linear` and `Camera3d` enables `DebandDither` by default; `Tonemapping::None` is now a full passthrough, so use `Linear` to keep the old "tone curve off" look. Sprites now go through the Mesh2d/Material2d backend, and sprites at the same Z may draw in a different order; give them distinct Z values when layering must be stable. Modern Bevy fills in base components through required components, so do not bring back the old `*Bundle` types from tutorials. The material type must match the render path.

```rust
commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

With `looking_at`, make sure the target and position differ. Choose perspective or orthographic projection by game design; camera angle, depth of field, colour and feedback still follow the user's art direction, and "there is a cube on screen" does not mean the visuals are done.

Separate the cases when 3D is black or objects are pure black:

| Symptom | Bevy checks |
| --- | --- |
| The target scene is missing entirely | Active `Camera`, render target, Transform/GlobalTransform, near/far planes, asset load errors |
| Only some objects disappear | `Visibility` and inherited visibility, bounding boxes/culling, `RenderLayers`, back faces from negative scale, alpha mode |
| Geometry is there but materials are black | Normals/textures, lights, exposure, ambient light; in 0.20.1 global ambient light is `GlobalAmbientLight` |
| No shadows | The version's light fields, material/mesh cast and receive settings, shadow distance; 0.20.1 lights use `shadow_maps_enabled` |
| The scene draws twice or overlaps | Multiple cameras' target, order, clear settings and layers; rule out the same entity drawn by several passes |

`RenderLayers` is not inherited down the entity hierarchy. After setting layers on a GLB root, meshes inside the instance can stay on the default layer; handle the actual descendants once the instance is ready. Lights must also match the camera and mesh layers. The [RenderLayers docs](https://docs.rs/bevy/0.20.1/bevy/camera/visibility/struct.RenderLayers.html) explain the intersection rules.

## Bevy UI

In 0.20.1 the main layout component is `Node`; text uses `Text`, `TextFont`, `TextColor` and related components. Do not mix in old `Style` / `TextBundle` fields. Express layout through parent container constraints, distinguish percentage from pixel sizes, and keep absolutely positioned decoration separate from the content flow.

In 0.20 `Interaction` is deprecated: read button state from `Hovered` / `Pressed`, and `bevy::ui::widget::Button` gives way to `ui_widgets::Button`. Older projects that still use `Changed<Interaction>` can keep it; do not migrate everything just to silence a deprecation warning. `TextFont::default()` changed its font size from 20px to `FontSize::Rem(1.)`; write `font_size: FontSize::Px(20.)` when you need a fixed pixel size. Editable input needs both `EditableText` and `TextInput`. Theme updates write only the values that changed and do not rebuild the whole UI tree every frame. Focus, text editing, submit errors and mouse hover are different states; English text input working does not prove the Chinese IME path works.

Read actual sizes from the post-layout `ComputedNode`, after `PostUpdate` / `UiSystems::Layout`, and mind the conversion between physical pixels and UI logical coordinates/DPI. With several cameras, set `UiTargetCamera` on each UI root so the main HUD does not end up in a character preview texture.

## Screenshots and off-screen previews

The version's official `examples/window/screenshot.rs` shows:

```rust
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
// In a system, request it once:
commands.spawn(Screenshot::primary_window()).observe(save_to_disk("capture.png"));
```

A screenshot goes through rendering and GPU readback; the command returning does not mean the file is saved. Request it only after the target assets and scene are ready, wait for capture and the write, and open the actual image to check it. The off-screen target and the main window choice must match. Do not request a screenshot every frame and pile up readbacks; for frame-by-frame recording, follow the [animation and video guide](video.md).

Render-to-texture previews involve an `Image` asset, a `RenderTarget`, a camera and UI display; wire them like that version's `examples/ui/render_ui_to_texture.rs`. Hiding the panel's UI does not turn its camera off; set `Camera::is_active` by lifecycle and stop needless pose updates. Static previews can refresh after changes; animated previews must keep rendering and cannot be replaced by a still screenshot.

## Optimize from measurements

First tell apart slow compiles, a slow main-thread simulation, slow render submission and a slow GPU. `FrameTimeDiagnosticsPlugin` shows overall frame time; for attribution, use a profiler the target environment already has. FPS is not the cost of any single system or GPU pass.

- ECS hot spots: full-table scans every frame, repeated string allocation, needless change detection, long serial system dependencies.
- Rendering hot spots: extra cameras, shadow-casting lights, transparent overdraw, high-resolution preview textures and heavy post-processing.
- Asset hot spots: repeated `Assets::add`, long-lived strong handle caches, entities or tasks left behind after a level exits.

Reusing assets, narrowing queries and turning off unused cameras should address a measured problem. Do not cut the visuals or content the user asked for to raise the frame rate. Compare with the same release configuration, scene, hardware, window size, foreground/background state and warm-up; an offline video's encoded frame rate is not the runtime frame rate.

Where to look: the same version's official `examples/2d`, `examples/3d`, `examples/ui` and `examples/window/screenshot.rs`; the [Bevy UI API](https://docs.rs/bevy/0.20.1/bevy/ui/index.html).
