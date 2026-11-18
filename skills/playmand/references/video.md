# Animation and video recording

Baseline: Bevy 0.20.1. Window capture was verified by recording three scenes from the repository's `demos/` (a pelican on a bicycle, an ECS galaxy and particle icons) on macOS (Metal), including 1920×1080 / 30 fps. Offscreen capture, the soundtrack and MP4 encoding with audio were verified by the [motion-video example](../examples/motion-video/src/bin/render.rs) in 16:9, 9:16 and 1:1. Encoding with ffmpeg and the Blender render path were not run in that verification. For planning, captions, sound and formats of a video people will watch, see [Motion graphics and video production](motion-graphics.md).

Making a video with Bevy means writing the scene as a function of "time → image" and rendering it frame by frame offline. The only difference from a game is the clock: a game follows real time and input, a recording follows a fixed frame number.

## Offscreen rendering (recommended for video)

Render into an image instead of a window, read each frame back once, and advance the clock only after that frame is on disk. Nothing can cover the output, it needs no display, and it was the fastest path in the example (270 frames of 1920×1080 in about 4 seconds on an Apple M4).

```rust
use bevy::{app::ScheduleRunnerPlugin, camera::RenderTarget, prelude::*, window::ExitCondition, winit::WinitPlugin};
use bevy::render::{gpu_readback::{ReadbackComplete, ReadbackOnce}, render_resource::{TextureFormat, TextureUsages}};

App::new()
    .add_plugins(DefaultPlugins
        .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
        .disable::<WinitPlugin>())
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO));

// Setup: the camera draws into an image the GPU may copy from.
let mut image = Image::new_target_texture(w, h, TextureFormat::Rgba8UnormSrgb, None);
image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
let target = images.add(image);
commands.spawn((Camera2d, RenderTarget::Image(target.clone().into())));

// Each frame, after the scene was posed for frame `n` (one readback in flight):
commands.spawn(ReadbackOnce::texture(target.clone()))
    .observe(move |ev: On<ReadbackComplete>, mut rec: ResMut<Rec>| {
        let pixels = unpad_rows(&ev.data, w, h, 4);   // see below
        save_png(&rec.dir, n, &pixels);
        rec.frame = n + 1;                             // only now does the clock move
        rec.in_flight = false;
    });
```

- **Row padding**: the readback pads every row to a multiple of 256 bytes. 1920×4 = 7680 bytes needs none, but 1080×4 = 4320 is padded to 4352, so 9:16 and square frames come out skewed unless you copy `width × 4` bytes out of each padded row.
- **Warm-up**: render about 30 updates before the first capture so pipelines, fonts and the glyph atlas are ready.
- The bytes are sRGB RGBA when the target is `Rgba8UnormSrgb`; write them straight to PNG.
- Text with a CSS family list needs Bevy's `system_font_discovery` feature (and fontconfig on Linux); a font file loaded by handle does not.

## Window capture, one screenshot per frame

Use this when you need the real window (a game's own recording mode).


Animation systems read one clock resource and never read `Time` directly:

```rust
#[derive(Resource, Default)]
struct Clock { t: f32 }

// While recording: the clock is the number of the next frame to capture.
fn step_clock(rec: Res<Recording>, mut clock: ResMut<Clock>) {
    clock.t = rec.frame as f32 / rec.fps as f32;
}
```

For interactive previews, write the same `Clock` from `time.elapsed_secs() % loop_seconds`. The same animation code can then be watched live or exported frame by frame, and every machine records identical frames.

In `Last`, request a screenshot named after the frame number, and advance the frame only after the previous capture is done:

```rust
use bevy::render::view::screenshot::{Capturing, Screenshot, save_to_disk};

fn capture(
    mut commands: Commands,
    mut rec: ResMut<Recording>,
    pending: Query<(), Or<(With<Screenshot>, With<Capturing>)>>,
    mut exit: MessageWriter<AppExit>,
) {
    rec.update += 1;
    if rec.update <= rec.warmup || !pending.is_empty() {
        return; // warming up, or the previous screenshot is still in flight
    }
    if rec.frame < rec.frames {
        let path = rec.dir.join(format!("frame-{:04}.png", rec.frame));
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        rec.frame += 1;
    } else {
        exit.write(AppExit::Success);
    }
}
```

Key points:

- **Warm-up**: skip screenshots for the first updates (90 in the demos) while pipelines compile and meshes and materials become ready, so the clip does not start with blank frames.
- **Exit after the last capture**: the screenshot entity is despawned once its image is read back. Exit only when the `pending` query is empty; a returned request does not mean the file has been written.
- **Exact pixels**: create the window with `WindowResolution::new(960, 540).with_scale_factor_override(1.0)`, otherwise HiDPI screens output twice the size.
- **Readback cost**: the [rendering guide](rendering.md) warns against screenshots every frame in a game. Offline recording can capture every frame because the image depends only on the frame number, so a slower recording does not change the result. Keeping a single screenshot in flight, as above, stops heavy scenes (high resolution, bloom) from piling up readbacks and returning black frames.
- **High-resolution output**: keep the scene's logical size and use `WindowResolution::new(1920, 1080).with_scale_factor_override(2.0)` so a 960×540 layout renders at 2x with text and layout unchanged.
- **Black frames from a covered window (macOS)**: once the recording window is covered by another window, or after a long capture, it stops presenting and later screenshots are all black. Set `window_level: WindowLevel::AlwaysOnTop` while recording, count blank frames afterwards (for example Pillow's `convert("L").getbbox() is None`), and if there are any, delete from the first bad frame and resume with `frame` starting there.
- Recording opens a real window and uses the GPU; an environment without a GPU cannot check images this way.

## Seamless loops

README animations and social clips usually need a seamless loop. Give every periodic motion a whole number of cycles in one loop of length `L`:

- Rotation: angle `= 2π · k · t / L` with integer k (the galaxy's inner, middle and outer rings use 3, 2 and 1 turns).
- Scrolling background: a pattern of period `P` and a speed where `speed · L = P`, positioned at `base - (speed · t) % P`.
- A wheel rolling on a road: wheel angle `= -speed · t / r`; with 8 equal spokes, `speed · L / r` must be a multiple of `π/4`.
- Use a deterministic RNG with a fixed seed for random layouts, so every run builds the same scene.

Record `L · fps` frames, numbered 0 to `L · fps - 1`; frame `L · fps` itself equals frame 0.

## Encoding

- **Animated images (README, web)**: Pillow can write a looping WebP or GIF directly, with no ffmpeg. WebP is clearly better than GIF in quality and size for glow and gradients; GIF is acceptable for flat-colour scenes. At 720×405, 25 fps and 4–6 seconds, WebP is about 1–2.5 MB.
- **Video (MP4, macOS without ffmpeg)**: the skill's [`scripts/encode_mp4.swift`](../scripts/encode_mp4.swift) (`swiftc -O`) encodes PNG frames to H.264 with AVFoundation, adds a WAV soundtrack as AAC with `--audio`, repeats a loop with `--repeat`, and checks a finished file with `--verify`. 12–16 Mbps is enough for 1080p30.
- **Video (MP4, ffmpeg)**: with ffmpeg, `ffmpeg -framerate 25 -i frame-%04d.png -c:v libx264 -pix_fmt yuv420p -crf 18 out.mp4`; add `-i soundtrack.wav -c:a aac -shortest` for sound. `yuv420p` keeps it playable in browsers and on social platforms; width and height must be even.
- Scale before encoding with a high-quality filter such as LANCZOS; for glowing scenes, compare dark-area banding before lowering the quality setting.

## Visual effects

- 2D bloom: add `Bloom`, `Tonemapping::TonyMcMapface` and `DebandDither::Enabled` to the camera, and give glowing objects HDR colours with components above 1. Based on that version's official `examples/2d/bloom_2d.rs`.
- 2D characters built from shapes: combine primitives such as `Rectangle`, `Circle`, `Ellipse`, `Triangle2d`, `Annulus` and `Capsule2d`; place bars as unit-width rectangles scaled and rotated into position, with circles at both ends for rounded joints. When limbs follow a target, solve the knee/elbow position with two-bone IK.
- Many identical objects: share the mesh and bucket materials by colour so they batch well; update them each frame with `par_iter_mut`.

## Verification

- Open the actual first, middle and last frames and check composition, occlusion and blank frames; "the recording command succeeded" does not replace looking at the image.
- Check frame count, size, frame rate and the loop flag (`loop=0` for animated images).
- Compare the last frame with the first to make sure there is no jump at the loop point.
- Preview the animation on the target page (such as a GitHub README) to confirm it autoplays and the size fits.

## Blender render path

For cinematic lighting, materials or complex character animation, use a connected Blender MCP to set up camera paths and keyframes and render an image sequence or video; see [Blender and MCP](blender-mcp.md) for the workflow and its limits. Real-time, procedural effects, or effects that share code with the game, are better done in Bevy.
