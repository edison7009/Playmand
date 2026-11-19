//! The harbor shot: a giant mecha standing in a rainy night harbour, brought
//! to life in real time. Everything here is a pure function of time `t`
//! (seconds), so the page and a frame-by-frame recording show exactly the
//! same picture, and every periodic motion divides the loop so it repeats
//! without a seam.
//!
//! Coordinates are pixels of the background plate (2000 x 837, y down).

use std::f32::consts::TAU;

/// The whole shot loops every `LOOP` seconds.
pub const LOOP: f32 = 24.0;
/// Background plate size in pixels.
pub const PLATE: [f32; 2] = [2000.0, 837.0];
/// Where the mecha's 576 x 840 cut-out sits on the plate (top-left).
pub const MECHA_POS: [f32; 2] = [735.0, 85.0];
pub const MECHA_SIZE: [f32; 2] = [576.0, 840.0];
/// Chest core, in the mecha's own pixels.
pub const CORE: [f32; 2] = [290.0, 173.0];
/// Below this mecha-local y the body is under water.
pub const WATERLINE: f32 = 705.0;

/// Segments the shader's bolt array holds.
pub const BOLT_SEGMENTS: usize = 24;

/// Crane lamps found on the plate (warm, bright local maxima).
pub const LAMPS: [[f32; 2]; 29] = [
    [88.0, 598.0],
    [103.0, 510.0],
    [129.0, 607.0],
    [146.0, 566.0],
    [163.0, 507.0],
    [296.0, 606.0],
    [319.0, 12.0],
    [363.0, 661.0],
    [366.0, 607.0],
    [461.0, 281.0],
    [488.0, 292.0],
    [504.0, 659.0],
    [1501.0, 337.0],
    [1577.0, 674.0],
    [1599.0, 675.0],
    [1637.0, 104.0],
    [1655.0, 590.0],
    [1667.0, 69.0],
    [1694.0, 637.0],
    [1757.0, 602.0],
    [1824.0, 588.0],
    [1833.0, 627.0],
    [1887.0, 431.0],
    [1889.0, 583.0],
    [1902.0, 645.0],
    [1918.0, 470.0],
    [1941.0, 510.0],
    [1952.0, 652.0],
    [1961.0, 539.0],
];

/// Edges water pours off, as segments in mecha-local pixels: shoulder
/// plates, chest plate, elbows, fingers, crotch, thigh plates.
pub const EMITTERS: [[f32; 4]; 12] = [
    [48.0, 150.0, 150.0, 162.0],
    [426.0, 162.0, 530.0, 150.0],
    [175.0, 228.0, 405.0, 228.0],
    [40.0, 262.0, 90.0, 262.0],
    [488.0, 262.0, 538.0, 262.0],
    [30.0, 500.0, 80.0, 535.0],
    [497.0, 535.0, 547.0, 500.0],
    [255.0, 440.0, 325.0, 440.0],
    [150.0, 540.0, 235.0, 540.0],
    [345.0, 540.0, 430.0, 540.0],
    [150.0, 60.0, 200.0, 40.0],
    [380.0, 40.0, 430.0, 60.0],
];

/// Searchlights: origin on the plate, base angle (radians, y down), sweep
/// amplitude, sweep cycles per loop, half-width of the cone.
pub struct Beam {
    pub origin: [f32; 2],
    pub angle: f32,
    pub sweep: f32,
    pub cycles: f32,
    pub spread: f32,
}

pub const BEAMS: [Beam; 6] = [
    Beam {
        origin: [488.0, 292.0],
        angle: -0.16,
        sweep: 0.10,
        cycles: 2.0,
        spread: 0.075,
    },
    Beam {
        origin: [432.0, 462.0],
        angle: 0.02,
        sweep: 0.08,
        cycles: 3.0,
        spread: 0.07,
    },
    Beam {
        origin: [1637.0, 104.0],
        angle: 2.78,
        sweep: 0.09,
        cycles: 2.0,
        spread: 0.075,
    },
    Beam {
        origin: [1448.0, 560.0],
        angle: -2.85,
        sweep: 0.08,
        cycles: 3.0,
        spread: 0.07,
    },
    Beam {
        origin: [1010.0, 380.0],
        angle: -2.05,
        sweep: 0.22,
        cycles: 1.0,
        spread: 0.06,
    },
    Beam {
        origin: [1040.0, 380.0],
        angle: -1.1,
        sweep: 0.22,
        cycles: 1.0,
        spread: 0.06,
    },
];

/// A sine with a whole number of cycles per loop, so it repeats seamlessly.
pub fn wave(t: f32, cycles: f32, phase: f32) -> f32 {
    (TAU * cycles * t / LOOP + phase).sin()
}

/// Current angle of searchlight `i`.
pub fn beam_angle(i: usize, t: f32) -> f32 {
    let b = &BEAMS[i];
    b.angle + b.sweep * wave(t, b.cycles, i as f32 * 1.7)
}

/// Breathing, -1..1: a slow idle cycle every four seconds.
pub fn breath(t: f32) -> f32 {
    wave(t, 6.0, 0.0)
}

/// Lightning brightness 0..1: two strikes per loop, each a quick double
/// flicker that fades.
pub fn lightning(t: f32) -> f32 {
    let t = t.rem_euclid(LOOP);
    let strike = |at: f32| {
        let d = t - at;
        if !(0.0..1.2).contains(&d) {
            return 0.0;
        }
        let first = (-d * 18.0).exp();
        let second = if d > 0.14 {
            0.8 * (-(d - 0.14) * 9.0).exp()
        } else {
            0.0
        };
        first.max(second)
    };
    strike(7.2).max(strike(17.6) * 0.7)
}

/// Which strike is lit at `t` (0 or 1), if any.
pub fn strike_index(t: f32) -> usize {
    if t.rem_euclid(LOOP) < 12.0 {
        0
    } else {
        1
    }
}

/// A forked lightning bolt as line segments (x0, y0, x1, y1) on the plate,
/// far behind the cranes. Midpoint displacement from a fixed seed, so every
/// frame and every render draws the same bolt.
pub fn bolt(strike: usize) -> Vec<[f32; 4]> {
    let (top, bottom, seed) = if strike == 0 {
        ([1480.0, -10.0], [1330.0, 330.0], 0x9E37_79B9u32)
    } else {
        ([360.0, -10.0], [560.0, 300.0], 0x85EB_CA6Bu32)
    };
    let mut rng = seed;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        (rng >> 8) as f32 / (1u32 << 24) as f32 - 0.5
    };
    let mut trunk = vec![top, bottom];
    let mut spread = 90.0;
    for _ in 0..4 {
        let mut finer = Vec::with_capacity(trunk.len() * 2);
        for w in trunk.windows(2) {
            let (a, b) = (w[0], w[1]);
            finer.push(a);
            finer.push([
                (a[0] + b[0]) / 2.0 + next() * spread,
                (a[1] + b[1]) / 2.0 + next() * spread * 0.3,
            ]);
        }
        finer.push(*trunk.last().unwrap());
        trunk = finer;
        spread *= 0.5;
    }
    let mut segments: Vec<[f32; 4]> = trunk
        .windows(2)
        .map(|w| [w[0][0], w[0][1], w[1][0], w[1][1]])
        .collect();
    // One branch off the upper third.
    let from = trunk[trunk.len() / 3];
    let dir = if strike == 0 { -1.0 } else { 1.0 };
    let mut p = from;
    for i in 0..5 {
        let q = [
            p[0] + dir * (22.0 + next() * 20.0),
            p[1] + 24.0 + next() * 14.0 - i as f32,
        ];
        segments.push([p[0], p[1], q[0], q[1]]);
        p = q;
    }
    segments
}

/// Camera: zoom and offset (plate pixels). A slow push-in and drift that
/// returns to the start at the end of the loop.
pub fn camera(t: f32) -> (f32, [f32; 2]) {
    let zoom = 1.06 + 0.025 * (0.5 - 0.5 * wave(t, 1.0, std::f32::consts::FRAC_PI_2));
    let dx = 14.0 * wave(t, 1.0, 0.0);
    let dy = 5.0 * wave(t, 2.0, 0.8);
    (zoom, [dx, dy])
}

/// Map a point of the canvas (CSS pixels, y down) to plate pixels, with a
/// parallax factor (1 = background, >1 = nearer layers move more).
pub fn to_plate(p: [f32; 2], canvas: [f32; 2], t: f32, parallax: f32) -> [f32; 2] {
    let (zoom, d) = camera(t);
    let s = (canvas[0] / PLATE[0]).max(canvas[1] / PLATE[1]) * zoom;
    [
        (p[0] - canvas[0] * 0.5) / s + PLATE[0] * 0.5 + d[0] * parallax,
        (p[1] - canvas[1] * 0.5) / s + PLATE[1] * 0.5 + d[1] * parallax,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_motion_loops_without_a_seam() {
        for t in [0.0, 3.3, 11.0] {
            assert!((breath(t) - breath(t + LOOP)).abs() < 1e-4);
            for i in 0..BEAMS.len() {
                assert!((beam_angle(i, t) - beam_angle(i, t + LOOP)).abs() < 1e-4);
            }
            let (a, b) = (camera(t), camera(t + LOOP));
            assert!((a.0 - b.0).abs() < 1e-4 && (a.1[0] - b.1[0]).abs() < 1e-3);
            assert!((lightning(t) - lightning(t + LOOP)).abs() < 1e-6);
        }
    }

    #[test]
    fn lightning_strikes_twice_and_is_dark_otherwise() {
        assert!(lightning(7.2) > 0.99);
        assert!(lightning(17.6) > 0.6);
        assert_eq!(lightning(3.0), 0.0);
        assert_eq!(lightning(12.0), 0.0);
        assert!((0..2400).all(|i| (0.0..=1.0).contains(&lightning(i as f32 * 0.01))));
    }

    #[test]
    fn bolts_are_fixed_and_fit_the_shader() {
        for strike in 0..2 {
            let b = bolt(strike);
            assert_eq!(b, bolt(strike), "same bolt every frame");
            assert!(b.len() <= BOLT_SEGMENTS, "{} segments", b.len());
            assert!(
                b.iter().all(|s| s[1] < 420.0 && s[3] < 420.0),
                "stays in the sky"
            );
        }
        assert_eq!(strike_index(7.3), 0);
        assert_eq!(strike_index(17.7), 1);
    }

    #[test]
    fn the_camera_always_covers_the_canvas() {
        // The plate must fill any canvas shape: no edge may show.
        for canvas in [
            [1920.0, 1080.0],
            [1280.0, 535.0],
            [390.0, 844.0],
            [2560.0, 900.0],
        ] {
            for i in 0..48 {
                let t = i as f32 * 0.5;
                for parallax in [1.0, 1.15] {
                    let a = to_plate([0.0, 0.0], canvas, t, parallax);
                    let b = to_plate(canvas, canvas, t, parallax);
                    assert!(a[0] >= 0.0 && a[1] >= 0.0, "{canvas:?} t={t} {a:?}");
                    assert!(
                        b[0] <= PLATE[0] && b[1] <= PLATE[1],
                        "{canvas:?} t={t} {b:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn emitters_and_lamps_are_where_they_belong() {
        for e in EMITTERS {
            for (x, y) in [(e[0], e[1]), (e[2], e[3])] {
                assert!((0.0..MECHA_SIZE[0]).contains(&x) && (0.0..WATERLINE).contains(&y));
            }
        }
        for l in LAMPS {
            assert!((0.0..PLATE[0]).contains(&l[0]) && (0.0..PLATE[1]).contains(&l[1]));
        }
        let core = [MECHA_POS[0] + CORE[0], MECHA_POS[1] + CORE[1]];
        assert!((core[0] - 1025.0).abs() < 4.0 && (core[1] - 258.0).abs() < 4.0);
    }
}
