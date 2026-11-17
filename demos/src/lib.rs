//! Shared helpers for the Playmand README demos.
//!
//! Every demo animates from [`Clock`], never from wall time. Interactive runs
//! advance the clock with real time; `--record <dir>` advances it by exactly
//! one frame per update and saves a screenshot of each frame, so the output
//! is identical on every machine and loops seamlessly.

use bevy::{
    prelude::*,
    render::view::screenshot::{Capturing, Screenshot, save_to_disk},
    window::WindowResolution,
};
use std::path::PathBuf;

/// Output size of recorded frames, in physical pixels.
pub const WIDTH: u32 = 960;
pub const HEIGHT: u32 = 540;

/// Animation time in seconds, looping every `loop_seconds`.
#[derive(Resource, Default)]
pub struct Clock {
    pub t: f32,
}

#[derive(Resource)]
struct Recording {
    dir: PathBuf,
    fps: u32,
    frames: u32,
    /// Updates to wait before the first capture, so pipelines and meshes are ready.
    warmup: u32,
    update: u32,
}

/// Window, clock and optional frame recorder for a demo.
pub struct DemoPlugin {
    pub title: &'static str,
    pub loop_seconds: f32,
    pub fps: u32,
}

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        let record_dir = std::env::args()
            .skip_while(|arg| arg != "--record")
            .nth(1)
            .map(PathBuf::from);

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: self.title.into(),
                resolution: WindowResolution::new(WIDTH, HEIGHT).with_scale_factor_override(1.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Clock>();

        match record_dir {
            Some(dir) => {
                std::fs::create_dir_all(&dir).expect("create the recording directory");
                app.insert_resource(Recording {
                    dir,
                    fps: self.fps,
                    frames: (self.loop_seconds * self.fps as f32).round() as u32,
                    warmup: 90,
                    update: 0,
                })
                .add_systems(First, step_recording_clock)
                .add_systems(Last, capture_frame);
            }
            None => {
                let period = self.loop_seconds;
                app.add_systems(First, move |time: Res<Time>, mut clock: ResMut<Clock>| {
                    clock.t = time.elapsed_secs() % period;
                });
            }
        }
    }
}

fn step_recording_clock(rec: Res<Recording>, mut clock: ResMut<Clock>) {
    let frame = rec.update.saturating_sub(rec.warmup);
    clock.t = frame as f32 / rec.fps as f32;
}

fn capture_frame(
    mut commands: Commands,
    mut rec: ResMut<Recording>,
    capturing: Query<(), With<Capturing>>,
    mut exit: MessageWriter<AppExit>,
) {
    let update = rec.update;
    rec.update += 1;
    if update < rec.warmup {
        return;
    }
    let frame = update - rec.warmup;
    if frame < rec.frames {
        let path = rec.dir.join(format!("frame-{frame:04}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    } else if capturing.is_empty() {
        exit.write(AppExit::Success);
    }
}

/// A 2D bar from `a` to `b`, used for bicycle tubes and limbs.
pub fn bar_transform(a: Vec2, b: Vec2, z: f32) -> Transform {
    let d = b - a;
    Transform {
        translation: ((a + b) / 2.0).extend(z),
        rotation: Quat::from_rotation_z(d.y.atan2(d.x)),
        scale: Vec3::new(d.length(), 1.0, 1.0),
    }
}
