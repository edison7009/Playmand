//! A short motion-graphics video described as data: shots with captions,
//! sound cues and a brand kit. Everything here is plain Rust and testable
//! without a GPU; `src/bin/render.rs` draws it with Bevy, offscreen, one
//! frame at a time, for any aspect ratio.
//!
//! The same timeline drives the picture, the captions (SRT) and the audio
//! (WAV), so they cannot drift apart.

use std::f32::consts::PI;
use std::fmt::Write;

// ---------------------------------------------------------------- formats

/// An output format. Lay out against `safe()`, not the full frame: platforms
/// draw their own buttons and captions over the edges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Format {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    /// Fractions of the frame to keep clear: left, top, right, bottom.
    pub margins: [f32; 4],
}

pub const LANDSCAPE: Format = Format {
    name: "landscape",
    width: 1920,
    height: 1080,
    margins: [0.05, 0.05, 0.05, 0.05],
};
/// 9:16 for Shorts, Reels and TikTok: the app's buttons sit on the right and
/// its caption and account name over the bottom fifth.
pub const PORTRAIT: Format = Format {
    name: "portrait",
    width: 1080,
    height: 1920,
    margins: [0.06, 0.12, 0.14, 0.22],
};
pub const SQUARE: Format = Format {
    name: "square",
    width: 1080,
    height: 1080,
    margins: [0.06, 0.06, 0.06, 0.08],
};
pub const FORMATS: [Format; 3] = [LANDSCAPE, PORTRAIT, SQUARE];

/// A rectangle in pixels, origin at the frame's centre, y up (Bevy 2D).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }
    pub fn center_x(&self) -> f32 {
        (self.left + self.right) / 2.0
    }
    pub fn center_y(&self) -> f32 {
        (self.bottom + self.top) / 2.0
    }
}

impl Format {
    pub fn portrait(&self) -> bool {
        self.height > self.width
    }

    /// The area text and key visuals must stay inside.
    pub fn safe(&self) -> Rect {
        let (w, h) = (self.width as f32, self.height as f32);
        let [l, t, r, b] = self.margins;
        Rect {
            left: -w / 2.0 + l * w,
            right: w / 2.0 - r * w,
            top: h / 2.0 - t * h,
            bottom: -h / 2.0 + b * h,
        }
    }

    /// One type scale for every format: sizes follow the short side, so a
    /// title is as readable on a phone as on a TV. 9:16 is watched full-screen
    /// on a phone, where the same pixel size reads smaller, so it gets 1.3x.
    pub fn unit(&self) -> f32 {
        let base = self.width.min(self.height) as f32 / 1080.0;
        if self.portrait() { base * 1.3 } else { base }
    }
}

// ---------------------------------------------------------------- brand

/// Brand kit: colours as sRGB bytes, plus the copy the video may use.
#[derive(Clone, Copy, Debug)]
pub struct Brand {
    pub paper: [u8; 3],
    pub ink: [u8; 3],
    pub soft: [u8; 3],
    pub line: [u8; 3],
    pub accent: [u8; 3],
    pub name: &'static str,
    pub url: &'static str,
}

pub const PLAYMAND: Brand = Brand {
    paper: [0xf1, 0xee, 0xe8],
    ink: [0x24, 0x22, 0x1f],
    soft: [0x77, 0x71, 0x6a],
    line: [0xd9, 0xd4, 0xcb],
    accent: [0xd9, 0x57, 0x2b],
    name: "Playmand",
    url: "playmand.com",
};

// ---------------------------------------------------------------- easing

/// 0..1 progress of `t` through `[start, start + len]`.
pub fn progress(t: f32, start: f32, len: f32) -> f32 {
    ((t - start) / len).clamp(0.0, 1.0)
}

pub fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
    }
}

/// Overshoots a little and settles: for things that pop in.
pub fn ease_out_back(x: f32) -> f32 {
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

// ---------------------------------------------------------------- timeline

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotKind {
    Title,
    Chart,
    EndCard,
}

#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub kind: ShotKind,
    pub start: f32,
    pub len: f32,
    pub caption: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// A soft noise sweep on a cut.
    Whoosh,
    /// A short tick as each bar lands.
    Tick,
    /// A two-note chime for the end card.
    Chime,
    /// A soft low thump when the title lands.
    Pop,
}

/// Title letters rise one after another, this far apart.
pub const TITLE_STAGGER: f32 = 0.055;
/// How long each letter takes to rise and settle.
pub const TITLE_RISE: f32 = 0.55;
/// When the underline snaps out and the particles burst (the "pop").
pub const TITLE_BURST: f32 = 0.85;
/// How long a cut's wipe takes, and the band's width in frame widths.
pub const WIPE: f32 = 0.7;
pub const WIPE_BAND: f32 = 0.32;
/// Length of the whoosh on a cut.
pub const WHOOSH: f32 = 0.35;

/// A small deterministic random generator: the same seed gives the same
/// particles and layout on every machine and every render.
#[derive(Clone, Copy, Debug)]
pub struct Rng(u32);

impl Rng {
    /// Neighbouring seeds (100, 101, 102...) would give nearly the same first
    /// numbers, so the seed is scrambled first.
    pub fn new(seed: u32) -> Self {
        let mut r = Rng((seed.wrapping_mul(2_654_435_761) ^ 0x9E37_79B9).max(1));
        for _ in 0..4 {
            r.next();
        }
        r
    }
    /// 0..1
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// One particle of a burst, `age` seconds after it: offset from the burst
/// point (in units of `reach`), opacity and size. Thrown outward with drag and
/// a little gravity; gone by `life`.
pub fn particle(seed: u32, age: f32, life: f32) -> Option<([f32; 2], f32, f32)> {
    if !(0.0..life).contains(&age) {
        return None;
    }
    let mut r = Rng::new(seed.wrapping_mul(2_654_435_761).wrapping_add(1));
    let angle = r.next() * 2.0 * PI;
    let speed = 0.6 + 0.4 * r.next();
    let x = age / life;
    // Drag: fast at first, settling; gravity pulls down a little at the end.
    let travel = speed * (1.0 - (1.0 - x).powi(3));
    let offset = [angle.cos() * travel, angle.sin() * travel - 0.25 * x * x];
    let size = 0.5 + 0.5 * r.next();
    Some((offset, (1.0 - x).powi(2), size * (1.0 - 0.5 * x)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cue {
    pub at: f32,
    pub sound: Sound,
}

/// One bar of the chart: label and value.
pub const BARS: [(&str, f32); 4] = [
    ("Rust", 1.04),
    ("C++", 1.56),
    ("C#", 3.14),
    ("JavaScript", 6.52),
];

/// When bar `i` starts growing, in seconds after the chart shot starts.
pub fn bar_start(i: usize) -> f32 {
    0.35 + 0.28 * i as f32
}
pub const BAR_GROW: f32 = 0.6;

/// Cross-fade between shots, in seconds.
pub const FADE: f32 = 0.35;

#[derive(Clone, Debug)]
pub struct Timeline {
    pub fps: u32,
    pub shots: Vec<Shot>,
}

impl Timeline {
    /// The sample: a title, a data story and an end card, 9 seconds.
    pub fn promo(fps: u32) -> Self {
        Timeline {
            fps,
            shots: vec![
                Shot {
                    kind: ShotKind::Title,
                    start: 0.0,
                    len: 3.0,
                    caption: "Games and visual work, in Rust.",
                },
                Shot {
                    kind: ShotKind::Chart,
                    start: 3.0,
                    len: 3.5,
                    caption: "Rust runs at C speed: 1.04x C's time.",
                },
                Shot {
                    kind: ShotKind::EndCard,
                    start: 6.5,
                    len: 2.5,
                    caption: "A skill for your AI coding agent.",
                },
            ],
        }
    }

    pub fn duration(&self) -> f32 {
        self.shots
            .iter()
            .map(|s| s.start + s.len)
            .fold(0.0, f32::max)
    }

    pub fn frames(&self) -> u32 {
        (self.duration() * self.fps as f32).round() as u32
    }

    /// Time of frame `n`. Every frame depends only on its number.
    pub fn time(&self, frame: u32) -> f32 {
        frame as f32 / self.fps as f32
    }

    /// How visible each shot is at `t` (0..1). A cut is a cross-fade: the
    /// next shot fades in over the same `FADE` seconds the current one fades
    /// out, ending at the cut, so the two always add up to one and the frame
    /// never dips to the background. The last shot holds to the end.
    pub fn opacity(&self, index: usize, t: f32) -> f32 {
        let s = &self.shots[index];
        let end = s.start + s.len;
        let last = index + 1 == self.shots.len();
        let fade_in = if index == 0 {
            1.0
        } else {
            progress(t, s.start - FADE, FADE)
        };
        let fade_out = if last {
            1.0
        } else {
            1.0 - progress(t, end - FADE, FADE)
        };
        // The eased curve is symmetric (f(x) + f(1 - x) = 1), so overlapping
        // fades still add up to one.
        ease_in_out_cubic(fade_in.min(fade_out))
    }

    /// The caption on screen at `t`: captions cut, they never cross-fade
    /// (two half-visible lines are unreadable), and they follow the SRT times.
    pub fn caption_at(&self, t: f32) -> Option<usize> {
        let last = self.shots.len() - 1;
        self.shots
            .iter()
            .position(|s| t >= s.start && t < s.start + s.len)
            .or((t >= self.duration() - 1e-4).then_some(last))
    }

    /// Every sound and when it plays. Derived from the same numbers as the
    /// picture, so a bar's tick lands on the frame the bar arrives.
    pub fn cues(&self) -> Vec<Cue> {
        let mut cues = Vec::new();
        for (i, s) in self.shots.iter().enumerate() {
            if i > 0 {
                // The whoosh swells and fades over 0.35 s: start it early so
                // its peak lands on the cut, with the wipe band mid-frame.
                cues.push(Cue {
                    at: s.start - WHOOSH / 2.0,
                    sound: Sound::Whoosh,
                });
            }
            match s.kind {
                ShotKind::Chart => {
                    for b in 0..BARS.len() {
                        cues.push(Cue {
                            at: s.start + bar_start(b) + BAR_GROW,
                            sound: Sound::Tick,
                        });
                    }
                }
                ShotKind::EndCard => cues.push(Cue {
                    at: s.start + 0.5,
                    sound: Sound::Chime,
                }),
                ShotKind::Title => cues.push(Cue {
                    at: s.start + TITLE_BURST,
                    sound: Sound::Pop,
                }),
            }
        }
        cues.sort_by(|a, b| a.at.total_cmp(&b.at));
        cues
    }

    /// The colour band that sweeps across the frame on each cut, as left and
    /// right edges in frame widths (-0.5 is the left edge, 0.5 the right).
    /// Its centre crosses the middle exactly at the cut, while the shots
    /// cross-fade underneath; it starts and ends fully off-screen.
    pub fn wipe(&self, t: f32) -> Option<(f32, f32)> {
        self.shots.iter().skip(1).find_map(|s| {
            let p = progress(t, s.start - WIPE / 2.0, WIPE);
            if p <= 0.0 || p >= 1.0 {
                return None;
            }
            let left = -0.5 - WIPE_BAND + (1.0 + WIPE_BAND) * ease_in_out_cubic(p);
            Some((left, left + WIPE_BAND))
        })
    }

    /// Captions as SubRip (.srt), one per shot.
    pub fn srt(&self) -> String {
        let mut out = String::new();
        for (i, s) in self.shots.iter().enumerate() {
            let _ = write!(
                out,
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                srt_time(s.start),
                srt_time(s.start + s.len),
                s.caption
            );
        }
        out
    }
}

/// `HH:MM:SS,mmm`, the SubRip timestamp.
pub fn srt_time(seconds: f32) -> String {
    let ms = (seconds as f64 * 1000.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02},{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

// ---------------------------------------------------------------- audio

/// Renders the soundtrack: a quiet music bed plus every cue, mono, then
/// normalizes the peak to -1 dBFS so nothing clips.
pub fn render_audio(timeline: &Timeline, rate: u32) -> Vec<f32> {
    let n = (timeline.duration() * rate as f32).round() as usize;
    let mut out = vec![0.0f32; n];
    let r = rate as f32;
    // Music bed: a soft chord that fades in and out with the video.
    let chord = [220.0, 277.18, 329.63];
    let dur = timeline.duration();
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / r;
        let env = progress(t, 0.0, 0.8).min(1.0 - progress(t, dur - 1.0, 1.0));
        let pad: f32 = chord.iter().map(|f| (2.0 * PI * f * t).sin()).sum::<f32>() / 3.0;
        *s = 0.12 * env * pad;
    }
    // Cues, mixed in at their exact sample.
    let mut noise = 0x1234_5678u32;
    for cue in timeline.cues() {
        let start = (cue.at * r).round() as usize;
        let (len, gain) = match cue.sound {
            Sound::Whoosh => (WHOOSH, 0.35),
            Sound::Tick => (0.05, 0.5),
            Sound::Chime => (1.2, 0.45),
            Sound::Pop => (0.25, 0.6),
        };
        for k in 0..(len * r) as usize {
            let Some(s) = out.get_mut(start + k) else {
                break;
            };
            let t = k as f32 / r;
            let x = t / len;
            let v = match cue.sound {
                Sound::Whoosh => {
                    noise ^= noise << 13;
                    noise ^= noise >> 17;
                    noise ^= noise << 5;
                    let white = noise as f32 / u32::MAX as f32 * 2.0 - 1.0;
                    white * (PI * x).sin()
                }
                Sound::Tick => (2.0 * PI * 1800.0 * t).sin() * (1.0 - x).powi(3),
                Sound::Chime => {
                    let second = if t > 0.12 {
                        (2.0 * PI * 1318.5 * t).sin()
                    } else {
                        0.0
                    };
                    ((2.0 * PI * 987.77 * t).sin() + second) * 0.5 * (1.0 - x).powi(2)
                }
                // A low sine dropping in pitch, with a fast decay: a soft thump.
                Sound::Pop => (2.0 * PI * (90.0 + 120.0 * (1.0 - x)) * t).sin() * (1.0 - x).powi(4),
            };
            *s += gain * v;
        }
    }
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let target = 10f32.powf(-1.0 / 20.0);
    if peak > 0.0 {
        for s in &mut out {
            *s *= target / peak;
        }
    }
    out
}

/// A 16-bit PCM mono WAV file.
pub fn wav_bytes(samples: &[f32], rate: u32) -> Vec<u8> {
    let data_len = samples.len() as u32 * 2;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // mono
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    b
}

// ---------------------------------------------------------------- frames

/// GPU readback pads each row to a multiple of 256 bytes. A 1920-wide RGBA
/// frame (7680 bytes a row) happens to need none; 1080 wide (4320) does.
/// Strip it, or every 9:16 and square frame comes out skewed.
pub fn unpad_rows(data: &[u8], width: u32, height: u32, bytes_per_pixel: u32) -> Vec<u8> {
    let row = (width * bytes_per_pixel) as usize;
    let padded = row.div_ceil(256) * 256;
    if padded == row {
        return data[..row * height as usize].to_vec();
    }
    let mut out = Vec::with_capacity(row * height as usize);
    for y in 0..height as usize {
        out.extend_from_slice(&data[y * padded..y * padded + row]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_have_the_platform_sizes_and_a_safe_area_inside_the_frame() {
        assert_eq!((LANDSCAPE.width, LANDSCAPE.height), (1920, 1080));
        assert_eq!((PORTRAIT.width, PORTRAIT.height), (1080, 1920));
        assert_eq!((SQUARE.width, SQUARE.height), (1080, 1080));
        for f in FORMATS {
            let s = f.safe();
            assert!(s.left > -(f.width as f32) / 2.0 && s.right < f.width as f32 / 2.0);
            assert!(s.width() > 0.7 * f.width as f32, "{}", f.name);
            // Even dimensions: H.264 with yuv420p requires them.
            assert!(f.width % 2 == 0 && f.height % 2 == 0);
        }
        // 9:16 keeps the bottom fifth clear of text for the app's own UI.
        let s = PORTRAIT.safe();
        assert!(s.bottom >= -(1920.0 / 2.0) + 0.2 * 1920.0);
    }

    #[test]
    fn easing_starts_at_zero_and_ends_at_one() {
        for f in [ease_out_cubic, ease_in_out_cubic, ease_out_back] {
            assert!(f(0.0).abs() < 1e-6 && (f(1.0) - 1.0).abs() < 1e-6);
        }
        assert!(
            (0..100).any(|i| ease_out_back(i as f32 / 100.0) > 1.0),
            "back overshoots"
        );
    }

    #[test]
    fn shots_cross_fade_and_never_leave_an_empty_frame() {
        let tl = Timeline::promo(30);
        assert_eq!(tl.duration(), 9.0);
        assert_eq!(tl.frames(), 270);
        for n in 0..tl.frames() {
            let t = tl.time(n);
            let total: f32 = (0..tl.shots.len()).map(|i| tl.opacity(i, t)).sum();
            assert!(
                (total - 1.0).abs() < 1e-4,
                "frame {n} at {t:.2}s: shots add up to {total:.3}"
            );
        }
        // Halfway through a cross-fade, both shots are at half.
        assert!((tl.opacity(0, 3.0 - FADE / 2.0) - 0.5).abs() < 1e-4);
        // Mid-shot, exactly one shot is fully visible.
        assert_eq!(tl.opacity(1, 4.5), 1.0);
        assert_eq!(tl.opacity(0, 4.5) + tl.opacity(2, 4.5), 0.0);
    }

    #[test]
    fn exactly_one_caption_shows_on_every_frame_and_it_matches_the_srt() {
        let tl = Timeline::promo(30);
        for n in 0..tl.frames() {
            let t = tl.time(n);
            let i = tl.caption_at(t).expect("a caption on every frame");
            let s = &tl.shots[i];
            assert!(t >= s.start && t <= s.start + s.len, "frame {n}");
        }
        assert_eq!(tl.caption_at(2.999), Some(0));
        assert_eq!(tl.caption_at(3.0), Some(1));
    }

    #[test]
    fn captions_are_valid_srt_with_exact_times() {
        let srt = Timeline::promo(30).srt();
        assert!(
            srt.starts_with(
                "1\n00:00:00,000 --> 00:00:03,000\nGames and visual work, in Rust.\n\n"
            )
        );
        assert!(srt.contains("3\n00:00:06,500 --> 00:00:09,000\n"));
        assert_eq!(srt_time(3725.25), "01:02:05,250");
    }

    #[test]
    fn every_sound_is_tied_to_the_picture() {
        let tl = Timeline::promo(30);
        let cues = tl.cues();
        let ticks: Vec<f32> = cues
            .iter()
            .filter(|c| c.sound == Sound::Tick)
            .map(|c| c.at)
            .collect();
        assert_eq!(ticks.len(), BARS.len());
        // Each tick is when its bar finishes growing.
        for (i, at) in ticks.iter().enumerate() {
            assert!((at - (3.0 + bar_start(i) + BAR_GROW)).abs() < 1e-6);
        }
        // A whoosh on every cut, a chime on the end card.
        assert_eq!(cues.iter().filter(|c| c.sound == Sound::Whoosh).count(), 2);
        assert_eq!(cues.iter().filter(|c| c.sound == Sound::Chime).count(), 1);
        // The title's pop lands with its burst; whooshes peak on the cuts.
        assert!(cues.contains(&Cue {
            at: TITLE_BURST,
            sound: Sound::Pop
        }));
        for s in &tl.shots[1..] {
            assert!(
                cues.iter()
                    .any(|c| c.sound == Sound::Whoosh
                        && (c.at + WHOOSH / 2.0 - s.start).abs() < 1e-6)
            );
        }
        assert!(
            cues.windows(2).all(|w| w[0].at <= w[1].at),
            "cues in time order"
        );
    }

    #[test]
    fn the_soundtrack_lasts_as_long_as_the_video_and_never_clips() {
        let tl = Timeline::promo(30);
        let audio = render_audio(&tl, 48_000);
        assert_eq!(audio.len(), 9 * 48_000);
        let peak = audio.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - 10f32.powf(-1.0 / 20.0)).abs() < 1e-3, "peak {peak}");
        // The first tick is audible right at its cue.
        let at = (tl
            .cues()
            .iter()
            .find(|c| c.sound == Sound::Tick)
            .unwrap()
            .at
            * 48_000.0) as usize;
        let near: f32 = audio[at..at + 240]
            .iter()
            .map(|s| s.abs())
            .fold(0.0, f32::max);
        let before: f32 = audio[at - 2400..at - 2000]
            .iter()
            .map(|s| s.abs())
            .fold(0.0, f32::max);
        assert!(near > before * 1.5, "tick {near} vs bed {before}");
    }

    #[test]
    fn wav_header_matches_the_data() {
        let w = wav_bytes(&[0.0, 1.0, -1.0], 48_000);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(w[4..8].try_into().unwrap()), 36 + 6);
        assert_eq!(u32::from_le_bytes(w[24..28].try_into().unwrap()), 48_000);
        assert_eq!(w.len(), 44 + 6);
        assert_eq!(i16::from_le_bytes([w[46], w[47]]), 32767);
    }

    #[test]
    fn padded_readback_rows_are_stripped() {
        // 1080 wide RGBA: 4320 bytes a row, padded to 4352.
        let (w, h) = (1080u32, 3u32);
        let mut data = Vec::new();
        for y in 0..h {
            data.extend(std::iter::repeat_n(y as u8 + 1, 4320));
            data.extend(std::iter::repeat_n(0xEE, 32));
        }
        let out = unpad_rows(&data, w, h, 4);
        assert_eq!(out.len(), 4320 * 3);
        assert!(out[..4320].iter().all(|&b| b == 1) && out[4320 * 2..].iter().all(|&b| b == 3));
        // 1920 wide needs no padding.
        assert_eq!(unpad_rows(&vec![7; 7680 * 2], 1920, 2, 4).len(), 7680 * 2);
    }

    #[test]
    fn the_wipe_crosses_mid_frame_on_the_cut_and_is_off_screen_otherwise() {
        let tl = Timeline::promo(30);
        for s in &tl.shots[1..] {
            let (l, r) = tl.wipe(s.start).unwrap();
            assert!(((l + r) / 2.0).abs() < 1e-5, "centred on the cut");
            let (l0, _) = tl.wipe(s.start - WIPE / 2.0 + 0.01).unwrap();
            let (_, r1) = tl.wipe(s.start + WIPE / 2.0 - 0.01).unwrap();
            assert!(l0 < -0.5 && r1 > 0.5, "enters and leaves off-screen");
        }
        assert_eq!(tl.wipe(1.5), None, "no wipe mid-shot");
    }

    #[test]
    fn particles_are_deterministic_and_vanish() {
        let a = particle(7, 0.2, 0.8).unwrap();
        assert_eq!(
            a,
            particle(7, 0.2, 0.8).unwrap(),
            "same seed, same particle"
        );
        assert_ne!(a.0, particle(8, 0.2, 0.8).unwrap().0);
        let end = particle(7, 0.799, 0.8).unwrap();
        assert!(end.1 < 0.01, "faded out by the end of its life");
        assert_eq!(particle(7, 0.8, 0.8), None);
        assert_eq!(particle(7, -0.1, 0.8), None);
    }

    #[test]
    fn neighbouring_seeds_spread_out() {
        // Layouts use seeds 100, 101, 102...: their first numbers must not bunch up.
        let mut firsts: Vec<f32> = (100..110).map(|s| Rng::new(s).next()).collect();
        firsts.sort_by(f32::total_cmp);
        assert!(firsts[9] - firsts[0] > 0.6, "{firsts:?}");
    }
}
