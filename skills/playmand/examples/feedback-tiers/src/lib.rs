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
    seconds: f64,
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
        self.start_faster(tier, reduced, now, 1.0)
    }

    /// Like [`Player::start`], but plays `speed` times faster (used when
    /// rewards are queued, so the display keeps up with the game).
    pub fn start_faster(&mut self, tier: Tier, reduced: bool, now: f64, speed: f64) -> u64 {
        self.serial += 1;
        self.running = Some(Run {
            serial: self.serial,
            tier,
            reduced,
            start: now,
            seconds: tier.seconds(reduced) / speed.max(1.0),
            impacted: tier.impact().is_none(),
        });
        self.serial
    }

    /// Plays the running effect `speed` times faster than its normal span from
    /// now on, keeping its progress (no jump in the animation).
    pub fn hurry(&mut self, speed: f64, now: f64) {
        if let Some(run) = self.running.as_mut() {
            let t = ((now - run.start) / run.seconds).clamp(0.0, 1.0);
            run.seconds = run.tier.seconds(run.reduced) / speed.max(1.0);
            run.start = now - t * run.seconds;
        }
    }

    /// The serial of the running effect, if one is running.
    pub fn current(&self) -> Option<u64> {
        self.running.as_ref().map(|r| r.serial)
    }

    /// True once the running effect has passed its impact (or has none), so a
    /// new one can take over without swallowing a beat.
    pub fn past_impact(&self) -> bool {
        self.running.as_ref().is_none_or(|r| r.impacted)
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
        let t = ((now - run.start) / run.seconds).clamp(0.0, 1.0);
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

/// Rewards that can arrive faster than their effects play: a player mashing
/// the feed button. Each level gets its own impact, in order, and the shown
/// level stays within a level or two of the real one.
///
/// - An effect is never restarted before its impact; the next level waits.
/// - Once the current effect has passed its impact, the next queued level
///   takes over at once (only the current follow-through is cut short).
/// - While levels are queued, each plays faster, down to a third of its span.
/// - Progress without a level (`Gain`) only plays when nothing else is.
#[derive(Debug, Default)]
pub struct Rewards {
    player: Player,
    queue: std::collections::VecDeque<Tier>,
    reduced: bool,
    shown_levels: u32,
}

/// What a frame of [`Rewards`] produced.
#[derive(Debug, PartialEq)]
pub struct RewardFrame {
    pub beats: Beats,
    /// The tier whose impact landed this frame: play its sound now.
    pub impact: Option<Tier>,
    /// Levels the display has shown so far (it rises at each impact).
    pub shown_levels: u32,
}

const REST: Beats = Beats {
    hop: 0.0,
    squash: 1.0,
    glow: 0.0,
    rings: [None; 3],
    particles: None,
};

impl Rewards {
    pub fn new(reduced: bool) -> Self {
        Rewards {
            reduced,
            ..Default::default()
        }
    }

    /// The game granted a reward at `now`. Settle the real state in the game;
    /// this only decides what the player sees.
    pub fn grant(&mut self, tier: Tier, now: f64) {
        if tier == Tier::Gain {
            if self.player.current().is_none() && self.queue.is_empty() {
                self.player.start(Tier::Gain, self.reduced, now);
            }
            return;
        }
        self.queue.push_back(tier);
        self.start_next(now);
        // More is waiting: speed up what is playing so the display keeps up.
        if !self.queue.is_empty() {
            self.player.hurry(self.speed(), now);
        }
    }

    /// Faster the more is queued, down to a third of the normal span.
    fn speed(&self) -> f64 {
        (1.0 + self.queue.len() as f64).min(3.0)
    }

    /// Switching characters or leaving the screen: drop everything.
    pub fn cancel(&mut self) {
        self.player.cancel();
        self.queue.clear();
    }

    /// True while anything is playing or waiting.
    pub fn busy(&self) -> bool {
        self.player.current().is_some() || !self.queue.is_empty()
    }

    fn start_next(&mut self, now: f64) {
        let running_gain = self
            .player
            .running
            .as_ref()
            .is_some_and(|r| r.tier == Tier::Gain);
        if !(self.player.past_impact() || running_gain) {
            return;
        }
        if let Some(tier) = self.queue.pop_front() {
            let speed = self.speed();
            self.player.start_faster(tier, self.reduced, now, speed);
        }
    }

    pub fn frame(&mut self, now: f64) -> RewardFrame {
        let mut impact = None;
        let mut beats = REST;
        if let Some(serial) = self.player.current() {
            let tier = self.player.running.as_ref().map(|r| r.tier);
            if let Some(f) = self.player.frame(serial, now) {
                beats = f.beats;
                if f.impact {
                    impact = tier;
                    self.shown_levels += 1;
                }
            }
        }
        self.start_next(now);
        RewardFrame {
            beats,
            impact,
            shown_levels: self.shown_levels,
        }
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

    /// Feeds one level every `every` seconds for `seconds`, at 60 fps.
    /// Returns (levels granted, impacts seen, worst lag of the shown level,
    /// seconds after the last feed until everything is still).
    fn mash(every: f64, seconds: f64) -> (u32, u32, u32, f64) {
        let mut rewards = Rewards::new(false);
        let (mut granted, mut impacts, mut worst_lag) = (0, 0, 0);
        let mut next_feed = 0.0;
        let mut last_feed = 0.0;
        let mut k = 0;
        loop {
            let now = k as f64 / 60.0;
            if now <= seconds && now >= next_feed {
                rewards.grant(Tier::Level, now);
                granted += 1;
                last_feed = now;
                next_feed += every;
            }
            let f = rewards.frame(now);
            impacts += u32::from(f.impact.is_some());
            worst_lag = worst_lag.max(granted - f.shown_levels);
            if now > seconds && !rewards.busy() {
                return (granted, impacts, worst_lag, now - last_feed);
            }
            k += 1;
        }
    }

    #[test]
    fn mashing_gives_every_level_its_own_impact_and_the_display_keeps_up() {
        for every in [0.1, 0.2, 0.3, 0.5] {
            let (granted, impacts, lag, settle) = mash(every, 3.0);
            assert_eq!(impacts, granted, "every {every} s: a level lost its impact");
            // Counted on the frame of the feed, so 1 is the least possible: the
            // newest level has not reached its impact yet.
            assert!(
                lag <= 3,
                "every {every} s: the shown level fell {lag} behind"
            );
            assert!(
                settle < 1.0,
                "every {every} s: still playing {settle:.2} s after the last feed"
            );
        }
    }

    #[test]
    fn a_single_level_plays_at_full_length() {
        let mut rewards = Rewards::new(false);
        rewards.grant(Tier::Level, 0.0);
        let mut end = 0.0;
        for k in 0..200 {
            let now = k as f64 / 60.0;
            rewards.frame(now);
            if !rewards.busy() {
                end = now;
                break;
            }
        }
        assert!(
            (end - Tier::Level.seconds(false)).abs() < 0.05,
            "ended at {end}"
        );
    }

    #[test]
    fn progress_without_a_level_never_delays_or_hides_a_level() {
        let mut rewards = Rewards::new(false);
        rewards.grant(Tier::Gain, 0.0);
        rewards.frame(0.1);
        rewards.grant(Tier::Level, 0.1);
        let mut impacts = 0;
        for k in 7..120 {
            impacts += u32::from(rewards.frame(k as f64 / 60.0).impact.is_some());
        }
        assert_eq!(impacts, 1, "the level replaced the gain and landed once");
    }

    #[test]
    fn cancelling_drops_the_queue() {
        let mut rewards = Rewards::new(false);
        for _ in 0..5 {
            rewards.grant(Tier::Level, 0.0);
        }
        rewards.cancel();
        assert!(!rewards.busy());
        assert_eq!(rewards.frame(1.0).impact, None);
    }
}
