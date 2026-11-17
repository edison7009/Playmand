//! Claude and ChatGPT icons reimagined as particle swarms. Each icon rests as
//! a round 3D nebula rolling in space (star ball, globe, ringed planet, torus)
//! and morphs into the icon in four different ways: vortex, rain, big bang
//! and scan. Each segment releases into the next nebula form.
//!
//! cargo run --bin icons                      # watch it
//! cargo run --bin icons -- --record frames   # save one seamless loop
//!
//! Fan-made reinterpretation; not affiliated with Anthropic or OpenAI.

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    text::FontSize,
};
use playmand_demos::{Clock, DemoPlugin, icon_points};
use std::f32::consts::{PI, TAU};

/// One nebula-assemble-hold-release cycle per style.
const SEGMENT: f32 = 4.0;
const STYLES: [&str; 4] = ["Vortex", "Rain", "Big bang", "Scan"];
const FORMS: [&str; 4] = ["Star ball", "Globe", "Ringed planet", "Torus"];
/// Nebula radius and how many whole turns it rolls per loop.
const NEBULA_R: f32 = 125.0;
const ROLL_TURNS: f32 = 4.0;
const LOOP: f32 = SEGMENT * STYLES.len() as f32;
const FPS: u32 = 20;
const PARTICLES: usize = 1_800;
const CENTER_Y: f32 = 38.0;
const ICON_X: f32 = 235.0;
const MAX_STAGGER: f32 = 0.12;
/// Ordered styles spread arrivals wider so the order is visible.
const ORDERED_STAGGER: f32 = 0.20;

#[derive(Component)]
struct Particle {
    target: Vec2,
    /// Three random numbers that place this star in every nebula form.
    seed: Vec3,
    center: Vec2,
    /// -1 for the left icon, +1 for the right one.
    side: f32,
    rand: f32,
    shimmer: f32,
}

/// Icon name shown while the icon holds; the field is the icon's side.
#[derive(Component)]
struct Caption(f32);

/// Per-icon "form -> style" label; the field is the icon's side.
#[derive(Component)]
struct StyleLabel(f32);

/// The right icon runs 2.5 segments ahead, so the two sides never show the
/// same form or style, and assemble at different moments.
const RIGHT_OFFSET: f32 = SEGMENT * 2.5;

fn local_time(t: f32, side: f32) -> f32 {
    let offset = if side > 0.0 { RIGHT_OFFSET } else { 0.0 };
    (t + offset) % LOOP
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
}

fn main() {
    App::new()
        .add_plugins(DemoPlugin {
            title: "Playmand · icons in particles",
            loop_seconds: LOOP,
            fps: FPS,
        })
        .insert_resource(ClearColor(Color::srgb_u8(0x06, 0x06, 0x09)))
        .add_systems(Startup, setup)
        .add_systems(Update, (swarm, captions))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        DebandDither::Enabled,
    ));

    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    let dots = [
        meshes.add(Circle::new(1.6)),
        meshes.add(Circle::new(2.1)),
        meshes.add(Circle::new(2.7)),
    ];

    // (points, side, base colour, label)
    let icons = [
        (
            icon_points::CLAUDE,
            -1.0,
            Vec3::new(0.85, 0.47, 0.34),
            "Claude",
        ),
        (
            icon_points::CHATGPT,
            1.0,
            Vec3::new(0.80, 0.95, 0.95),
            "ChatGPT",
        ),
    ];
    for (points, side, base, label) in icons {
        let center = Vec2::new(side * ICON_X, CENTER_Y);
        let palette: Vec<Handle<ColorMaterial>> = (0..8)
            .map(|i| {
                let k = 1.6 + i as f32 * 0.18;
                materials.add(Color::srgb(base.x * k, base.y * k, base.z * k))
            })
            .collect();
        for p in points.iter().take(PARTICLES) {
            commands.spawn((
                Particle {
                    target: Vec2::new(p[0], p[1]),
                    seed: Vec3::new(rng.next(), rng.next(), rng.next()),
                    center,
                    side,
                    rand: rng.next(),
                    shimmer: rng.next() * TAU,
                },
                Mesh2d(dots[(rng.next() * 3.0) as usize % 3].clone()),
                MeshMaterial2d(palette[(rng.next() * 8.0) as usize % 8].clone()),
                Transform::from_translation(center.extend(0.0)),
            ));
        }
        commands.spawn((
            Caption(side),
            Text2d::new(label),
            TextFont {
                font_size: FontSize::Px(30.0),
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
            Transform::from_xyz(center.x, CENTER_Y - 190.0, 1.0),
        ));
        commands.spawn((
            StyleLabel(side),
            Text2d::new(""),
            TextFont {
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.62, 0.35, 0.0)),
            Transform::from_xyz(center.x, CENTER_Y - 226.0, 1.0),
        ));
    }
    commands.spawn((
        Text2d::new("Reimagined in particles with Playmand  |  Bevy 0.20"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgba(0.75, 0.75, 0.8, 0.5)),
        Transform::from_xyz(0.0, -244.0, 1.0),
    ));
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 0 = in the nebula, 1 = in the icon. Zero at both ends of every segment, so
/// segments chain together and the whole loop is seamless. `stagger` orders
/// the arrivals; `flight` is how long one particle takes to land. The nebula
/// rests on its own for a moment at the start of each segment.
fn assembled(u: f32, stagger: f32, flight: f32) -> f32 {
    let gather = smoothstep((u - 0.18 - stagger) / flight);
    let release = smoothstep((u - 0.76 - stagger * 0.4) / 0.12);
    gather * (1.0 - release)
}

fn bezier(a: Vec2, k: Vec2, b: Vec2, w: f32) -> Vec2 {
    let m = 1.0 - w;
    a * (m * m) + k * (2.0 * m * w) + b * (w * w)
}

/// A star's place in one of the round nebula forms, before rolling.
fn nebula_point(form: usize, s: Vec3) -> Vec3 {
    let r = NEBULA_R;
    let on_sphere = |u: f32, v: f32| {
        let z = 2.0 * u - 1.0;
        let a = TAU * v;
        let ring = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(ring * a.cos(), z, ring * a.sin())
    };
    match form {
        // Star ball: a solid sphere, denser towards the core.
        0 => on_sphere(s.x, s.y) * r * s.z.powf(0.6),
        // Globe: stars traced along 7 latitude and 8 longitude lines.
        1 => {
            let (lat, lon) = if s.z < 0.55 {
                (((s.x * 7.0).floor() + 0.5) / 7.0 - 0.5, s.y)
            } else {
                (s.x - 0.5, (s.y * 8.0).floor() / 8.0)
            };
            let lat = lat * PI;
            let lon = lon * TAU;
            Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin()) * r
        }
        // Ringed planet: a smaller sphere wrapped in a flat, tilted ring.
        2 => {
            if s.z < 0.55 {
                on_sphere(s.x, s.y) * r * 0.62
            } else {
                let a = TAU * s.y;
                let rr = r * (0.95 + 0.45 * s.x);
                Quat::from_rotation_x(0.45) * Vec3::new(rr * a.cos(), 0.0, rr * a.sin())
            }
        }
        // Torus: a rolling ring of stars.
        _ => {
            let a = TAU * s.x;
            let b = TAU * s.y;
            let (major, minor) = (r * 0.72, r * 0.30);
            Vec3::new(
                (major + minor * b.cos()) * a.cos(),
                minor * b.sin(),
                (major + minor * b.cos()) * a.sin(),
            )
        }
    }
}

fn swarm(clock: Res<Clock>, mut q: Query<(&Particle, &mut Transform)>) {
    let roll = TAU * ROLL_TURNS * clock.t / LOOP;
    q.par_iter_mut().for_each(|(p, mut t)| {
        // Each icon follows its own timeline.
        let lt = local_time(clock.t, p.side);
        let style = (lt / SEGMENT) as usize % STYLES.len();
        let u = (lt % SEGMENT) / SEGMENT;
        // Before the hold the stars sit in this segment's nebula; after it they
        // leave for the next segment's form, so all four forms appear in turn.
        let form = if u < 0.64 {
            style
        } else {
            (style + 1) % FORMS.len()
        };
        // Roll the ball around a tilted axis; the two icons roll opposite ways.
        let axis = Vec3::new(0.85 * -p.side, 0.45, 0.28).normalize();
        let star = Quat::from_axis_angle(axis, roll) * nebula_point(form, p.seed);
        let cloud = star.truncate();
        let depth = (star.z / NEBULA_R).clamp(-1.0, 1.0);
        let goal = p.target + Vec2::from_angle(TAU * 4.0 * clock.t / LOOP + p.shimmer) * 1.2;
        let pos = match style {
            // Vortex: spiral in along a curved path, random arrival order.
            0 => {
                let w = assembled(u, p.rand * MAX_STAGGER, 0.25);
                let k = Vec2::from_angle(1.9 * -p.side).rotate(cloud) * 0.5;
                bezier(cloud, k, goal, w)
            }
            // Rain: fall from above the screen, lowest pixels land first.
            1 => {
                let order = ((p.target.y + 150.0) / 300.0).clamp(0.0, 1.0);
                let w = assembled(u, order * ORDERED_STAGGER, 0.14);
                let k = Vec2::new(p.target.x + (p.rand - 0.5) * 60.0, 360.0);
                bezier(cloud, k, goal, w)
            }
            // Big bang: collapse to the centre, then burst out in a ripple.
            2 => {
                let order = (p.target.length() / 212.0).clamp(0.0, 1.0);
                let w = assembled(u, order * ORDERED_STAGGER, 0.14);
                if w < 0.5 {
                    cloud * (1.0 - smoothstep(w * 2.0))
                } else {
                    goal * smoothstep(w * 2.0 - 1.0)
                }
            }
            // Scan: sweep in sideways from the outer edge, one column at a time.
            _ => {
                let order = ((-p.side * p.target.x + 150.0) / 300.0).clamp(0.0, 1.0);
                let w = assembled(u, order * ORDERED_STAGGER, 0.14);
                let k = Vec2::new(p.target.x + p.side * 320.0, p.target.y);
                bezier(cloud, k, goal, w)
            }
        };
        // Near stars look bigger while in the nebula; all settle to one size in the icon.
        let w = assembled(u, p.rand * MAX_STAGGER, 0.25);
        let nebula_scale = 0.8 + 0.4 * depth;
        t.translation = (p.center + pos).extend(depth * (1.0 - w));
        t.scale = Vec3::splat(nebula_scale + (1.15 - nebula_scale) * w);
    });
}

fn captions(
    clock: Res<Clock>,
    mut names: Query<(&Caption, &mut TextColor), Without<StyleLabel>>,
    mut labels: Query<(&StyleLabel, &mut Text2d, &mut TextColor), Without<Caption>>,
) {
    for (caption, mut color) in &mut names {
        let u = (local_time(clock.t, caption.0) % SEGMENT) / SEGMENT;
        let a = smoothstep((u - 0.50) / 0.08) * (1.0 - smoothstep((u - 0.76) / 0.06));
        color.0 = Color::srgba(1.0, 1.0, 1.0, 0.9 * a);
    }
    for (label, mut text, mut color) in &mut labels {
        let lt = local_time(clock.t, label.0);
        let style = (lt / SEGMENT) as usize % STYLES.len();
        let u = (lt % SEGMENT) / SEGMENT;
        let name = format!("{}  ->  {}", FORMS[style], STYLES[style]);
        if text.0 != name {
            text.0 = name;
        }
        let fade = smoothstep(u / 0.06) * (1.0 - smoothstep((u - 0.92) / 0.08));
        color.0 = Color::srgba(1.0, 0.62, 0.35, 0.8 * fade);
    }
}
