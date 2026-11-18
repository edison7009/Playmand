# Debug panels and tools with egui

Baseline: bevy_egui 0.43.1 on Bevy 0.20.1, which uses **egui 0.36** (0.36.2 when resolved), not the newest egui. Verified by the [egui example](../examples/ui-egui/src/lib.rs), which drives egui headless with real pointer and text events in CI. The bevy_egui setup below is from the 0.43.1 source.

Use egui for developer-facing UI: debug panels, tweak sliders, inspectors, consoles, level-editor tools. Player-facing menus usually belong in Bevy UI, which follows the game's look and scales with it.

## In Bevy

```toml
[dependencies]
bevy = "=0.20.1"
bevy_egui = "=0.43.1"
```

```rust
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

App::new()
    .add_plugins((DefaultPlugins, EguiPlugin::default()))
    .add_systems(EguiPrimaryContextPass, debug_panel);

fn debug_panel(mut contexts: EguiContexts, mut settings: ResMut<DebugSettings>) -> Result {
    egui::Window::new("Debug").show(contexts.ctx_mut()?, |ui| {
        ui.checkbox(&mut settings.show_colliders, "Show colliders");
        ui.add(egui::Slider::new(&mut settings.time_scale, 0.0..=2.0).text("Time scale"));
        if ui.button("Spawn enemy").clicked() {
            settings.spawn_requested = true;
        }
    });
    Ok(())
}
```

- UI systems run in the **`EguiPrimaryContextPass`** schedule, not `Update`.
- `contexts.ctx_mut()` returns a `Result`; let the system return `Result` and use `?`.
- If the project adds its own `egui` dependency, pin it to the **same minor** bevy_egui uses (0.36 here). A second egui version compiles but its types do not match bevy_egui's.

## Immediate mode in practice

The whole UI is rebuilt every frame from your state, and each widget returns a `Response` saying what happened to it this frame: `clicked()` is true on the frame the button is released, `changed()` when a value was edited, `has_focus()` while a field has the keyboard. Bind widgets straight to the state they edit (`&mut settings.show_colliders`). Widgets in a loop need distinct ids: wrap each row in `ui.push_id(index, |ui| ...)`.

Keep the UI from stealing game input: when egui wants the pointer or keyboard (`ctx.egui_wants_pointer_input()`, `ctx.egui_wants_keyboard_input()`; older egui called them `wants_pointer_input` and `wants_keyboard_input`), skip game actions bound to clicks and keys.

## Checking it headless

Make one `egui::Context`, and per frame call `ctx.run_ui(RawInput { screen_rect, time, events, .. }, |ui| ...)`. Feed `Event::PointerMoved`, `Event::PointerButton { pressed: true }` then `{ pressed: false }` over separate frames to click, and `Event::Text` to type. Lay out once first to learn a widget's `rect`, then aim at `rect.center()`.

**Call `drop_without_applying_deltas()` on each `FullOutput`**: with no renderer the font texture update is never applied, and dropping it unapplied panics.

## Pitfalls

- UI drawn in `Update` with bevy_egui 0.43: nothing appears, or it lags a frame. Use `EguiPrimaryContextPass`.
- Every row in a list edits the first row: duplicate ids; add `push_id`.
- Clicking the panel also shoots in the game: check `egui_wants_pointer_input()`.
