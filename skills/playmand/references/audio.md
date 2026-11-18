# Audio with Kira

Baseline: Kira 0.12.6; in Bevy 0.20 through bevy_kira_audio 0.27.1. Verified by the [audio example](../examples/audio-kira/src/lib.rs), which runs Kira headless in CI on Windows and Linux. The bevy_kira_audio plugin itself is not exercised there (it needs an audio device); its setup below is from the 0.27.1 source and README.

Use Kira when a game needs more than "play a sound": music that fades between tracks, mixer tracks with their own volume (music, effects, voice), pausing everything on a menu, filters and reverb.

## In Bevy

```toml
[dependencies]
bevy = { version = "=0.20.1", default-features = false, features = ["2d", "3d", "ui"] }
bevy_kira_audio = "=0.27.1"  # ogg by default; add "mp3", "wav" or "flac" as needed
```

**Turn off Bevy's own `audio` feature**: it is on in Bevy's defaults and conflicts with bevy_kira_audio. Bevy's defaults minus `audio` are `2d`, `3d`, `ui`.

```rust
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

App::new().add_plugins((DefaultPlugins, AudioPlugin));

fn start_music(assets: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(assets.load("music/theme.ogg"))
        .looped()
        .fade_in(AudioTween::new(Duration::from_secs(2), AudioEasing::OutPowi(2)))
        .with_volume(-6.0); // decibels
}
```

Group sounds into channels (`AudioChannel<Music>`, `AudioChannel<Effects>` via `add_audio_channel::<T>()`) so a settings slider changes one channel and a pause menu pauses another.

## How Kira mixes

- Volumes are **decibels**: 0 dB is unchanged, −6 dB is about half the amplitude, `Decibels::SILENCE` (−60 dB) is silent. Sliders feel natural in dB, not in linear 0–1.
- Every change can take a `Tween` (duration and easing): fades, crossfades and ducking are tweens, not hand-written per-frame code.
- A sound plays on a track; tracks feed their parent and finally the main track. **A track runs its effects before its own volume**, so a meter, compressor or visualiser on the same track hears the signal before that track's fader. Put it on the parent (the example meters the main track).
- A one-shot sound stops by itself at its end (`PlaybackState::Stopped`); a paused sound keeps its position and resumes from there.

## Checking it without speakers

`AudioManager::<MockBackend>` with `MockBackendSettings { sample_rate }` runs the whole mixer with no sound card: call `backend_mut().on_start_processing()` and `backend_mut().process()` to advance one block (`internal_buffer_size` frames). Generate test tones from `StaticSoundData { frames, .. }` instead of loading files, attach a small `Effect` that records levels, and assert what a player would hear: levels, fades, silence, positions. Judge the real mix by ear in the game.

## Pitfalls

- No sound and no error: Bevy's `audio` feature still on, the asset's format feature (`mp3`, `wav`, `flac`) not enabled, or the browser waiting for a user gesture before audio may start (web builds).
- Volume set in linear units: `0.5` is not half volume in Kira; use decibels.
- Restarting music every time a state is entered: keep the handle, check whether it is already playing.
