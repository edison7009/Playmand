# Small 2D games with Macroquad

Baseline: Macroquad 0.4.16, standalone (no Bevy). Verified by the [Macroquad example](../examples/macroquad-2d/src/lib.rs): its game rules are tested headless in CI, and CI also builds it for `wasm32-unknown-unknown`. The window itself was built and checked on macOS only.

Use Macroquad when the user asks for it, or for a small 2D game where a quick start and a light build matter more than ECS, scenes and 3D: game-jam entries, arcade toys, prototypes of one mechanic. For anything that will grow (many systems, 3D, an editor workflow, plugins), use Bevy. Never mix the two in one project.

## Shape of a game

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

- `main` is an async loop; every frame ends with `next_frame().await`. Use `#[macroquad::main(conf_fn)]` with a `fn() -> Conf` to set the window title and size.
- Immediate drawing (`draw_rectangle`, `draw_circle`, `draw_text`, `draw_texture`) in screen pixels, y down.
- Keep the rules in a plain struct (`Game::update(dt, input)`) that never calls drawing or input functions. `main.rs` reads input, updates, draws. Then the rules are unit-testable without a window, as in the example.
- Step the rules on a fixed timestep with an accumulator (`while leftover >= STEP`), and cap one frame's `dt`: then speed and collisions do not change with the frame rate or after a stall.

## Size and the web

`default-features = false` drops the audio backend (add the `audio` feature to play sounds). With `opt-level = "s"`, LTO, one codegen unit and `panic = "abort"`, the example's whole game builds to **434 KB of WebAssembly, 150 KB gzipped**: far below a Bevy web build, larger than a hand-written WebGL2 effect (see [web effects](web-wasm.md), 58 KB gzipped). Serve it with Macroquad's `mq_js_bundle.js` loader as described in its README; the build itself needs no wasm-bindgen.

## Pitfalls

- Game rules inside `main`'s loop, calling `get_frame_time()` and `is_key_down()` directly: untestable and frame-rate dependent.
- A long frame (dragging the window, a breakpoint) makes the ball jump through walls: cap `dt` and step in small fixed steps.
- Reaching for Bevy plugins: they do not work with Macroquad; pick the engine first.
