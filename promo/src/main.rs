//! Playmand promotional film.
//!
//! The picture is a Bevy scene animated only from [`Clock`]. Interactive
//! runs advance that clock from wall time; `--record <dir>` advances it by
//! exactly one frame per update and saves a screenshot, which is the video
//! path in `skills/playmand/references/video.md`.
//!
//! ```text
//! cargo run -- --record frames
//! ffmpeg -framerate 30 -i frames/frame-%04d.png -i assets/music/promo.wav \
//!     -c:v libx264 -pix_fmt yuv420p -crf 20 -c:a aac -shortest out.mp4
//! ```

use bevy::{
    asset::AssetPlugin,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
    prelude::*,
    render::view::screenshot::{Capturing, Screenshot, save_to_disk},
    window::{PresentMode, WindowPosition, WindowResolution},
};
use std::f32::consts::TAU;
use std::path::PathBuf;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const W: f32 = WIDTH as f32;

const SHOT_IDEA: (f32, f32) = (0.00, 4.28);
const SHOT_CODE: (f32, f32) = (3.72, 8.28);
const SHOT_SPLIT: (f32, f32) = (7.72, 16.28);
const SHOT_PUNCH: (f32, f32) = (15.72, 24.28);
const SHOT_END: (f32, f32) = (23.72, 80.0);

const CODE: &str = "\
App::new()\n\
    .add_systems(Update, play)\n\
    .add_systems(Last, record_frame)\n\
    .run();";

#[derive(Resource, Clone, Copy)]
struct Film {
    fps: u32,
    seconds: f32,
    frames: u32,
}

#[derive(Resource, Default)]
struct Clock {
    t: f32,
}

#[derive(Resource, Default)]
struct Gate {
    ready: bool,
}

#[derive(Resource)]
struct PromoAssets {
    regular: Handle<Font>,
    bold: Handle<Font>,
    mono: Handle<Font>,
    logo: Handle<Image>,
}

#[derive(Resource)]
struct Recording {
    dir: PathBuf,
    warmup: u32,
    update: u32,
}

#[derive(Component)]
struct Particle {
    a: f32,
    b: f32,
    c: f32,
    base: f32,
    bright: bool,
}

#[derive(Component)]
struct Orb {
    origin: Vec2,
    ax: f32,
    ay: f32,
    sx: f32,
    sy: f32,
    phase: f32,
    scale: f32,
}

#[derive(Component)]
struct GridLine {
    vertical: bool,
    index: f32,
}

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Pickup {
    phase: f32,
}

#[derive(Component)]
struct Streak;

#[derive(Component)]
struct Scan;

#[derive(Component)]
struct Cell {
    index: f32,
}

#[derive(Component)]
struct Shock;

#[derive(Component)]
struct GameRing;

#[derive(Component)]
struct Finder;

#[derive(Clone, Copy)]
struct Fade {
    start: f32,
    end: f32,
    base_top: f32,
    slide: f32,
}

impl Fade {
    fn new(span: (f32, f32), top: f32) -> Self {
        Self {
            start: span.0,
            end: span.1,
            base_top: top,
            slide: 22.0,
        }
    }

    fn still(top: f32) -> Self {
        Self {
            start: 0.0,
            end: 80.0,
            base_top: top,
            slide: 0.0,
        }
    }
}

#[derive(Component, Clone, Copy)]
struct FadeItem(Fade);

#[derive(Component, Clone, Copy)]
struct Tint {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Component)]
struct TypeOn {
    full: &'static str,
    start: f32,
    dur: f32,
}

#[derive(Component)]
struct Timecode;

#[derive(Component)]
struct Tick {
    at: f32,
    next: f32,
}

#[derive(Component)]
struct RailFill;

#[derive(Component)]
struct Playhead;

#[derive(Component)]
struct Accent;

#[derive(Component)]
struct LogoMark;

#[derive(Component)]
struct Plate {
    fill_a: f32,
    border_a: f32,
    r: f32,
    g: f32,
    b: f32,
}

struct Cli {
    record: Option<PathBuf>,
    fps: u32,
    seconds: f32,
}

fn main() {
    let cli = parse_cli();
    let frames = (cli.seconds * cli.fps as f32).round() as u32;
    let film = Film {
        fps: cli.fps,
        seconds: cli.seconds,
        frames,
    };

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Playmand".into(),
                    resolution: WindowResolution::new(WIDTH, HEIGHT)
                        .with_scale_factor_override(1.0),
                    resizable: false,
                    decorations: false,
                    present_mode: PresentMode::AutoNoVsync,
                    position: WindowPosition::At(IVec2::ZERO),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.027, 0.022, 0.055)))
    .insert_resource(film)
    .init_resource::<Clock>()
    .init_resource::<Gate>()
    .add_systems(Startup, setup);

    if let Some(dir) = cli.record {
        std::fs::create_dir_all(&dir).expect("create the recording directory");
        println!(
            "playmand promo: {WIDTH}x{HEIGHT} @ {}fps, {frames} frames, {:.2}s -> {}",
            cli.fps,
            cli.seconds,
            dir.display()
        );
        app.insert_resource(Recording {
            dir,
            warmup: 120,
            update: 0,
        })
        .add_systems(First, (poll_assets, step_recording_clock).chain())
        .add_systems(
            Update,
            (
                animate_field,
                animate_stage,
                animate_text,
                animate_plates,
                animate_logo,
                animate_rail,
            )
                .chain(),
        )
        .add_systems(Last, capture_frame);
    } else {
        app.add_systems(First, preview_clock).add_systems(
            Update,
            (
                animate_field,
                animate_stage,
                animate_text,
                animate_plates,
                animate_logo,
                animate_rail,
            )
                .chain(),
        );
    }

    app.run();
}

fn parse_cli() -> Cli {
    let mut record = None;
    let mut fps = 30u32;
    let mut seconds = 30.0f32;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--record" => record = args.next().map(PathBuf::from),
            "--fps" => {
                fps = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--fps needs a number");
            }
            "--seconds" => {
                seconds = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--seconds needs a number");
            }
            other => eprintln!("ignored argument: {other}"),
        }
    }
    Cli {
        record,
        fps,
        seconds,
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        Bloom {
            intensity: 0.45,
            low_frequency_boost: 0.62,
            low_frequency_boost_curvature: 0.45,
            high_pass_frequency: 1.0,
            prefilter: BloomPrefilter {
                threshold: 1.05,
                threshold_softness: 0.22,
            },
            composite_mode: BloomCompositeMode::Additive,
            ..Bloom::NATURAL
        },
    ));

    let regular = assets.load("fonts/PlaymandSans-Regular.ttf");
    let bold = assets.load("fonts/PlaymandSans-Bold.ttf");
    let mono = assets.load("fonts/NotoSansMono-Regular.ttf");
    let logo = assets.load("brand/logo.png");
    commands.insert_resource(PromoAssets {
        regular: regular.clone(),
        bold: bold.clone(),
        mono: mono.clone(),
        logo: logo.clone(),
    });

    let circle = meshes.add(Circle::new(1.0));
    let rect = meshes.add(Rectangle::new(1.0, 1.0));
    let ring = meshes.add(Annulus::new(148.0, 151.5));
    let finder = meshes.add(Rectangle::new(1.0, 1.0));
    let ship = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 16.0),
        Vec2::new(-11.0, -13.0),
        Vec2::new(11.0, -13.0),
    ));

    let grid_mat = materials.add(Color::srgba(0.55, 0.82, 1.0, 0.055));
    let dim = materials.add(Color::srgb(0.45, 0.62, 0.95));
    let bright = [
        materials.add(Color::srgb(1.5, 3.6, 6.2)),
        materials.add(Color::srgb(6.2, 1.15, 4.4)),
        materials.add(Color::srgb(6.4, 3.5, 1.15)),
        materials.add(Color::srgb(2.4, 4.8, 6.4)),
    ];
    let flare_mat = materials.add(Color::srgba(1.3, 2.4, 4.8, 0.28));
    let orb_mats = [
        materials.add(Color::srgba(0.12, 0.28, 0.72, 0.28)),
        materials.add(Color::srgba(0.48, 0.10, 0.38, 0.20)),
        materials.add(Color::srgba(0.42, 0.24, 0.08, 0.16)),
    ];
    let line_mat = materials.add(Color::srgb(1.8, 4.2, 6.0));
    let finder_mat = materials.add(Color::srgb(1.2, 2.6, 4.4));
    let ship_mat = materials.add(Color::srgb(7.0, 8.5, 9.5));
    let pickup_mat = materials.add(Color::srgb(6.5, 2.2, 4.8));
    let shock_mat = materials.add(Color::srgba(0.7, 0.95, 1.0, 0.0));

    for (origin, phase, mat, ax, ay, sx, sy, scale) in [
        (
            Vec2::new(-900.0, 40.0),
            0.2,
            orb_mats[0].clone(),
            36.0,
            28.0,
            0.11,
            0.08,
            130.0,
        ),
        (
            Vec2::new(760.0, -300.0),
            2.1,
            orb_mats[1].clone(),
            40.0,
            36.0,
            0.08,
            0.12,
            190.0,
        ),
        (
            Vec2::new(-760.0, -260.0),
            4.2,
            orb_mats[2].clone(),
            36.0,
            24.0,
            0.14,
            0.07,
            150.0,
        ),
    ] {
        commands.spawn((
            Mesh2d(circle.clone()),
            MeshMaterial2d(mat),
            Transform::from_xyz(origin.x, origin.y, -3.0).with_scale(Vec3::splat(scale)),
            Orb {
                origin,
                ax,
                ay,
                sx,
                sy,
                phase,
                scale,
            },
        ));
    }

    for i in -9..=9 {
        commands.spawn((
            Mesh2d(rect.clone()),
            MeshMaterial2d(grid_mat.clone()),
            Transform::from_xyz(0.0, i as f32 * 70.0, -4.0).with_scale(Vec3::new(2100.0, 1.0, 1.0)),
            GridLine {
                vertical: false,
                index: i as f32,
            },
        ));
    }
    for i in -7..=7 {
        commands.spawn((
            Mesh2d(rect.clone()),
            MeshMaterial2d(grid_mat.clone()),
            Transform::from_xyz(i as f32 * 140.0, 0.0, -4.0)
                .with_scale(Vec3::new(1.0, 1200.0, 1.0)),
            GridLine {
                vertical: true,
                index: i as f32,
            },
        ));
    }

    let mut rng = Rng(0x5A17_C0DE_1234_89AB);
    for i in 0..460 {
        let a = rng.next();
        let b = rng.next();
        let c = rng.next();
        let bright_on = a > 0.74;
        let material = if bright_on {
            bright[(i % 4) as usize].clone()
        } else {
            dim.clone()
        };
        let base = if bright_on {
            2.1 + c * 1.8
        } else {
            1.35 + b * 1.1
        };
        commands.spawn((
            Particle {
                a,
                b,
                c,
                base,
                bright: bright_on,
            },
            Mesh2d(circle.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
    }
    for _ in 0..14 {
        let a = rng.next();
        let b = rng.next();
        let c = rng.next();
        commands.spawn((
            Particle {
                a,
                b,
                c,
                base: 11.0 + c * 8.0,
                bright: true,
            },
            Mesh2d(circle.clone()),
            MeshMaterial2d(flare_mat.clone()),
            Transform::from_xyz(0.0, 0.0, -0.2),
        ));
    }

    commands.spawn((
        Mesh2d(ring),
        MeshMaterial2d(line_mat.clone()),
        Transform::from_xyz(-500.0, -30.0, 0.4).with_scale(Vec3::new(1.28, 0.78, 1.0)),
        GameRing,
    ));
    commands.spawn((
        Mesh2d(ship),
        MeshMaterial2d(ship_mat),
        Transform::from_xyz(-500.0, -30.0, 1.2),
        Ship,
    ));
    commands.spawn((
        Mesh2d(rect.clone()),
        MeshMaterial2d(line_mat.clone()),
        Transform::from_xyz(-500.0, -30.0, 1.0),
        Streak,
    ));
    for phase in [0.4, 2.6] {
        commands.spawn((
            Mesh2d(rect.clone()),
            MeshMaterial2d(pickup_mat.clone()),
            Transform::from_xyz(-500.0, -30.0, 0.8).with_scale(Vec3::splat(11.0)),
            Pickup { phase },
        ));
    }

    // Viewfinder: four bars of a 500×280 frame, plus a scan line and film cells.
    let finder_center = Vec2::new(500.0, -30.0);
    let fw = 500.0;
    let fh = 280.0;
    for (pos, scale) in [
        (Vec2::new(0.0, fh * 0.5), Vec3::new(fw, 2.4, 1.0)),
        (Vec2::new(0.0, -fh * 0.5), Vec3::new(fw, 2.4, 1.0)),
        (Vec2::new(-fw * 0.5, 0.0), Vec3::new(2.4, fh, 1.0)),
        (Vec2::new(fw * 0.5, 0.0), Vec3::new(2.4, fh, 1.0)),
    ] {
        commands.spawn((
            Mesh2d(finder.clone()),
            MeshMaterial2d(finder_mat.clone()),
            Transform::from_translation((finder_center + pos).extend(0.5)).with_scale(scale),
            Finder,
        ));
    }
    commands.spawn((
        Mesh2d(finder.clone()),
        MeshMaterial2d(line_mat.clone()),
        Transform::from_xyz(finder_center.x, finder_center.y, 0.7).with_scale(Vec3::new(
            2.0,
            fh - 16.0,
            1.0,
        )),
        Scan,
    ));
    for i in 0..12 {
        commands.spawn((
            Mesh2d(finder.clone()),
            MeshMaterial2d(bright[i % 4].clone()),
            Transform::from_xyz(0.0, 0.0, 0.6),
            Cell { index: i as f32 },
        ));
    }

    commands.spawn((
        Mesh2d(meshes.add(Annulus::new(40.0, 43.0))),
        MeshMaterial2d(shock_mat),
        Transform::from_xyz(0.0, 0.0, 1.6),
        Shock,
    ));

    spawn_ui(&mut commands, &regular, &bold, &mono, &logo);
}

fn spawn_ui(
    commands: &mut Commands,
    regular: &Handle<Font>,
    bold: &Handle<Font>,
    mono: &Handle<Font>,
    logo: &Handle<Image>,
) {
    let white = Tint {
        r: 0.96,
        g: 0.97,
        b: 1.0,
    };
    let muted = Tint {
        r: 0.62,
        g: 0.70,
        b: 0.84,
    };
    let cyan = Tint {
        r: 0.55,
        g: 0.92,
        b: 1.0,
    };
    let amber = Tint {
        r: 1.0,
        g: 0.78,
        b: 0.48,
    };

    // Chrome that stays up for the whole film.
    commands.spawn((
        ImageNode::new(logo.clone()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(48.0),
            top: Val::Px(32.0),
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            ..default()
        },
        GlobalZIndex(4),
    ));
    spawn_text(
        commands,
        bold,
        22.0,
        "PLAYMAND",
        white,
        Fade::still(42.0),
        116.0,
        116.0,
        280.0,
        Justify::Left,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        18.0,
        "Rust + Bevy  ·  Games & Videos",
        muted,
        Fade::still(46.0),
        1420.0,
        1420.0,
        440.0,
        Justify::Right,
        None,
        false,
    );
    spawn_text(
        commands,
        mono,
        18.0,
        "00:00.00",
        cyan,
        Fade::still(74.0),
        1560.0,
        1560.0,
        300.0,
        Justify::Right,
        None,
        true,
    );

    spawn_shot_copy(commands, regular, bold, white, muted, cyan, amber);
    spawn_code_card(commands, mono, cyan);
    spawn_rail(commands, regular);
    spawn_end_card(commands, bold, regular, mono, logo, white, muted, cyan);
}

fn spawn_shot_copy(
    commands: &mut Commands,
    regular: &Handle<Font>,
    bold: &Handle<Font>,
    white: Tint,
    muted: Tint,
    cyan: Tint,
    amber: Tint,
) {
    spawn_text(
        commands,
        regular,
        24.0,
        "从一句想法",
        cyan,
        Fade::new(SHOT_IDEA, 348.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        bold,
        84.0,
        "做一支会发光的宣传片",
        white,
        Fade::new(SHOT_IDEA, 408.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        30.0,
        "写下来，交给会写 Bevy 的 Agent",
        muted,
        Fade::new(SHOT_IDEA, 548.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );

    spawn_text(
        commands,
        regular,
        24.0,
        "代码驱动",
        amber,
        Fade::new(SHOT_CODE, 148.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        bold,
        68.0,
        "场景写在 Rust 里",
        white,
        Fade::new(SHOT_CODE, 196.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        28.0,
        "组件、系统、状态，都在代码里",
        muted,
        Fade::new(SHOT_CODE, 760.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );

    spawn_text(
        commands,
        regular,
        24.0,
        "游戏  ×  视频",
        cyan,
        Fade::new(SHOT_SPLIT, 78.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        bold,
        64.0,
        "同一套场景代码",
        white,
        Fade::new(SHOT_SPLIT, 122.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        bold,
        52.0,
        "游戏",
        white,
        Fade::new(SHOT_SPLIT, 214.0),
        140.0,
        140.0,
        640.0,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        26.0,
        "实时游玩",
        cyan,
        Fade::new(SHOT_SPLIT, 278.0),
        140.0,
        140.0,
        640.0,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        bold,
        52.0,
        "视频",
        white,
        Fade::new(SHOT_SPLIT, 214.0),
        1140.0,
        1140.0,
        640.0,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        26.0,
        "固定时钟逐帧录制",
        amber,
        Fade::new(SHOT_SPLIT, 278.0),
        1140.0,
        1140.0,
        640.0,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        28.0,
        "能跑的画面，再逐帧收成片子",
        muted,
        Fade::new(SHOT_SPLIT, 852.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );

    spawn_text(
        commands,
        regular,
        36.0,
        "AI 时代",
        cyan,
        Fade::new(SHOT_PUNCH, 318.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        bold,
        100.0,
        "创作王炸 Skill",
        white,
        Fade::new(SHOT_PUNCH, 378.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        34.0,
        "游戏与视频，同一套 Bevy 场景",
        muted,
        Fade::new(SHOT_PUNCH, 548.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        28.0,
        "你定风格，Agent 写场景",
        amber,
        Fade::new(SHOT_PUNCH, 628.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(W * 0.5),
            top: Val::Px(512.0),
            width: Val::Px(0.0),
            height: Val::Px(3.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.55, 0.92, 1.0, 0.0)),
        GlobalZIndex(3),
        FadeItem(Fade::new(SHOT_PUNCH, 512.0)),
        Accent,
    ));
}

fn spawn_code_card(commands: &mut Commands, mono: &Handle<Font>, cyan: Tint) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(430.0),
            top: Val::Px(330.0),
            width: Val::Px(1060.0),
            height: Val::Px(390.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(22.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.07, 0.0)),
        BorderColor::all(Color::srgba(0.55, 0.9, 1.0, 0.0)),
        GlobalZIndex(1),
        FadeItem(Fade::new(SHOT_CODE, 330.0)),
        Plate {
            fill_a: 0.94,
            border_a: 0.45,
            r: 0.03,
            g: 0.04,
            b: 0.09,
        },
    ));
    for (i, (r, g, b)) in [(1.0, 0.45, 0.62), (1.0, 0.78, 0.42), (0.45, 0.9, 1.0)]
        .into_iter()
        .enumerate()
    {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(462.0 + i as f32 * 26.0),
                top: Val::Px(354.0),
                width: Val::Px(12.0),
                height: Val::Px(12.0),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(r, g, b, 0.0)),
            GlobalZIndex(2),
            FadeItem(Fade::new(SHOT_CODE, 354.0)),
            Plate {
                fill_a: 0.95,
                border_a: 0.0,
                r,
                g,
                b,
            },
        ));
    }
    spawn_text(
        commands,
        mono,
        22.0,
        "// fixed clock  ->  one frame, one picture",
        Tint {
            r: 0.55,
            g: 0.78,
            b: 0.72,
        },
        Fade::new(SHOT_CODE, 392.0),
        480.0,
        480.0,
        960.0,
        Justify::Left,
        None,
        false,
    );
    spawn_text(
        commands,
        mono,
        34.0,
        "",
        cyan,
        Fade::new(SHOT_CODE, 440.0),
        480.0,
        480.0,
        960.0,
        Justify::Left,
        Some(TypeOn {
            full: CODE,
            start: 4.05,
            dur: 2.7,
        }),
        false,
    );
}

fn spawn_rail(commands: &mut Commands, regular: &Handle<Font>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(120.0),
            top: Val::Px(996.0),
            width: Val::Px(1680.0),
            height: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.16)),
        GlobalZIndex(3),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(120.0),
            top: Val::Px(996.0),
            width: Val::Px(0.0),
            height: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.55, 0.92, 1.0)),
        GlobalZIndex(3),
        RailFill,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(116.0),
            top: Val::Px(990.0),
            width: Val::Px(14.0),
            height: Val::Px(14.0),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.95, 0.97, 1.0)),
        GlobalZIndex(4),
        Playhead,
    ));

    let chapters = [
        (0.0, 4.0, "想法"),
        (4.0, 8.0, "代码"),
        (8.0, 16.0, "成片"),
        (16.0, 24.0, "定位"),
        (24.0, 30.01, "品牌"),
    ];
    for (at, next, label) in chapters {
        let x = rail_x(at);
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x - 4.0),
                top: Val::Px(992.0),
                width: Val::Px(10.0),
                height: Val::Px(10.0),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.62, 0.70, 0.84, 1.0)),
            GlobalZIndex(4),
            Tick { at, next },
        ));
        spawn_text(
            commands,
            regular,
            18.0,
            label,
            Tint {
                r: 0.62,
                g: 0.70,
                b: 0.84,
            },
            Fade::still(1014.0),
            x - 48.0,
            x - 48.0,
            96.0,
            Justify::Center,
            None,
            false,
        );
    }
}

fn spawn_end_card(
    commands: &mut Commands,
    bold: &Handle<Font>,
    regular: &Handle<Font>,
    mono: &Handle<Font>,
    logo: &Handle<Image>,
    white: Tint,
    muted: Tint,
    cyan: Tint,
) {
    commands.spawn((
        ImageNode::new(logo.clone()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px((W - 168.0) * 0.5),
            top: Val::Px(268.0),
            width: Val::Px(168.0),
            height: Val::Px(168.0),
            ..default()
        },
        GlobalZIndex(3),
        FadeItem(Fade::new(SHOT_END, 268.0)),
        LogoMark,
    ));
    spawn_text(
        commands,
        bold,
        92.0,
        "PLAYMAND",
        white,
        Fade::new(SHOT_END, 468.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        regular,
        32.0,
        "Rust + Bevy Skill  ·  Games & Videos",
        muted,
        Fade::new(SHOT_END, 590.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
    spawn_text(
        commands,
        mono,
        30.0,
        "github.com/edison7009/Playmand",
        cyan,
        Fade::new(SHOT_END, 668.0),
        0.0,
        0.0,
        W,
        Justify::Center,
        None,
        false,
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_text(
    commands: &mut Commands,
    font: &Handle<Font>,
    size: f32,
    text: &str,
    tint: Tint,
    fade: Fade,
    left: f32,
    _left_base: f32,
    width: f32,
    justify: Justify,
    type_on: Option<TypeOn>,
    timecode: bool,
) {
    let mut entity = commands.spawn((
        Text::new(text),
        TextFont::from_font_size(FontSize::Px(size)).with_font(font.clone()),
        TextColor(Color::srgba(tint.r, tint.g, tint.b, 0.0)),
        TextLayout::justify(justify),
        TextShadow {
            offset: Vec2::new(0.0, 2.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(fade.base_top),
            width: Val::Px(width),
            ..default()
        },
        GlobalZIndex(if fade.slide == 0.0 { 4 } else { 2 }),
        FadeItem(fade),
        tint,
    ));
    if let Some(type_on) = type_on {
        entity.insert(type_on);
    }
    if timecode {
        entity.insert(Timecode);
    }
}

fn poll_assets(
    assets: Res<PromoAssets>,
    server: Res<AssetServer>,
    mut gate: ResMut<Gate>,
    mut ticks: Local<u32>,
) {
    *ticks += 1;
    let ready = server.is_loaded_with_dependencies(assets.regular.id())
        && server.is_loaded_with_dependencies(assets.bold.id())
        && server.is_loaded_with_dependencies(assets.mono.id())
        && server.is_loaded_with_dependencies(assets.logo.id());
    if ready {
        gate.ready = true;
    } else if *ticks % 90 == 0 {
        println!(
            "waiting for assets: regular {:?} bold {:?} mono {:?} logo {:?}",
            server.load_state(assets.regular.id()),
            server.load_state(assets.bold.id()),
            server.load_state(assets.mono.id()),
            server.load_state(assets.logo.id()),
        );
    }
}

fn step_recording_clock(
    gate: Res<Gate>,
    rec: Res<Recording>,
    film: Res<Film>,
    mut clock: ResMut<Clock>,
) {
    if !gate.ready {
        return;
    }
    let frame = rec.update.saturating_sub(rec.warmup);
    clock.t = frame as f32 / film.fps as f32;
}

fn preview_clock(time: Res<Time>, film: Res<Film>, mut clock: ResMut<Clock>) {
    clock.t = time.elapsed_secs() % film.seconds;
}

fn capture_frame(
    mut commands: Commands,
    gate: Res<Gate>,
    film: Res<Film>,
    mut rec: ResMut<Recording>,
    capturing: Query<(), With<Capturing>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !gate.ready {
        return;
    }
    let update = rec.update;
    rec.update += 1;
    if update < rec.warmup {
        return;
    }
    let frame = update - rec.warmup;
    if frame < film.frames {
        let path = rec.dir.join(format!("frame-{frame:04}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    } else if capturing.is_empty() {
        println!("recorded {} frames", film.frames);
        exit.write(AppExit::Success);
    }
}

fn animate_field(
    clock: Res<Clock>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut set: ParamSet<(
        Query<(&Particle, &mut Transform)>,
        Query<(&Orb, &mut Transform)>,
        Query<(&GridLine, &mut Transform)>,
        Query<(&MeshMaterial2d<ColorMaterial>, &mut Transform), With<Shock>>,
    )>,
) {
    let t = clock.t;

    set.p0()
        .par_iter_mut()
        .for_each(|(particle, mut transform)| {
            let pos = particle_pos(particle, t);
            let beat = (-((t / 0.5).fract()) * 8.0).exp();
            let energy = if (4.0..24.0).contains(&t) { 1.0 } else { 0.25 };
            let punch = if particle.bright { 0.45 } else { 0.12 };
            transform.translation = pos.extend(0.1);
            transform.scale = Vec3::splat(particle.base * (1.0 + punch * beat * energy));
        });

    for (orb, mut transform) in &mut set.p1() {
        let x = orb.origin.x + (t * orb.sx + orb.phase).sin() * orb.ax;
        let y = orb.origin.y + (t * orb.sy + orb.phase * 1.7).cos() * orb.ay;
        transform.translation = Vec3::new(x, y, -3.0);
        transform.scale = Vec3::splat(orb.scale);
    }

    for (line, mut transform) in &mut set.p2() {
        let drift = (t * 18.0) % 70.0;
        if line.vertical {
            transform.translation.x = line.index * 140.0 + (t * 12.0).sin() * 6.0;
            transform.translation.y = 0.0;
        } else {
            transform.translation.y = line.index * 70.0 - drift;
        }
    }

    if let Ok((material, mut transform)) = set.p3().single_mut() {
        let handle = material.0.clone();
        let mut age: Option<f32> = None;
        for cut in [4.0, 8.0, 16.0, 24.0] {
            let d = t - cut;
            if (0.0..0.7).contains(&d) {
                age = Some(d);
            }
        }
        if let Some(d) = age {
            let k = d / 0.7;
            transform.scale = Vec3::splat(0.25 + k * 14.0);
            if let Some(mut mat) = materials.get_mut(&handle) {
                mat.color = Color::srgba(0.65, 0.9, 1.0, (1.0 - k) * 0.45);
            }
        } else {
            transform.scale = Vec3::splat(0.001);
        }
    }
}

fn animate_stage(
    clock: Res<Clock>,
    mut set: ParamSet<(
        Query<&mut Transform, With<GameRing>>,
        Query<&mut Transform, With<Ship>>,
        Query<&mut Transform, With<Streak>>,
        Query<(&Pickup, &mut Transform)>,
        Query<&mut Transform, With<Finder>>,
        Query<&mut Transform, With<Scan>>,
        Query<(&Cell, &mut Transform)>,
    )>,
) {
    let t = clock.t;
    let show_game = fade_alpha(t, SHOT_SPLIT.0, SHOT_SPLIT.1);

    if let Ok(mut transform) = set.p0().single_mut() {
        transform.translation = Vec3::new(-500.0, -30.0, 0.4);
        transform.scale = Vec3::new(1.28, 0.78, 1.0) * show_game.max(0.001);
    }

    let angle = t * 1.35;
    let ship_pos = Vec2::new(-500.0, -30.0) + Vec2::new(angle.cos() * 190.0, angle.sin() * 116.0);
    let tangent = Vec2::new(-angle.sin() * 190.0, angle.cos() * 116.0);
    if let Ok(mut transform) = set.p1().single_mut() {
        transform.translation = ship_pos.extend(1.2);
        transform.rotation =
            Quat::from_rotation_z(tangent.y.atan2(tangent.x) - std::f32::consts::FRAC_PI_2);
        transform.scale = Vec3::splat(show_game.max(0.001));
    }
    let dash = {
        let x = (t / 1.1).fract();
        if x < 0.16 { 1.0 - x / 0.16 } else { 0.0 }
    };
    if let Ok(mut transform) = set.p2().single_mut() {
        let len = 70.0 * dash * show_game;
        transform.translation = (ship_pos - tangent.normalize_or_zero() * 28.0).extend(1.0);
        transform.rotation = Quat::from_rotation_z(tangent.y.atan2(tangent.x));
        transform.scale = Vec3::new(len.max(0.001), 3.0, 1.0);
    }
    for (pickup, mut transform) in &mut set.p3() {
        let a = t * 0.7 + pickup.phase;
        let pos = Vec2::new(-500.0, -30.0) + Vec2::new(a.cos() * 150.0, a.sin() * 92.0);
        transform.translation = pos.extend(0.8);
        transform.rotation = Quat::from_rotation_z(t * 1.4 + pickup.phase);
        let pulse = 1.0 + 0.25 * (t * 3.0 + pickup.phase).sin();
        transform.scale = Vec3::splat(11.0 * pulse * show_game.max(0.001));
    }

    for mut transform in &mut set.p4() {
        let center = Vec2::new(500.0, -30.0);
        let offset = transform.translation.truncate() - center;
        let full = if offset.x.abs() > offset.y.abs() {
            Vec2::new(2.4, 280.0)
        } else {
            Vec2::new(500.0, 2.4)
        };
        transform.scale = Vec3::new(full.x, full.y, 1.0) * show_game.max(0.001);
    }
    if let Ok(mut transform) = set.p5().single_mut() {
        let x = (t * 0.55).fract();
        let x = if ((t * 0.55) as i32).rem_euclid(2) == 0 {
            x
        } else {
            1.0 - x
        };
        transform.translation = Vec3::new(500.0 - 230.0 + x * 460.0, -30.0, 0.7);
        transform.scale = Vec3::new(2.0, 250.0, 1.0) * show_game.max(0.001);
    }
    for (cell, mut transform) in &mut set.p6() {
        let x = 500.0 - 210.0 + cell.index * 38.0;
        let lit = (t * 2.0) as i32 % 12 == cell.index as i32;
        let s = if lit { 16.0 } else { 8.0 };
        transform.translation = Vec3::new(x, -128.0, 0.6);
        transform.scale = Vec3::splat(s * show_game.max(0.001));
    }
}

fn particle_pos(p: &Particle, t: f32) -> Vec2 {
    let targets = [
        idea_pos(p, t),
        code_pos(p, t),
        split_pos(p, t),
        punch_pos(p, t),
        end_pos(p, t),
    ];
    let cuts = [0.0, 4.0, 8.0, 16.0, 24.0];
    let mut seg = 0;
    for (index, cut) in cuts.iter().enumerate() {
        if t >= *cut {
            seg = index;
        }
    }
    if seg == 0 {
        return targets[0];
    }
    let k = smooth((t - cuts[seg]) / 0.55);
    targets[seg - 1].lerp(targets[seg], k)
}

fn idea_pos(p: &Particle, t: f32) -> Vec2 {
    if p.a > 0.84 {
        return Vec2::new((p.b - 0.5) * 1800.0, (p.c - 0.5) * 980.0);
    }
    let x = (p.b - 0.5) * 1500.0;
    let y = -210.0 + (p.c - 0.5) * 80.0;
    let wave = (x * 0.012 + t * 1.6 + p.a * TAU).sin() * 28.0;
    Vec2::new(x, y + wave)
}

fn code_pos(p: &Particle, t: f32) -> Vec2 {
    let row = (p.a * 8.0).floor();
    let y = -340.0 + row * 95.0;
    let span = 2100.0;
    let x = ((p.b * span + t * (180.0 + p.c * 140.0)) % span) - span * 0.5;
    Vec2::new(x, y)
}

fn split_pos(p: &Particle, t: f32) -> Vec2 {
    if p.a < 0.52 {
        let ang = p.b * TAU + t * (0.45 + p.c);
        let rx = 150.0 + p.c * 70.0;
        let ry = 90.0 + p.b * 40.0;
        Vec2::new(-500.0 + ang.cos() * rx, -30.0 + ang.sin() * ry)
    } else {
        let col = (p.b * 12.0).floor();
        let row = (p.c * 7.0).floor();
        Vec2::new(500.0 - 230.0 + col * 40.0, 90.0 - row * 36.0)
    }
}

fn punch_pos(p: &Particle, t: f32) -> Vec2 {
    let ang = p.b * TAU + t * (0.28 + p.a * 0.35);
    let radius = 340.0 + p.c * 210.0;
    Vec2::new(ang.cos() * radius * 1.2, 30.0 + ang.sin() * radius * 0.62)
}

fn end_pos(p: &Particle, t: f32) -> Vec2 {
    if p.a > 0.4 {
        let x = (p.b - 0.5) * 1900.0 + (t * 0.4 + p.c).sin() * 10.0;
        let y = (p.c - 0.5) * 1000.0;
        return Vec2::new(x, y);
    }
    let ang = p.b * TAU + t * 0.35;
    let radius = 128.0 + p.c * 22.0;
    Vec2::new(ang.cos() * radius, 200.0 + ang.sin() * radius)
}

fn animate_text(
    clock: Res<Clock>,
    film: Res<Film>,
    mut query: Query<(
        &FadeItem,
        &Tint,
        Option<&TypeOn>,
        Option<&Timecode>,
        &mut Text,
        &mut TextColor,
        &mut TextShadow,
        &mut Node,
    )>,
) {
    let t = clock.t;
    for (fade, tint, type_on, timecode, mut text, mut color, mut shadow, mut node) in &mut query {
        let alpha = fade_alpha(t, fade.0.start, fade.0.end);
        let appear = ease_out(((t - fade.0.start) / 0.38).clamp(0.0, 1.0));
        node.top = Val::Px(fade.0.base_top + (1.0 - appear) * fade.0.slide);
        color.0 = Color::srgba(tint.r, tint.g, tint.b, alpha);
        shadow.color = Color::srgba(0.0, 0.0, 0.0, alpha * 0.72);
        if timecode.is_some() {
            text.0 = format_timecode(t.min(film.seconds));
        } else if let Some(type_on) = type_on {
            let progress = ((t - type_on.start) / type_on.dur).clamp(0.0, 1.0);
            let blink = ((t * 3.0) as i32) % 2 == 0;
            text.0 = typed(type_on.full, progress, blink && progress < 1.0);
        }
    }
}

fn animate_plates(
    clock: Res<Clock>,
    mut plates: Query<
        (
            &FadeItem,
            &Plate,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Node,
        ),
        Without<Accent>,
    >,
    mut accent: Query<(&FadeItem, &mut Node, &mut BackgroundColor), With<Accent>>,
) {
    let t = clock.t;
    for (fade, plate, mut background, mut border, mut node) in &mut plates {
        let alpha = fade_alpha(t, fade.0.start, fade.0.end);
        let appear = ease_out(((t - fade.0.start) / 0.38).clamp(0.0, 1.0));
        node.top = Val::Px(fade.0.base_top + (1.0 - appear) * fade.0.slide);
        background.0 = Color::srgba(plate.r, plate.g, plate.b, alpha * plate.fill_a);
        *border = BorderColor::all(Color::srgba(0.55, 0.9, 1.0, alpha * plate.border_a));
    }
    for (fade, mut node, mut background) in &mut accent {
        let alpha = fade_alpha(t, fade.0.start, fade.0.end);
        let local = (t - fade.0.start - 0.2).max(0.0);
        let width = 460.0 * ease_out((local / 0.45).clamp(0.0, 1.0));
        node.width = Val::Px(width);
        node.left = Val::Px((W - width) * 0.5);
        background.0 = Color::srgba(0.55, 0.92, 1.0, alpha);
    }
}

fn animate_logo(
    clock: Res<Clock>,
    mut query: Query<(&FadeItem, &mut ImageNode, &mut Node), With<LogoMark>>,
) {
    let t = clock.t;
    for (fade, mut image, mut node) in &mut query {
        let alpha = fade_alpha(t, fade.0.start, fade.0.end);
        let appear = ease_out(((t - fade.0.start) / 0.42).clamp(0.0, 1.0));
        node.top = Val::Px(fade.0.base_top + (1.0 - appear) * 18.0);
        image.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

fn animate_rail(
    clock: Res<Clock>,
    film: Res<Film>,
    mut fill: Query<&mut Node, With<RailFill>>,
    mut head: Query<&mut Node, (With<Playhead>, Without<RailFill>)>,
    mut ticks: Query<(&Tick, &mut BackgroundColor), Without<Text>>,
) {
    let t = clock.t.min(film.seconds);
    let x = rail_x(t / film.seconds * 30.0);
    if let Ok(mut node) = fill.single_mut() {
        node.width = Val::Px((x - 120.0).max(0.0));
    }
    if let Ok(mut node) = head.single_mut() {
        node.left = Val::Px(x - 7.0);
        let pulse = 12.0 + 3.0 * (t * 4.0).sin();
        node.width = Val::Px(pulse);
        node.height = Val::Px(pulse);
        node.top = Val::Px(997.0 - pulse * 0.5);
    }
    for (tick, mut color) in &mut ticks {
        let active = t >= tick.at && t < tick.next;
        color.0 = if active {
            Color::srgb(0.55, 0.92, 1.0)
        } else if t >= tick.next {
            Color::srgba(0.85, 0.9, 1.0, 0.55)
        } else {
            Color::srgba(0.62, 0.70, 0.84, 0.45)
        };
    }
}

fn fade_alpha(t: f32, start: f32, end: f32) -> f32 {
    if t < start || t >= end {
        return 0.0;
    }
    let inn = ease_out(((t - start) / 0.38).clamp(0.0, 1.0));
    let out = if end >= 40.0 {
        1.0
    } else {
        smooth(((end - t) / 0.30).clamp(0.0, 1.0))
    };
    inn * out
}

fn typed(full: &str, progress: f32, caret: bool) -> String {
    let count = full.chars().count();
    let n = ((count as f32) * progress).round() as usize;
    let mut text: String = full.chars().take(n.min(count)).collect();
    if caret {
        text.push('|');
    }
    text
}

fn format_timecode(t: f32) -> String {
    let mins = (t / 60.0) as u32;
    let secs = t - mins as f32 * 60.0;
    format!("{mins:02}:{secs:05.2}")
}

fn rail_x(seconds_mark: f32) -> f32 {
    120.0 + (seconds_mark / 30.0).clamp(0.0, 1.0) * 1680.0
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
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
