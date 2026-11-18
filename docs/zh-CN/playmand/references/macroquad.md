# 用 Macroquad 做小型 2D 游戏

基线：Macroquad 0.4.16，独立使用（不依赖 Bevy）。验证来源是 [Macroquad 示例](../../../../skills/playmand/examples/macroquad-2d/src/lib.rs)：游戏规则在 CI 中无窗口测试，CI 还会把它构建为 `wasm32-unknown-unknown`。窗口版本只在 macOS 上构建和检查过。

用户指定要用 Macroquad 时用它；或者做小型 2D 游戏、更看重快速上手和轻量构建，而不需要 ECS、场景和 3D 时：游戏 Jam 作品、街机小玩具、单一玩法原型。会不断长大的项目（很多系统、3D、编辑器流程、插件）用 Bevy。两者绝不混在同一个项目里。

## 游戏的结构

```rust
use macroquad::prelude::*;

#[macroquad::main("My game")]
async fn main() {
    let mut game = Game::default();
    loop {
        let steer = (is_key_down(KeyCode::Right) as i32 - is_key_down(KeyCode::Left) as i32) as f32;
        game.update(get_frame_time(), steer);
        clear_background(BLACK);
        draw_circle(game.ball.x, game.ball.y, 8.0, WHITE);
        next_frame().await
    }
}
```

- `main` 是一个异步循环，每帧以 `next_frame().await` 结束。用 `#[macroquad::main(conf_fn)]` 配合返回 `Conf` 的函数设置窗口标题和尺寸。
- 即时绘制（`draw_rectangle`、`draw_circle`、`draw_text`、`draw_texture`），单位是屏幕像素，y 轴向下。
- 规则写在一个普通结构体里（`Game::update(dt, input)`），里面从不调用绘制或输入函数；`main.rs` 只负责读输入、更新、绘制。这样规则不用窗口也能单元测试，就像示例那样。
- 用累加器按固定步长推进规则（`while leftover >= STEP`），并限制单帧的 `dt` 上限：速度和碰撞就不会随帧率变化，也不会在卡顿后出错。

## 体积与网页

`default-features = false` 会去掉音频后端（要播放声音时加上 `audio` feature）。配合 `opt-level = "s"`、LTO、单个 codegen unit 和 `panic = "abort"`，示例的整个游戏构建为 **434 KB 的 WebAssembly，gzip 后 150 KB**：远小于 Bevy 的网页构建，但比手写 WebGL2 的特效大（见[网页特效](web-wasm.md)，gzip 后 58 KB）。按 Macroquad README 的说明用 `mq_js_bundle.js` 加载器发布；构建本身不需要 wasm-bindgen。

## 常见坑

- 游戏规则写在 `main` 的循环里，直接调用 `get_frame_time()` 和 `is_key_down()`：无法测试，而且结果随帧率变化。
- 一帧很长（拖动窗口、断点）时球直接穿墙：限制 `dt`，用小的固定步长推进。
- 想用 Bevy 插件：它们不能用于 Macroquad；先选定引擎。
