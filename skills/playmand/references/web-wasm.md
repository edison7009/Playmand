# Rust → WebAssembly effects inside a web or Tauri app

Verified by building the card pack, holographic cards and voxel pets in this repository's [`web/`](https://github.com/edison7009/Playmand/tree/main/web) (live: <https://playmand.com/>), first shipped inside the EchoBird desktop app. Rust 1.93, wasm-bindgen 0.2.127, web-sys 0.3.104, WebGL2; checked in Chromium-based browsers on macOS. Windows WebView2 and Linux WebKitGTK were not checked.

Use this when the effect lives inside an existing page or React/Tauri window: a reward animation, a 3D companion, a product showcase. Bevy is still the default for a game or a recorded video; here a whole engine would be too heavy, and the host page already owns layout, text and accessibility.

## Shape of the crate

- `crate-type = ["cdylib", "rlib"]`. Put **pure logic** (timing, springs, layout, models, poses, colour maths) in modules that compile natively, and gate the WebGL side with `#[cfg(target_arch = "wasm32")]`. Then `cargo test` runs on any machine and CI needs no browser.
- One `#[wasm_bindgen]` struct per scene with a small API: `new(canvas)`, `resize(w, h, dpr)`, pointer calls, `frame(now_ms) -> u32`, `dispose()`. Keep every exported method in **one** `#[wasm_bindgen] impl` block; move helpers to a plain `impl`, or methods silently stop being exported.
- `frame` returns **event bits** (tear started, cards ready, flip midpoint, done…). The page reacts to bits instead of guessing with timers, so sound and UI stay in sync at any frame rate.
- Anything that must happen inside the user's gesture (unlocking audio) is returned synchronously from the pointer call, e.g. `timing() -> [cue_ms, ready_ms]`, not from a later frame.
- Render on demand when nothing moves; run `requestAnimationFrame` only while animating, and stop it when `document.hidden` or the canvas scrolls out of view. Free GPU objects in `dispose()` and call wasm-bindgen's `free()`.
- Text the user reads (names, amounts, labels) comes from the page, drawn into a 2D canvas and uploaded as a texture, so translation stays in the host app.

## Making it look real

| Effect | How |
| --- | --- |
| Card thickness | A band of triangles around the rounded rectangle between the two faces. Rebuild its normal in the vertex shader as `normalize(p.xy - clamp(p.xy, -(half - r), half - r))`; no normal attribute needed. |
| Clear coat | Reflect the view vector off the face and look up two soft "studio window" bands in reflection space; they slide across as the card tilts. Add a hot spot where the pointer is, a thin bright rim line, a top-edge highlight and a darker bottom. Blend screen-like so it still reads on bright faces. |
| Depth | A soft rounded-rectangle shadow behind each card (drawn first, depth writes off), a pressed-in art window (inner shadow in the 2D texture), the character on its own layer above the face with a blurred silhouette shadow. |
| Hover | Lean the card so the corner under the pointer **presses away** (about ±17° pitch, ±19° yaw), on a spring; idle cards waft a few degrees. For a face rotated by π (a card back), the pitch sign flips. Halve tilt and stop the waft under reduced motion. |
| Coming out of a slit | Clip the card in the fragment shader against a world-space plane through the opening, moving with the pack. Drawing the card "in front" when it rises makes it look like it came from below. |
| Voxels | Cull shared faces, add per-vertex ambient occlusion (and flip the quad diagonal by AO), and flag real outer edges for a thin highlight that fades with `fwidth` at small sizes. |
| Matching three.js | Lambert light is divided by π, colours are linear (convert hex with the sRGB curve), output goes through the sRGB transfer. Euler order XYZ is `Rx·Ry·Rz`. |

## Porting from JavaScript exactly

Dump the original's output (every voxel, pivot, generated track) to text in Node, hash it (FNV-1a is enough) and assert the same hash in a Rust test. Match JavaScript on purpose: f64, `Math.round` rounds halves up (`(x + 0.5).floor()`), and `Map` keeps insertion order, so a re-inserted key moves to the end. The voxel pets here matched 9,390 lines on the first run this way.

## Artwork from Bevy

Model characters as Bevy meshes, render each layer (body, eyes, expressions) twice with an orthographic camera, on black and on white, and recover exact alpha from the difference. Ship small WebP layers; the browser composites them, so eyes can blink and look around.

## First paint

Show a CSS loading state at once, preload the module and the `.wasm` (`modulepreload`, `preload as=fetch crossorigin`), wait only for the art the first frame needs, load the rest in the background, and draw the first frame synchronously instead of waiting for `requestAnimationFrame`. Run exactly one animation loop.

## Checking it

- Add a fixed-clock test mode: the page stops its own loop and a script calls `frame(t)` with `t += 16.7`, so every capture is repeatable. Embedded browser panes may throttle `requestAnimationFrame` to a few frames per second; do not judge timing from them.
- Capture the canvas at full resolution with `canvas.toDataURL()` right after `frame`, in the same task (no `preserveDrawingBuffer` needed), and post it to a tiny local server that saves PNGs. Pane screenshots are too small to judge edges and highlights.
- Compare against reference renders side by side at the same moment; check every variant (each front, each species, each stage) and both pointer extremes.

## Size

Release build with `opt-level = "s"`, LTO, one codegen unit and `panic = "abort"` (set as `CARGO_PROFILE_RELEASE_*` environment variables in the build script if the workspace profile must not change), then `wasm-bindgen --remove-name-section --remove-producers-section`. That took this demo from 194 KB to 134 KB; the live card pack page ships 148 KB of WebAssembly plus 34 KB of JS, 66 KB together gzipped. The voxel pets sit behind a Cargo feature (`--features pets`); with them it is 235 KB (92 KB gzipped), against 516 KB (133 KB gzipped) for the three.js scene they replaced. Keep optional scenes behind features so each page ships only what it draws.

## Pitfalls met along the way

- A grid generated top to bottom winds clockwise: set `front_face(CW)` for it or the back face draws.
- `img.decode()` can hang in a hidden pane; wait for `onload` instead.
- `CanvasRenderingContext2d::letterSpacing` is missing from web-sys 0.3.104: set it through `js_sys::Reflect`. `measure_text` needs the `TextMetrics` feature.
- Bundlers targeting ES2020 reject top-level await: export a `load()` promise and await it before creating the scene.
- Screenshot tools fire `pointerleave`, which can cancel a scripted drag; drive the test mode from script instead.
