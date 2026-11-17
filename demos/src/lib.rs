//! Shared helpers for the Playmand README demos.
//!
//! Every demo animates from [`Clock`], never from wall time. Interactive runs
//! advance the clock with real time; `--record <dir>` advances it by exactly
//! one frame per update and saves a screenshot of each frame, so the output
//! is identical on every machine and loops seamlessly.

pub mod icon_points;

use bevy::{
    prelude::*,
    render::view::screenshot::{Capturing, Screenshot, save_to_disk},
    window::{WindowLevel, WindowResolution},
};
use std::path::PathBuf;

/// Logical size of every demo scene; recorded frames are this times `--scale`.
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
    /// Next frame to capture. The clock only advances once the previous
    /// screenshot has been read back, so heavy scenes never drop frames.
    frame: u32,
}

/// Window, clock and optional frame recorder for a demo.
pub struct DemoPlugin {
    pub title: &'static str,
    pub loop_seconds: f32,
    pub fps: u32,
}

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        let record_dir = arg_value("--record").map(PathBuf::from);
        // `--scale 2` renders the same 960x540 scene at 1920x1080 for video.
        let scale: f32 = arg_value("--scale")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        // `--from 212` resumes a recording at that frame (long 1080p captures
        // can stop presenting on macOS; re-run from the first bad frame).
        let from: u32 = arg_value("--from")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        // `--fps 30` records more frames per loop for smoother video.
        let fps: u32 = arg_value("--fps")
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.fps);

        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: self.title.into(),
                    resolution: WindowResolution::new(
                        (WIDTH as f32 * scale) as u32,
                        (HEIGHT as f32 * scale) as u32,
                    )
                    .with_scale_factor_override(scale),
                    resizable: false,
                    // A covered window stops presenting on macOS and screenshots
                    // come back black, so keep the recording window on top.
                    window_level: if record_dir.is_some() {
                        WindowLevel::AlwaysOnTop
                    } else {
                        WindowLevel::Normal
                    },
                    ..default()
                }),
                ..default()
            }),
        )
        .init_resource::<Clock>();

        match record_dir {
            Some(dir) => {
                std::fs::create_dir_all(&dir).expect("create the recording directory");
                app.insert_resource(Recording {
                    dir,
                    fps,
                    frames: (self.loop_seconds * fps as f32).round() as u32,
                    warmup: 90,
                    update: 0,
                    frame: from,
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

fn arg_value(flag: &str) -> Option<String> {
    std::env::args().skip_while(|arg| arg != flag).nth(1)
}

fn step_recording_clock(rec: Res<Recording>, mut clock: ResMut<Clock>) {
    clock.t = rec.frame as f32 / rec.fps as f32;
}

fn capture_frame(
    mut commands: Commands,
    mut rec: ResMut<Recording>,
    pending: Query<(), Or<(With<Screenshot>, With<Capturing>)>>,
    mut exit: MessageWriter<AppExit>,
) {
    rec.update += 1;
    // Wait for warmup, then keep at most one screenshot in flight: large or
    // bloom-heavy frames otherwise pile up in GPU readback and come back black.
    if rec.update <= rec.warmup || !pending.is_empty() {
        return;
    }
    if rec.frame < rec.frames {
        let path = rec.dir.join(format!("frame-{:04}.png", rec.frame));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        rec.frame += 1;
    } else {
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
