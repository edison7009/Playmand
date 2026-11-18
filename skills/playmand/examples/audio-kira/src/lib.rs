//! Headless Kira audio recipes. A mock backend stands in for the sound card,
//! and a small effect on the main track records what reaches the output, so
//! each test checks what a player would hear: levels, fades, pauses, endings.
//!
//! A track runs its effects before its own volume, so a meter must sit on the
//! track the sound feeds into (here the main track), not on the same track.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use kira::{
        AudioManager, AudioManagerSettings, Decibels, Frame, Tween,
        backend::mock::{MockBackend, MockBackendSettings},
        effect::{Effect, EffectBuilder},
        info::Info,
        sound::PlaybackState,
        sound::static_sound::{StaticSoundData, StaticSoundSettings},
        track::{MainTrackBuilder, TrackBuilder},
    };

    /// 1000 frames per second and 100 frames per block: one block is 0.1 s.
    const RATE: u32 = 1000;
    const BLOCK: usize = 100;

    /// Records the loudest sample of every block that passes through a track.
    struct Meter(Arc<Mutex<Vec<f32>>>);
    impl Effect for Meter {
        fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
            let peak = input
                .iter()
                .map(|f| f.left.abs().max(f.right.abs()))
                .fold(0.0, f32::max);
            self.0.lock().unwrap().push(peak);
        }
    }
    struct MeterBuilder(Arc<Mutex<Vec<f32>>>);
    impl EffectBuilder for MeterBuilder {
        type Handle = ();
        fn build(self) -> (Box<dyn Effect>, ()) {
            (Box::new(Meter(self.0)), ())
        }
    }

    type Levels = Arc<Mutex<Vec<f32>>>;

    /// A manager whose main track meters everything the player would hear.
    fn manager() -> (AudioManager<MockBackend>, Levels) {
        let levels = Levels::default();
        let manager = AudioManager::new(AudioManagerSettings {
            internal_buffer_size: BLOCK,
            backend_settings: MockBackendSettings { sample_rate: RATE },
            main_track_builder: MainTrackBuilder::new().with_effect(MeterBuilder(levels.clone())),
            ..Default::default()
        })
        .unwrap();
        (manager, levels)
    }

    /// Advances the mock sound card by `blocks` blocks.
    fn run(manager: &mut AudioManager<MockBackend>, blocks: usize) {
        for _ in 0..blocks {
            manager.backend_mut().on_start_processing();
            manager.backend_mut().process();
        }
    }

    /// A steady tone (constant level) lasting `seconds`.
    fn tone(seconds: f32, settings: StaticSoundSettings) -> StaticSoundData {
        let frames = vec![Frame::from_mono(0.5); (seconds * RATE as f32) as usize];
        StaticSoundData {
            sample_rate: RATE,
            frames: frames.into(),
            settings,
            slice: None,
        }
    }

    /// A sub-track (say, music or effects) at `volume`, feeding the main track.
    fn track(
        manager: &mut AudioManager<MockBackend>,
        volume: Decibels,
    ) -> kira::track::TrackHandle {
        manager
            .add_sub_track(TrackBuilder::new().volume(volume))
            .unwrap()
    }

    fn last(levels: &Levels) -> f32 {
        *levels.lock().unwrap().last().unwrap()
    }

    #[test]
    fn minus_six_decibels_on_a_track_halves_the_level() {
        let (mut full, la) = manager();
        let mut a = track(&mut full, Decibels::IDENTITY);
        a.play(tone(2.0, StaticSoundSettings::new())).unwrap();
        run(&mut full, 5);

        let (mut quieter, lb) = manager();
        let mut b = track(&mut quieter, Decibels(-6.0));
        b.play(tone(2.0, StaticSoundSettings::new())).unwrap();
        run(&mut quieter, 5);

        let ratio = last(&lb) / last(&la);
        assert!((ratio - 0.501).abs() < 0.01, "-6 dB gave {ratio:.3}x");
    }

    #[test]
    fn a_fade_in_rises_steadily_and_reaches_full_level() {
        let (mut m, levels) = manager();
        let mut track = track(&mut m, Decibels::IDENTITY);
        let fade = Tween {
            duration: Duration::from_secs(1),
            ..Default::default()
        };
        track
            .play(tone(3.0, StaticSoundSettings::new().fade_in_tween(fade)))
            .unwrap();
        run(&mut m, 16);
        let l = levels.lock().unwrap().clone();
        let rising = &l[1..10];
        assert!(
            rising.windows(2).all(|w| w[1] >= w[0]),
            "fade-in went down: {rising:?}"
        );
        assert!(l[1] < 0.6 * l[15], "starts quiet");
        assert!((l[14] - l[15]).abs() < 1e-4, "settled at full level");
    }

    #[test]
    fn fading_a_music_track_out_reaches_silence() {
        let (mut m, levels) = manager();
        let mut track = track(&mut m, Decibels::IDENTITY);
        track.play(tone(5.0, StaticSoundSettings::new())).unwrap();
        run(&mut m, 3);
        let before = last(&levels);
        track.set_volume(
            Decibels::SILENCE,
            Tween {
                duration: Duration::from_millis(500),
                ..Default::default()
            },
        );
        run(&mut m, 8);
        assert!(before > 0.1);
        assert!(last(&levels) < 0.001, "still audible: {}", last(&levels));
    }

    #[test]
    fn pausing_holds_the_position_and_resuming_continues() {
        let (mut m, levels) = manager();
        let mut track = track(&mut m, Decibels::IDENTITY);
        let mut sound = track.play(tone(5.0, StaticSoundSettings::new())).unwrap();
        run(&mut m, 5);
        sound.pause(Tween::default());
        run(&mut m, 2);
        let held = sound.position();
        assert_eq!(sound.state(), PlaybackState::Paused);
        run(&mut m, 5);
        assert_eq!(sound.position(), held, "paused sound kept moving");
        assert!(last(&levels) < 1e-6, "paused sound still audible");
        sound.resume(Tween::default());
        run(&mut m, 3);
        assert!(sound.position() > held);
        assert!(last(&levels) > 0.1);
    }

    #[test]
    fn a_one_shot_sound_stops_by_itself_when_it_ends() {
        let (mut m, _) = manager();
        let mut track = track(&mut m, Decibels::IDENTITY);
        let sound = track.play(tone(0.25, StaticSoundSettings::new())).unwrap();
        run(&mut m, 1);
        assert_eq!(sound.state(), PlaybackState::Playing);
        run(&mut m, 5);
        assert_eq!(sound.state(), PlaybackState::Stopped);
    }
}
