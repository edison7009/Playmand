//! "A pelican riding a bicycle", built from Bevy 2D primitives.
//!
//! cargo run --bin pelican                      # watch it
//! cargo run --bin pelican -- --record frames   # save one seamless loop

use bevy::prelude::*;
use playmand_demos::{Clock, DemoPlugin, bar_transform};
use std::f32::consts::{PI, TAU};

const LOOP: f32 = 4.0;
const FPS: u32 = 25;

// Ground speed and wheel size are chosen so wheels, road dashes, hills and
// clouds all return to their starting pose after exactly one loop.
const WHEEL_R: f32 = 62.0;
const SPEED: f32 = 19.0 * PI / 4.0 * WHEEL_R / LOOP; // 19 eighth-turns of the wheel per loop
const CADENCE: f32 = 3.0; // crank turns per loop
const CRANK_R: f32 = 26.0;
const THIGH: f32 = 78.0;
const SHIN: f32 = 78.0;

const GROUND_Y: f32 = -170.0;
const REAR_HUB: Vec2 = Vec2::new(-115.0, GROUND_Y + WHEEL_R);
const FRONT_HUB: Vec2 = Vec2::new(115.0, GROUND_Y + WHEEL_R);
const BB: Vec2 = Vec2::new(-12.0, -118.0);
const SEAT: Vec2 = Vec2::new(-52.0, -18.0);
const HEAD_TOP: Vec2 = Vec2::new(78.0, -10.0);
const HEAD_BOTTOM: Vec2 = Vec2::new(86.0, -38.0);
const GRIP: Vec2 = Vec2::new(70.0, 18.0);
const HIP: Vec2 = Vec2::new(-46.0, 2.0);

#[derive(Component)]
struct Scroll {
    base_x: f32,
    period: f32,
    speed: f32,
}

#[derive(Component)]
struct Wheel;

#[derive(Component)]
struct Crank {
    phase: f32,
}

#[derive(Component, Clone, Copy)]
enum LegPart {
    Thigh,
    Shin,
    Knee,
    Foot,
}

#[derive(Component)]
struct Leg {
    phase: f32,
    part: LegPart,
}

#[derive(Component)]
struct Rider;

#[derive(Component)]
struct Wing;

fn main() {
    App::new()
        .add_plugins(DemoPlugin {
            title: "Playmand · pelican riding a bicycle",
            loop_seconds: LOOP,
            fps: FPS,
        })
        .insert_resource(ClearColor(Color::srgb_u8(0x9f, 0xd8, 0xff)))
        .add_systems(Startup, setup)
        .add_systems(Update, (scroll, spin_wheels, pedal, bob_rider))
        .run();
}

struct Painter<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<ColorMaterial>,
}

impl Painter<'_, '_, '_> {
    fn shape(&mut self, mesh: impl Into<Mesh>, color: Color, transform: Transform) -> Entity {
        let mesh = self.meshes.add(mesh.into());
        let material = self.materials.add(color);
        self.commands
            .spawn((Mesh2d(mesh), MeshMaterial2d(material), transform))
            .id()
    }

    /// A bar with rounded ends; returns every entity it spawned.
    fn bar(&mut self, a: Vec2, b: Vec2, thickness: f32, color: Color, z: f32) -> Vec<Entity> {
        let mut out = vec![self.shape(
            Rectangle::new(1.0, thickness),
            color,
            bar_transform(a, b, z),
        )];
        for p in [a, b] {
            out.push(self.shape(
                Circle::new(thickness / 2.0),
                color,
                Transform::from_translation(p.extend(z)),
            ));
        }
        out
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    let mut p = Painter {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
    };

    // --- Sky, sun, clouds and hills -------------------------------------
    p.shape(
        Circle::new(46.0),
        Color::srgb_u8(0xff, 0xe0, 0x6b),
        Transform::from_xyz(330.0, 170.0, -10.0),
    );
    let cloud = Color::srgb_u8(0xff, 0xff, 0xff);
    for i in -2..=2 {
        let base_x = i as f32 * 320.0;
        for (dx, dy, rx, ry) in [
            (0.0, 0.0, 46.0, 22.0),
            (-34.0, -6.0, 30.0, 16.0),
            (36.0, -5.0, 32.0, 17.0),
        ] {
            let e = p.shape(
                Ellipse::new(rx, ry),
                cloud,
                Transform::from_xyz(0.0, 160.0 + dy, -9.0),
            );
            p.commands.entity(e).insert(Scroll {
                base_x: base_x + dx,
                period: 320.0,
                speed: 320.0 / LOOP,
            });
        }
    }
    for i in -2..=2 {
        for (dx, r, shade) in [(0.0, 150.0, 0x8f), (320.0, 115.0, 0xa6)] {
            let e = p.shape(
                Circle::new(r),
                Color::srgb_u8(0x6c, shade, 0x6a),
                Transform::from_xyz(0.0, GROUND_Y - 40.0, -8.0),
            );
            p.commands.entity(e).insert(Scroll {
                base_x: i as f32 * 640.0 + dx,
                period: 640.0,
                speed: 640.0 / LOOP,
            });
        }
    }

    // --- Road ------------------------------------------------------------
    p.shape(
        Rectangle::new(1200.0, 60.0),
        Color::srgb_u8(0x55, 0x58, 0x60),
        Transform::from_xyz(0.0, GROUND_Y - 30.0, -5.0),
    );
    p.shape(
        Rectangle::new(1200.0, 120.0),
        Color::srgb_u8(0x5f, 0x9e, 0x52),
        Transform::from_xyz(0.0, GROUND_Y - 120.0, -5.0),
    );
    let dash = SPEED * LOOP / 8.0;
    for i in -6..=6 {
        let e = p.shape(
            Rectangle::new(dash * 0.45, 5.0),
            Color::srgb_u8(0xf4, 0xf1, 0xe6),
            Transform::from_xyz(0.0, GROUND_Y - 30.0, -4.0),
        );
        p.commands.entity(e).insert(Scroll {
            base_x: i as f32 * dash,
            period: dash,
            speed: SPEED,
        });
    }

    // --- Bicycle -----------------------------------------------------------
    let tire = Color::srgb_u8(0x22, 0x24, 0x2a);
    let rim = Color::srgb_u8(0xc9, 0xcd, 0xd4);
    let frame = Color::srgb_u8(0xe8, 0x5a, 0x1e);
    for hub in [REAR_HUB, FRONT_HUB] {
        p.shape(
            Annulus::new(WHEEL_R - 7.0, WHEEL_R),
            tire,
            Transform::from_translation(hub.extend(1.0)),
        );
        p.shape(
            Annulus::new(WHEEL_R - 10.0, WHEEL_R - 7.0),
            rim,
            Transform::from_translation(hub.extend(1.0)),
        );
        let spoke_mesh = p.meshes.add(Rectangle::new((WHEEL_R - 8.0) * 2.0, 2.5));
        let spoke_mat = p.materials.add(rim);
        p.commands
            .spawn((
                Wheel,
                Transform::from_translation(hub.extend(1.1)),
                Visibility::default(),
            ))
            .with_children(|wheel| {
                for k in 0..4 {
                    wheel.spawn((
                        Mesh2d(spoke_mesh.clone()),
                        MeshMaterial2d(spoke_mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_z(k as f32 * PI / 4.0)),
                    ));
                }
            });
        p.shape(
            Circle::new(6.0),
            tire,
            Transform::from_translation(hub.extend(1.2)),
        );
    }
    for (a, b) in [
        (BB, SEAT),
        (SEAT, HEAD_TOP),
        (BB, HEAD_BOTTOM),
        (BB, REAR_HUB),
        (SEAT, REAR_HUB),
        (HEAD_TOP, HEAD_BOTTOM),
        (HEAD_BOTTOM, FRONT_HUB),
    ] {
        p.bar(a, b, 9.0, frame, 2.0);
    }
    p.bar(HEAD_TOP, Vec2::new(84.0, 14.0), 7.0, tire, 2.1);
    p.bar(Vec2::new(84.0, 14.0), GRIP, 8.0, tire, 2.1);
    p.shape(
        Capsule2d::new(6.0, 34.0),
        tire,
        Transform::from_xyz(SEAT.x - 2.0, SEAT.y + 6.0, 2.2)
            .with_rotation(Quat::from_rotation_z(PI / 2.0)),
    );
    p.shape(
        Circle::new(15.0),
        tire,
        Transform::from_translation(BB.extend(3.0)),
    );

    // --- Legs and cranks (far side behind the frame, near side in front) ---
    let leg_near = Color::srgb_u8(0xff, 0x9f, 0x1c);
    let leg_far = Color::srgb_u8(0xd9, 0x7a, 0x0b);
    for (phase, color, z) in [(PI, leg_far, 0.5), (0.0, leg_near, 6.0)] {
        let crank = p.shape(Rectangle::new(1.0, 6.0), tire, Transform::default());
        p.commands.entity(crank).insert(Crank { phase });
        for (part, mesh) in [
            (LegPart::Thigh, Mesh::from(Rectangle::new(1.0, 11.0))),
            (LegPart::Shin, Mesh::from(Rectangle::new(1.0, 8.0))),
            (LegPart::Knee, Mesh::from(Circle::new(6.0))),
            (LegPart::Foot, Mesh::from(Ellipse::new(13.0, 5.0))),
        ] {
            let e = p.shape(mesh, color, Transform::from_xyz(0.0, 0.0, z));
            p.commands.entity(e).insert(Leg { phase, part });
        }
    }

    // --- Pelican (bobs as one group) -------------------------------------
    let white = Color::srgb_u8(0xfb, 0xfa, 0xf6);
    let grey = Color::srgb_u8(0xc7, 0xcb, 0xd1);
    let dark_grey = Color::srgb_u8(0x7d, 0x84, 0x8e);
    let beak = Color::srgb_u8(0xff, 0xb3, 0x2b);
    let pouch = Color::srgb_u8(0xff, 0x8a, 0x3d);
    let ink = Color::srgb_u8(0x1b, 0x1d, 0x22);

    let root = p
        .commands
        .spawn((Rider, Transform::default(), Visibility::default()))
        .id();
    let mut parts = Vec::new();
    // Tail feathers
    parts.push(p.shape(
        Triangle2d::new(
            Vec2::new(0.0, 12.0),
            Vec2::new(0.0, -12.0),
            Vec2::new(-38.0, 6.0),
        ),
        grey,
        Transform::from_xyz(-98.0, 34.0, 3.9),
    ));
    // Body
    parts.push(p.shape(
        Ellipse::new(66.0, 42.0),
        white,
        Transform::from_xyz(-38.0, 38.0, 4.0).with_rotation(Quat::from_rotation_z(0.18)),
    ));
    // Neck: an S-curve of three segments
    let neck = [
        Vec2::new(4.0, 58.0),
        Vec2::new(16.0, 92.0),
        Vec2::new(6.0, 118.0),
        Vec2::new(22.0, 138.0),
    ];
    for w in neck.windows(2) {
        parts.extend(p.bar(w[0], w[1], 22.0, white, 4.1));
    }
    // Head
    parts.push(p.shape(
        Circle::new(21.0),
        white,
        Transform::from_xyz(28.0, 142.0, 4.3),
    ));
    parts.push(p.shape(
        Ellipse::new(9.0, 4.0),
        Color::srgb_u8(0xff, 0xe7, 0x9a),
        Transform::from_xyz(14.0, 160.0, 4.25).with_rotation(Quat::from_rotation_z(0.6)),
    ));
    // Pouch, then the long beak on top
    parts.push(p.shape(
        Ellipse::new(48.0, 14.0),
        pouch,
        Transform::from_xyz(86.0, 116.0, 4.4).with_rotation(Quat::from_rotation_z(-0.28)),
    ));
    parts.push(p.shape(
        Triangle2d::new(
            Vec2::new(0.0, 7.0),
            Vec2::new(0.0, -7.0),
            Vec2::new(108.0, -30.0),
        ),
        beak,
        Transform::from_xyz(40.0, 140.0, 4.5),
    ));
    // Eye
    parts.push(p.shape(Circle::new(4.5), ink, Transform::from_xyz(33.0, 148.0, 4.6)));
    parts.push(p.shape(
        Circle::new(1.5),
        white,
        Transform::from_xyz(34.5, 149.5, 4.7),
    ));
    // Wing, reaching for the handlebar
    let wing = p.shape(
        Ellipse::new(48.0, 22.0),
        grey,
        Transform::from_xyz(-46.0, 44.0, 4.8).with_rotation(Quat::from_rotation_z(0.25)),
    );
    p.commands.entity(wing).insert(Wing);
    parts.push(wing);
    parts.push(p.shape(
        Ellipse::new(22.0, 9.0),
        dark_grey,
        Transform::from_xyz(-82.0, 34.0, 4.85).with_rotation(Quat::from_rotation_z(0.35)),
    ));
    parts.extend(p.bar(
        Vec2::new(-10.0, 52.0),
        GRIP + Vec2::new(-4.0, 6.0),
        12.0,
        grey,
        7.0,
    ));
    parts.push(p.shape(
        Ellipse::new(10.0, 7.0),
        dark_grey,
        Transform::from_translation((GRIP + Vec2::new(-2.0, 4.0)).extend(7.1)),
    ));
    p.commands.entity(root).add_children(&parts);
}

fn scroll(clock: Res<Clock>, mut q: Query<(&Scroll, &mut Transform)>) {
    for (s, mut t) in &mut q {
        t.translation.x = s.base_x - (s.speed * clock.t) % s.period;
    }
}

fn spin_wheels(clock: Res<Clock>, mut q: Query<&mut Transform, With<Wheel>>) {
    let angle = -SPEED * clock.t / WHEEL_R;
    for mut t in &mut q {
        t.rotation = Quat::from_rotation_z(angle);
    }
}

fn crank_angle(t: f32) -> f32 {
    -TAU * CADENCE * t / LOOP
}

fn bob(t: f32) -> f32 {
    3.0 * (2.0 * crank_angle(t)).sin()
}

fn pedal(
    clock: Res<Clock>,
    mut cranks: Query<(&Crank, &mut Transform), Without<Leg>>,
    mut legs: Query<(&Leg, &mut Transform), Without<Crank>>,
) {
    let base = crank_angle(clock.t);
    for (c, mut t) in &mut cranks {
        let pedal = BB + Vec2::from_angle(base + c.phase) * CRANK_R;
        let z = if c.phase == 0.0 { 5.0 } else { 0.4 };
        *t = bar_transform(BB, pedal, z);
    }
    let hip = HIP + Vec2::Y * bob(clock.t);
    for (leg, mut t) in &mut legs {
        let foot = BB + Vec2::from_angle(base + leg.phase) * CRANK_R + Vec2::Y * 6.0;
        let knee = knee_position(hip, foot);
        let z = t.translation.z;
        *t = match leg.part {
            LegPart::Thigh => bar_transform(hip, knee, z),
            LegPart::Shin => bar_transform(knee, foot, z),
            LegPart::Knee => Transform::from_translation(knee.extend(z)),
            LegPart::Foot => Transform::from_translation((foot + Vec2::X * 6.0).extend(z)),
        };
    }
}

/// Two-bone IK with the knee bending forward.
fn knee_position(hip: Vec2, foot: Vec2) -> Vec2 {
    let d = (foot - hip).length().clamp(1.0, THIGH + SHIN - 0.01);
    let dir = (foot - hip).normalize();
    let a = (THIGH * THIGH - SHIN * SHIN + d * d) / (2.0 * d);
    let h = (THIGH * THIGH - a * a).max(0.0).sqrt();
    let mid = hip + dir * a;
    // Perpendicular pointing forward (+x) for a forward-bending knee.
    let perp = Vec2::new(-dir.y, dir.x);
    let perp = if perp.x < 0.0 { -perp } else { perp };
    mid + perp * h
}

fn bob_rider(
    clock: Res<Clock>,
    mut rider: Query<&mut Transform, (With<Rider>, Without<Wing>)>,
    mut wing: Query<&mut Transform, (With<Wing>, Without<Rider>)>,
) {
    for mut t in &mut rider {
        t.translation.y = bob(clock.t);
    }
    // A small wing flutter, twice per loop.
    let flutter = 0.08 * (TAU * 2.0 * clock.t / LOOP).sin();
    for mut t in &mut wing {
        t.rotation = Quat::from_rotation_z(0.25 + flutter);
    }
}
