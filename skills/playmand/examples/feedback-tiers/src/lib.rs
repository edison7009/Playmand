//! Game-feel feedback as plain data: what a level-up (or any reward) looks
//! like at time `t`, and a small player that runs it safely.
//!
//! Nothing here draws. A Bevy system, a web renderer or a Macroquad loop reads
//! [`Beats`] each frame and moves sprites, spawns particles and sets glow from
//! it, so the timing is tested once and every renderer agrees.

use std::f64::consts::PI;

/// How big a moment is. Each tier must read as clearly stronger than the one
/// before, and `Gain` (progress without a level) must never look like a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Gain,
    Level,
    Milestone,
    Finale,
}

impl Tier {
    /// Seconds the tier lasts: short for things players repeat, longer for rare ones.
    pub fn seconds(self, reduced: bool) -> f64 {
        let s: f64 = match self {
            Tier::Gain => 0.45,
            Tier::Level => 0.7,
            Tier::Milestone => 1.3,
            Tier::Finale => 2.4,
        };
        if reduced { s.min(0.45) } else { s }
    }

    /// Where in the span the impact lands (sound, rings, particles), if any.
    pub fn impact(self) -> Option<f64> {
        match self {
            Tier::Gain => None,
            Tier::Level | Tier::Milestone => Some(0.42),
            Tier::Finale => Some(0.62),
        }
    }
}

/// What the effect is doing at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beats {
    /// Height off the ground, in body heights.
    pub hop: f64,
    /// Vertical scale about the feet: below 1 crouches, above 1 stretches.
    /// Keep volume: scale x and z by 1/sqrt(squash).
    pub squash: f64,
    /// 0..1 glow on the character.
    pub glow: f64,
    /// Ground rings expanding after the impact: (age 0..1) for each ring alive.
    pub rings: [Option<f64>; 3],
    /// Particles flying after the impact: how many, and their age 0..1.
    pub particles: Option<(u32, f64)>,
}

/// A half-sine bump from `from` to `to`: 0 outside, 1 in the middle.
fn arc(t: f64, from: f64, to: f64) -> f64 {
    ((t - from) / (to - from)).clamp(0.0, 1.0) * PI
}

/// The beats of `tier` at `t` (0..1 of its span).
pub fn beats(tier: Tier, t: f64, reduced: bool) -> Beats {
    let t = t.clamp(0.0, 1.0);
    let mut b = Beats {
        hop: 0.0,
        squash: 1.0,
        glow: 0.0,
        rings: [None; 3],
        particles: None,
    };
    if reduced {
        // Nothing that moves, flashes or flies: a slow, steady glow only.
        b.glow = 0.4 * (t * PI).sin();
        return b;
    }
    // Every window ends inside the span, so nothing is left mid-motion.
    let bump = |from: f64, to: f64| arc(t, from, to.min(1.0)).sin();
    match tier {
        Tier::Gain => {
            b.glow = 0.3 * bump(0.2, 1.0);
            b.squash = 1.0 - 0.03 * bump(0.7, 1.0);
        }
        Tier::Level | Tier::Milestone | Tier::Finale => {
            let i = tier.impact().unwrap_or(0.5);
            let (height, rings, particles) = match tier {
                Tier::Level => (0.08, 1, 0),
                Tier::Milestone => (0.14, 2, 36),
                _ => (0.2, 3, 60),
            };
            // Anticipation (crouch), action (stretch in the air), follow-through
            // (squash on landing): the classic order that makes a hop feel alive.
            b.squash = 1.0 - 0.1 * bump(i - 0.12, i) + 0.07 * bump(i, i + 0.2)
                - 0.06 * bump(i + 0.35, i + 0.5);
            b.hop = height * bump(i, i + 0.4);
            b.glow = (0.3 + 0.2 * rings as f64) * bump(0.05, 0.95);
            for (k, slot) in b.rings.iter_mut().enumerate().take(rings) {
                let start = i + k as f64 * 0.07;
                let age = (t - start) / 0.45f64.min(1.0 - start);
                *slot = (0.0..1.0).contains(&age).then_some(age);
            }
            let age = (t - i) / (1.0 - i);
            if particles > 0 && (0.0..1.0).contains(&age) {
                b.particles = Some((particles, age));
            }
        }
    }
    b
}

/// Runs one effect at a time. Every start gets a serial number, so a frame
/// that was already queued for a cancelled effect cannot finish it, and the
/// impact and completion each fire at most once.
#[derive(Debug, Default)]
pub struct Player {
    serial: u64,
    running: Option<Run>,
}

#[derive(Debug)]
struct Run {
    serial: u64,
    tier: Tier,
    reduced: bool,
    start: f64,
    impacted: bool,
}

/// What a frame produced: the beats to draw, plus one-off events.
#[derive(Debug, PartialEq)]
pub struct Frame {
    pub beats: Beats,
    pub impact: bool,
    pub done: bool,
}

impl Player {
    /// Starts `tier` at `now` seconds, cancelling anything running. Returns the
    /// serial the caller attaches to frames it schedules for this effect.
    pub fn start(&mut self, tier: Tier, reduced: bool, now: f64) -> u64 {
        self.serial += 1;
        self.running = Some(Run {
            serial: self.serial,
            tier,
            reduced,
            start: now,
            impacted: tier.impact().is_none(),
        });
        self.serial
    }

    /// Stops whatever is running (switching characters, leaving the page).
    pub fn cancel(&mut self) {
        self.serial += 1;
        self.running = None;
    }

    /// Advances the effect a frame scheduled with `serial` belongs to.
    /// Frames from a cancelled or replaced effect return `None`.
    pub fn frame(&mut self, serial: u64, now: f64) -> Option<Frame> {
        let run = self.running.as_mut().filter(|r| r.serial == serial)?;
        let t = ((now - run.start) / run.tier.seconds(run.reduced)).clamp(0.0, 1.0);
        let impact = !run.impacted && run.tier.impact().is_some_and(|i| t >= i);
        if impact {
            run.impacted = true;
        }
        let beats = beats(run.tier, t, run.reduced);
        let done = t >= 1.0;
        if done {
            self.running = None;
        }
        Some(Frame {
            beats,
            impact,
            done,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIERS: [Tier; 4] = [Tier::Gain, Tier::Level, Tier::Milestone, Tier::Finale];

    fn peak(tier: Tier) -> f64 {
        (0..=200)
            .map(|i| {
                let b = beats(tier, i as f64 / 200.0, false);
                let rings = b.rings.iter().flatten().count() as f64;
                b.hop * 10.0 + b.glow + rings + b.particles.map_or(0.0, |(n, _)| n as f64 / 30.0)
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn each_tier_reads_stronger_than_the_one_before() {
        for w in TIERS.windows(2) {
            assert!(peak(w[1]) > peak(w[0]), "{:?} vs {:?}", w[1], w[0]);
        }
    }

    #[test]
    fn progress_without_a_level_never_looks_like_one() {
        for i in 0..=100 {
            let b = beats(Tier::Gain, i as f64 / 100.0, false);
            assert_eq!((b.hop, b.rings, b.particles), (0.0, [None; 3], None));
        }
        assert_eq!(Tier::Gain.impact(), None);
    }

    #[test]
    fn a_hop_crouches_before_it_leaves_the_ground_and_stretches_in_the_air() {
        let i = Tier::Level.impact().unwrap();
        assert!(beats(Tier::Level, i - 0.05, false).squash < 1.0);
        assert!(beats(Tier::Level, i + 0.1, false).squash > 1.0);
        assert!(beats(Tier::Level, i + 0.2, false).hop > 0.0);
    }

    #[test]
    fn everything_has_settled_when_the_span_ends() {
        for tier in TIERS {
            let b = beats(tier, 1.0, false);
            assert!(
                b.hop.abs() < 1e-9 && (b.squash - 1.0).abs() < 1e-9 && b.glow < 1e-9,
                "{tier:?}"
            );
            assert_eq!((b.rings, b.particles), ([None; 3], None), "{tier:?}");
        }
    }

    #[test]
    fn reduced_motion_keeps_a_glow_and_nothing_that_moves() {
        for tier in TIERS {
            assert!(tier.seconds(true) <= 0.45);
            for i in 0..=50 {
                let b = beats(tier, i as f64 / 50.0, true);
                assert_eq!(
                    (b.hop, b.squash, b.rings, b.particles),
                    (0.0, 1.0, [None; 3], None)
                );
            }
            assert!(beats(tier, 0.5, true).glow > 0.3);
        }
    }

    #[test]
    fn the_impact_and_the_completion_each_fire_once() {
        let mut player = Player::default();
        let serial = player.start(Tier::Milestone, false, 0.0);
        let (mut impacts, mut dones) = (0, 0);
        for k in 0..=200 {
            if let Some(f) = player.frame(serial, k as f64 / 60.0) {
                impacts += u32::from(f.impact);
                dones += u32::from(f.done);
            }
        }
        assert_eq!((impacts, dones), (1, 1));
    }

    #[test]
    fn a_frame_queued_before_a_cancel_cannot_finish_the_effect() {
        let mut player = Player::default();
        let serial = player.start(Tier::Finale, false, 0.0);
        assert!(player.frame(serial, 0.5).is_some());
        player.cancel();
        assert_eq!(player.frame(serial, 10.0), None);
        let next = player.start(Tier::Level, false, 11.0);
        assert_eq!(
            player.frame(serial, 12.0),
            None,
            "the old serial stays dead"
        );
        assert!(player.frame(next, 12.0).unwrap().done);
    }
}
