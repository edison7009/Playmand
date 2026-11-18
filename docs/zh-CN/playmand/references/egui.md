# 用 egui 做调试面板和工具

基线：bevy_egui 0.43.1、Bevy 0.20.1，对应 **egui 0.36**（实际解析为 0.36.2），不是最新的 egui。验证来源是 [egui 示例](../../../../skills/playmand/examples/ui-egui/src/lib.rs)，它在 CI 中无窗口运行 egui，并输入真实的指针和文字事件。下面的 bevy_egui 接入方式来自 0.43.1 的源码。

egui 适合给开发者用的界面：调试面板、调参滑杆、检查器、控制台、关卡编辑工具。面向玩家的菜单通常用 Bevy UI，它能跟游戏的画面风格和缩放保持一致。

## 在 Bevy 中

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

- 界面系统运行在 **`EguiPrimaryContextPass`** 调度阶段，不是 `Update`。
- `contexts.ctx_mut()` 返回 `Result`；让系统返回 `Result` 并使用 `?`。
- 项目如果自己也依赖 `egui`，要锁到和 bevy_egui **相同的次版本**（这里是 0.36）。第二个 egui 版本能编译通过，但类型和 bevy_egui 的对不上。

## 即时模式怎么用

每一帧都根据你的状态重建整个界面，每个控件返回一个 `Response`，说明这一帧它发生了什么：按钮松开的那一帧 `clicked()` 为真，数值被修改时 `changed()` 为真，输入框拿着键盘焦点时 `has_focus()` 为真。控件直接绑定它要修改的状态（`&mut settings.show_colliders`）。循环里的控件需要不同的 id：每一行用 `ui.push_id(index, |ui| ...)` 包起来。

别让界面抢走游戏输入：egui 需要指针或键盘时（`ctx.egui_wants_pointer_input()`、`ctx.egui_wants_keyboard_input()`；旧版 egui 叫 `wants_pointer_input` 和 `wants_keyboard_input`），跳过绑定在点击和按键上的游戏操作。

## 无窗口检查

创建一个 `egui::Context`，每帧调用 `ctx.run_ui(RawInput { screen_rect, time, events, .. }, |ui| ...)`。分几帧依次输入 `Event::PointerMoved`、`Event::PointerButton { pressed: true }`、`{ pressed: false }` 来模拟点击，用 `Event::Text` 模拟键入。先排版一帧拿到控件的 `rect`，再对准 `rect.center()`。

**每个 `FullOutput` 都要调用 `drop_without_applying_deltas()`**：没有渲染器时字体纹理更新永远不会被应用，未应用就丢弃会 panic。

## 常见坑

- 用 bevy_egui 0.43 时在 `Update` 里画界面：什么都不显示，或者慢一帧。改用 `EguiPrimaryContextPass`。
- 列表里每一行都在改第一行：id 重复了，加上 `push_id`。
- 点面板时游戏里也开了枪：检查 `egui_wants_pointer_input()`。
