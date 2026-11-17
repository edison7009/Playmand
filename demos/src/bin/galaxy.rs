//! A glowing spiral galaxy: 4,000 star entities moved by one ECS system.
//!
//! cargo run --bin galaxy                      # watch it
//! cargo run --bin galaxy -- --record frames   # save one seamless loop

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
};
use playmand_demos::{Clock, DemoPlugin};
use std::f32::consts::TAU;

const LOOP: f32 = 6.0;
const FPS: u32 = 25;
const STARS: usize = 4_000;
const ARMS: usize = 3;
const RADIUS: f32 = 250.0;
const TILT: f32 = 0.62; // squash y for a tilted-disc look

#[derive(Component)]
struct Star {
    radius: f32,
    angle: f32,
    /// Whole turns per loop, so every star is back in place when the loop ends.
    turns: f32,
    wobble: f32,
}

/// Small deterministic RNG so every run (and every recording) is identical.
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
            title: "Playmand · ECS galaxy",
            loop_seconds: LOOP,
            fps: FPS,
        })
        .insert_resource(ClearColor(Color::srgb_u8(0x05, 0x06, 0x0d)))
        .add_systems(Startup, setup)
        .add_systems(Update, orbit)
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

    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);

    // Faint background stars.
    let dot = meshes.add(Circle::new(1.0));
    let dim = materials.add(Color::srgb(0.35, 0.38, 0.5));
    for _ in 0..260 {
        let x = (rng.next() - 0.5) * 960.0;
        let y = (rng.next() - 0.5) * 540.0;
        commands.spawn((
            Mesh2d(dot.clone()),
            MeshMaterial2d(dim.clone()),
            Transform::from_xyz(x, y, -2.0).with_scale(Vec3::splat(0.6 + rng.next())),
        ));
    }

    // Glowing core.
    commands.spawn((
        Mesh2d(meshes.add(Ellipse::new(70.0, 70.0 * TILT))),
        MeshMaterial2d(materials.add(Color::srgba(1.0, 0.65, 0.35, 0.10))),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Ellipse::new(16.0, 16.0 * TILT))),
        MeshMaterial2d(materials.add(Color::srgb(9.0, 6.5, 3.5))),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    // Colour by radius: hot white-gold core, blue arms, magenta rim (HDR for bloom).
    let palette: Vec<Handle<ColorMaterial>> = (0..24)
        .map(|i| {
            let f = i as f32 / 23.0;
            let c = if f < 0.35 {
                let k = f / 0.35;
                Vec3::new(6.0, 4.2, 2.2).lerp(Vec3::new(2.4, 3.2, 6.0), k)
            } else {
                let k = (f - 0.35) / 0.65;
                Vec3::new(2.4, 3.2, 6.0).lerp(Vec3::new(4.5, 1.2, 4.8), k)
            };
            materials.add(Color::srgb(c.x, c.y, c.z))
        })
        .collect();
    let sizes = [
        meshes.add(Circle::new(1.1)),
        meshes.add(Circle::new(1.6)),
        meshes.add(Circle::new(2.3)),
    ];

    for i in 0..STARS {
        let arm = i % ARMS;
        // Denser towards the centre.
        let radius = 14.0 + RADIUS * rng.next().powf(1.6);
        let spread = (rng.next() - 0.5) * (0.55 + radius / RADIUS * 0.5);
        let angle = arm as f32 * TAU / ARMS as f32 + radius * 0.013 + spread;
        // Inner stars lap faster: 3, 2 or 1 whole turns per loop.
        let turns = if radius < 80.0 {
            3.0
        } else if radius < 160.0 {
            2.0
        } else {
            1.0
        };
        let shade = ((radius / (RADIUS + 14.0)) * 23.0) as usize;
        let size = (rng.next() * 3.0) as usize;
        commands.spawn((
            Star {
                radius,
                angle,
                turns,
                wobble: rng.next() * TAU,
            },
            Mesh2d(sizes[size.min(2)].clone()),
            MeshMaterial2d(palette[shade.min(23)].clone()),
            Transform::default(),
        ));
    }
}

/// The only per-frame system: every star follows its own orbit.
fn orbit(clock: Res<Clock>, mut stars: Query<(&Star, &mut Transform)>) {
    let phase = TAU * clock.t / LOOP;
    stars.par_iter_mut().for_each(|(star, mut t)| {
        let a = star.angle + phase * star.turns;
        let r = star.radius + 3.0 * (phase * 2.0 + star.wobble).sin();
        t.translation = Vec3::new(r * a.cos(), r * a.sin() * TILT, 0.0);
    });
}
