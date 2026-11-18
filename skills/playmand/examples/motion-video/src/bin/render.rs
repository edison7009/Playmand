//! Renders the promo timeline offscreen, one PNG per frame, plus the
//! soundtrack (WAV) and captions (SRT) from the same timeline.
//!
//! cargo run --release --features render -- <landscape|portrait|square> <out-dir>
//!
//! A game-style video: the same tools a game uses (springy motion, particles,
//! wipes, ripples, a drifting background) driven by a fixed clock instead of
//! input. No window: the camera draws into an image, each frame is read back
//! once, and the clock only advances after that frame is on disk.

use std::{f32::consts::PI, fs, path::PathBuf, time::Duration};

use bevy::{
    app::ScheduleRunnerPlugin,
    camera::RenderTarget,
    prelude::*,
    render::{
        gpu_readback::{ReadbackComplete, ReadbackOnce},
        render_resource::{TextureFormat, TextureUsages},
    },
    sprite::Anchor,
    sprite_render::AlphaMode2d,
    text::{FontSize, Justify, TextBounds, TextLayout, TextLayoutInfo},
    window::ExitCondition,
};
use playmand_motion_video::*;

/// Frames rendered before the first capture, so fonts, pipelines and the
/// glyph atlas are ready and letter widths are measured.
const WARMUP: u32 = 30;
const FONT: &str = "Helvetica Neue, Helvetica, Arial, sans-serif";
const BG_CIRCLES: usize = 9;
const PARTICLES: usize = 28;
const RIPPLES: usize = 3;
const GRID: usize = 7;

#[derive(Resource)]
struct Rec {
    timeline: Timeline,
    format: Format,
    target: Handle<Image>,
    dir: PathBuf,
    frame: u32,
    warm: u32,
    in_flight: bool,
}

/// Measured advance of each letter, so words built from one entity per
/// letter are laid out like the word itself.
#[derive(Resource, Default)]
struct Widths {
    title: Vec<f32>,
    end: Vec<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Word {
    Title,
    End,
}

/// What an entity is, so one system can pose everything from `t`.
#[derive(Component, Clone, Copy)]
enum Part {
    Background(usize),
    Letter(Word, usize),
    TitleRule,
    Particle(usize),
    TitleSub,
    Wipe(bool),
    ChartHeading,
    ChartNote,
    Grid(usize),
    GridLabel(usize),
    BarLabel(usize),
    Bar(usize),
    BarValue(usize),
    Sheen,
    LogoDisc,
    LogoPlay,
    Ripple(usize),
    EndUrl,
    EndRule,
    Caption(usize),
}

impl Part {
    fn shot(self) -> Option<ShotKind> {
        Some(match self {
            Part::Letter(Word::Title, _) | Part::TitleRule | Part::Particle(_) | Part::TitleSub => {
                ShotKind::Title
            }
            Part::ChartHeading
            | Part::ChartNote
            | Part::Grid(_)
            | Part::GridLabel(_)
            | Part::BarLabel(_)
            | Part::Bar(_)
            | Part::BarValue(_)
            | Part::Sheen => ShotKind::Chart,
            Part::Letter(Word::End, _)
            | Part::LogoDisc
            | Part::LogoPlay
            | Part::Ripple(_)
            | Part::EndUrl
            | Part::EndRule => ShotKind::EndCard,
            Part::Background(_) | Part::Wipe(_) | Part::Caption(_) => return None,
        })
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let format = FORMATS
        .into_iter()
        .find(|f| Some(f.name) == args.get(1).map(String::as_str))
        .unwrap_or(LANDSCAPE);
    let dir = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "out".into())).join(format.name);
    fs::create_dir_all(&dir).expect("create output folder");

    let timeline = Timeline::promo(30);
    // Sound and captions come from the same timeline as the picture.
    fs::write(
        dir.join("soundtrack.wav"),
        wav_bytes(&render_audio(&timeline, 48_000), 48_000),
    )
    .expect("write soundtrack");
    fs::write(dir.join("captions.srt"), timeline.srt()).expect("write captions");

    App::new()
        .add_plugins(
            DefaultPlugins
                // No window at all: built without winit, so nothing to disable.
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                }),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO))
        .insert_resource(ClearColor(rgb(PLAYMAND.paper)))
        .insert_resource(Rec {
            timeline,
            format,
            target: Handle::default(),
            dir,
            frame: 0,
            warm: 0,
            in_flight: false,
        })
        .init_resource::<Widths>()
        .add_systems(Startup, setup)
        .add_systems(Update, (measure, animate, capture).chain())
        .run();
}

fn rgb(c: [u8; 3]) -> Color {
    Color::srgb_u8(c[0], c[1], c[2])
}

fn text(s: &str, size: f32, color: [u8; 3]) -> (Text2d, TextFont, TextColor) {
    (
        Text2d::new(s),
        TextFont::from(FONT).with_font_size(FontSize::Px(size)),
        TextColor(rgb(color)),
    )
}

fn title_size(f: &Format) -> f32 {
    // Capped by the safe width so a long name never runs off a narrow frame.
    (170.0 * f.unit()).min(f.safe().width() / 4.8)
}

fn setup(
    mut commands: Commands,
    mut rec: ResMut<Rec>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let f = rec.format;
    let mut image =
        Image::new_target_texture(f.width, f.height, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    rec.target = images.add(image);
    commands.spawn((Camera2d, RenderTarget::Image(rec.target.clone().into())));

    let (u, s, b) = (f.unit(), f.safe(), PLAYMAND);
    let wrap = TextBounds::new_horizontal(s.width());
    let center = TextLayout::justify(Justify::Center);
    let disc = meshes.add(Circle::new(1.0));
    // Every mesh gets its own material so its opacity can change on its own.
    // Set the blend mode explicitly: `ColorMaterial::from_color` picks Opaque
    // for an opaque colour, and an opaque material ignores alpha later.
    let mut mat = |c: [u8; 3]| {
        MeshMaterial2d(materials.add(ColorMaterial {
            color: rgb(c),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        }))
    };

    // Background: large soft circles drifting slowly, for depth.
    for i in 0..BG_CIRCLES {
        commands.spawn((
            Part::Background(i),
            Mesh2d(disc.clone()),
            mat(b.line),
            Transform::default(),
        ));
    }

    // Title, one entity per letter so each can spring in on its own.
    for (i, c) in b.name.chars().enumerate() {
        commands.spawn((
            Part::Letter(Word::Title, i),
            text(&c.to_string(), title_size(&f), b.ink),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
    }
    commands.spawn((
        Part::TitleRule,
        Sprite::from_color(rgb(b.accent), Vec2::ONE),
        Transform::from_xyz(0.0, -50.0 * u, 1.0),
    ));
    for i in 0..PARTICLES {
        let color = if i % 3 == 0 { b.ink } else { b.accent };
        commands.spawn((
            Part::Particle(i),
            Mesh2d(disc.clone()),
            mat(color),
            Transform::default(),
        ));
    }
    commands.spawn((
        Part::TitleSub,
        text(
            "Bevy · Avian · Kira · egui · wgpu · WebAssembly",
            42.0 * u,
            b.soft,
        ),
        wrap,
        center,
        Transform::from_xyz(0.0, -130.0 * u, 1.0),
    ));

    // Chart.
    let top = s.top - 70.0 * u;
    commands.spawn((
        Part::ChartHeading,
        text("Execution time vs C", 72.0 * u, b.ink),
        wrap,
        center,
        Transform::from_xyz(0.0, top, 1.0),
    ));
    commands.spawn((
        Part::ChartNote,
        text(
            "lower is faster · Pereira et al., SLE 2017",
            32.0 * u,
            b.soft,
        ),
        wrap,
        center,
        Transform::from_xyz(0.0, top - 80.0 * u, 1.0),
    ));
    for k in 0..GRID {
        commands.spawn((
            Part::Grid(k),
            Sprite::from_color(rgb(b.line), Vec2::ONE),
            Transform::default(),
        ));
        commands.spawn((
            Part::GridLabel(k),
            text(&format!("{}×", k + 1), 26.0 * u, b.soft),
            Anchor::TOP_CENTER,
            Transform::default(),
        ));
    }
    for (i, bar) in BARS.iter().enumerate() {
        commands.spawn((
            Part::BarLabel(i),
            text(bar.0, 40.0 * u, b.ink),
            if f.portrait() {
                Anchor::BOTTOM_LEFT
            } else {
                Anchor::CENTER_RIGHT
            },
            Transform::default(),
        ));
        let color = if i == 0 { b.accent } else { b.line };
        commands.spawn((
            Part::Bar(i),
            Sprite::from_color(rgb(color), Vec2::ONE),
            Anchor::CENTER_LEFT,
            Transform::default(),
        ));
        commands.spawn((
            Part::BarValue(i),
            text("", 40.0 * u, b.ink),
            Anchor::CENTER_LEFT,
            Transform::default(),
        ));
    }
    commands.spawn((
        Part::Sheen,
        Sprite::from_color(Color::WHITE, Vec2::ONE),
        Transform::default(),
    ));

    // End card: ripples behind a play-button mark, the name, the address.
    for i in 0..RIPPLES {
        commands.spawn((
            Part::Ripple(i),
            Mesh2d(meshes.add(Annulus::new(0.93, 1.0))),
            mat(b.accent),
            Transform::default(),
        ));
    }
    commands.spawn((
        Part::LogoDisc,
        Mesh2d(disc.clone()),
        mat(b.ink),
        Transform::default(),
    ));
    let r = 34.0 * u;
    commands.spawn((
        Part::LogoPlay,
        Mesh2d(meshes.add(Triangle2d::new(
            Vec2::new(-0.6 * r, r),
            Vec2::new(-0.6 * r, -r),
            Vec2::new(1.1 * r, 0.0),
        ))),
        mat(b.paper),
        Transform::default(),
    ));
    for (i, c) in b.name.chars().enumerate() {
        commands.spawn((
            Part::Letter(Word::End, i),
            text(&c.to_string(), 120.0 * u, b.ink),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
    }
    commands.spawn((
        Part::EndUrl,
        text(b.url, 48.0 * u, b.accent),
        Transform::from_xyz(0.0, -120.0 * u, 1.0),
    ));
    commands.spawn((
        Part::EndRule,
        Sprite::from_color(rgb(b.accent), Vec2::ONE),
        Transform::from_xyz(0.0, -152.0 * u, 1.0),
    ));

    // The wipe on each cut: an accent band, and a thin ink band just behind it.
    commands.spawn((
        Part::Wipe(false),
        Sprite::from_color(rgb(b.accent), Vec2::ONE),
        Transform::default(),
    ));
    commands.spawn((
        Part::Wipe(true),
        Sprite::from_color(rgb(b.ink), Vec2::ONE),
        Transform::default(),
    ));

    // Burned-in captions on top of everything, inside the safe area.
    for i in 0..rec.timeline.shots.len() {
        commands.spawn((
            Part::Caption(i),
            text(rec.timeline.shots[i].caption, 40.0 * u, b.ink),
            wrap,
            center,
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(s.center_x(), s.bottom, 10.0),
        ));
    }
}

/// Reads each letter's laid-out width (ready a frame after it is spawned).
fn measure(mut widths: ResMut<Widths>, letters: Query<(&Part, &TextLayoutInfo)>, rec: Res<Rec>) {
    if rec.warm >= WARMUP {
        return;
    }
    let n = PLAYMAND.name.chars().count();
    widths.title = vec![0.0; n];
    widths.end = vec![0.0; n];
    for (part, info) in &letters {
        if let Part::Letter(word, i) = *part {
            let list = if word == Word::Title {
                &mut widths.title
            } else {
                &mut widths.end
            };
            list[i] = info.size.x;
        }
    }
}

/// Centre x of letter `i` in a word laid out from its letters' widths.
fn letter_x(widths: &[f32], i: usize) -> f32 {
    let total: f32 = widths.iter().sum();
    -total / 2.0 + widths[..i].iter().sum::<f32>() + widths.get(i).copied().unwrap_or(0.0) / 2.0
}

#[allow(clippy::type_complexity)]
fn animate(
    rec: Res<Rec>,
    widths: Res<Widths>,
    mut parts: Query<(
        &Part,
        &mut Transform,
        Option<&mut TextColor>,
        Option<&mut Text2d>,
        Option<&mut Sprite>,
        Option<&MeshMaterial2d<ColorMaterial>>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tl = &rec.timeline;
    let t = tl.time(rec.frame.min(tl.frames() - 1));
    let f = rec.format;
    let (u, s) = (f.unit(), f.safe());
    let (w, h) = (f.width as f32, f.height as f32);
    let shot_index = |kind: ShotKind| tl.shots.iter().position(|s| s.kind == kind).unwrap();
    let local = |kind: ShotKind| t - tl.shots[shot_index(kind)].start;

    // Chart layout. Landscape and square: labels left of the bars. 9:16 is
    // narrow, so labels sit above the bars and the bars get the full width.
    let stacked = f.portrait();
    let label_w = if stacked { 0.0 } else { 230.0 * u };
    let x0 = s.left + label_w;
    let full = s.right - 170.0 * u - x0;
    let max = 7.0; // the grid runs to 7x
    let gap = if stacked { 170.0 } else { 120.0 } * u;
    let row = |i: usize| s.center_y() + 1.5 * gap - 60.0 * u - i as f32 * gap;
    let grid_top = row(0) + gap * 0.5;
    let grid_bottom = row(BARS.len() - 1) - gap * 0.5;
    let bar_growth = |lt: f32, i: usize| ease_out_back(progress(lt, bar_start(i), BAR_GROW));

    let logo_y = 150.0 * u;
    let rule_y = -50.0 * u;
    let title_y = 60.0 * u;

    for (part, mut tf, color, label, mut sprite, material) in &mut parts {
        let mut alpha = match *part {
            Part::Caption(i) => {
                if tl.caption_at(t) == Some(i) {
                    1.0
                } else {
                    0.0
                }
            }
            p => p.shot().map_or(1.0, |k| tl.opacity(shot_index(k), t)),
        };
        match *part {
            Part::Background(i) => {
                let mut r = Rng::new(100 + i as u32);
                let (px, py, rad) = (r.next() - 0.5, r.next() - 0.5, 0.12 + 0.22 * r.next());
                let phase = r.next() * 2.0 * PI;
                let x = px * w + (0.35 * t + phase).sin() * 40.0 * u;
                let y = py * h + (0.27 * t + phase).cos() * 30.0 * u;
                tf.translation = Vec3::new(x, y, -10.0 + i as f32 * 0.01);
                tf.scale = Vec3::splat(rad * w.min(h));
                alpha = 0.10 + 0.12 * r.next();
            }
            Part::Letter(word, i) => {
                let (widths, start, y, rise) = match word {
                    Word::Title => (
                        &widths.title,
                        // Started before frame 0, so the first frame (often the
                        // thumbnail) already shows the name arriving.
                        -0.3 + i as f32 * TITLE_STAGGER,
                        title_y,
                        90.0 * u,
                    ),
                    Word::End => (&widths.end, 0.35 + i as f32 * 0.04, -20.0 * u, 50.0 * u),
                };
                let lt = local(if word == Word::Title {
                    ShotKind::Title
                } else {
                    ShotKind::EndCard
                });
                let p = progress(lt, start, TITLE_RISE);
                let spring = ease_out_back(p);
                // Each letter springs up from below, tilting and settling.
                let tilt = if i % 2 == 0 { 1.0 } else { -1.0 } * 0.35 * (1.0 - ease_out_cubic(p));
                tf.translation = Vec3::new(letter_x(widths, i), y - rise * (1.0 - spring), 1.0);
                tf.rotation = Quat::from_rotation_z(tilt);
                tf.scale = Vec3::splat(0.6 + 0.4 * spring);
                alpha *= ease_out_cubic((p * 2.5).min(1.0));
            }
            Part::TitleRule => {
                let p = ease_out_back(progress(local(ShotKind::Title), TITLE_BURST, 0.4));
                if let Some(sp) = sprite.as_mut() {
                    sp.custom_size = Some(Vec2::new((440.0 * u * p).max(0.01), 10.0 * u));
                }
            }
            Part::Particle(i) => {
                let age = local(ShotKind::Title) - TITLE_BURST;
                match particle(i as u32, age, 0.9) {
                    Some(([dx, dy], fade, size)) => {
                        let reach = 300.0 * u;
                        // Distinct depths: overlapping shapes at equal z draw in an unspecified order.
                        tf.translation =
                            Vec3::new(dx * reach, rule_y + dy * reach, 0.5 + i as f32 * 0.001);
                        tf.scale = Vec3::splat(9.0 * u * size);
                        alpha *= fade;
                    }
                    None => alpha = 0.0,
                }
            }
            Part::TitleSub => {
                let p = ease_out_cubic(progress(local(ShotKind::Title), 1.15, 0.5));
                tf.translation.y = -130.0 * u - 30.0 * u * (1.0 - p);
                alpha *= p;
            }
            Part::Wipe(trailing) => match tl.wipe(t - if trailing { 0.07 } else { 0.0 }) {
                Some((l, r)) => {
                    let band = if trailing { 0.04 } else { r - l };
                    let left = if trailing { l - band } else { l };
                    tf.translation = Vec3::new((left + band / 2.0) * w, 0.0, 5.0);
                    // A slight slant reads as speed.
                    tf.rotation = Quat::from_rotation_z(-0.12);
                    if let Some(sp) = sprite.as_mut() {
                        sp.custom_size = Some(Vec2::new(band * w, h * 1.3));
                    }
                }
                None => alpha = 0.0,
            },
            Part::ChartHeading | Part::ChartNote => {}
            Part::Grid(k) => {
                let x = x0 + full * (k + 1) as f32 / max;
                tf.translation = Vec3::new(x, (grid_top + grid_bottom) / 2.0, 0.2);
                if let Some(sp) = sprite.as_mut() {
                    sp.custom_size = Some(Vec2::new(2.0 * u, grid_top - grid_bottom));
                }
                alpha *= progress(local(ShotKind::Chart), 0.05 + 0.04 * k as f32, 0.3);
            }
            Part::GridLabel(k) => {
                let x = x0 + full * (k + 1) as f32 / max;
                tf.translation = Vec3::new(x, grid_bottom - 8.0 * u, 0.2);
                alpha *= progress(local(ShotKind::Chart), 0.05 + 0.04 * k as f32, 0.3);
            }
            Part::BarLabel(i) => {
                tf.translation = if stacked {
                    Vec3::new(x0, row(i) + 36.0 * u, 1.0)
                } else {
                    Vec3::new(x0 - 24.0 * u, row(i), 1.0)
                };
                alpha *= ease_out_cubic(progress(local(ShotKind::Chart), bar_start(i) - 0.15, 0.3));
            }
            Part::Bar(i) => {
                // Bars overshoot a little and settle, like a spring.
                let g = bar_growth(local(ShotKind::Chart), i);
                tf.translation = Vec3::new(x0, row(i), 1.0);
                if let Some(sp) = sprite.as_mut() {
                    sp.custom_size =
                        Some(Vec2::new((full * BARS[i].1 / max * g).max(0.01), 64.0 * u));
                }
            }
            Part::BarValue(i) => {
                let lt = local(ShotKind::Chart);
                let g = bar_growth(lt, i);
                tf.translation = Vec3::new(x0 + full * BARS[i].1 / max * g + 16.0 * u, row(i), 1.0);
                if let Some(l) = label {
                    // The number counts up with the bar and stops on the value.
                    l.into_inner().0 = format!("{:.2}×", BARS[i].1 * g.min(1.0));
                }
                // Once every bar is in, the Rust value pulses.
                let pulse = if i == 0 {
                    (PI * progress(lt, 2.0, 0.45)).sin()
                } else {
                    0.0
                };
                tf.scale = Vec3::splat(1.0 + 0.3 * pulse);
                alpha *= progress(lt, bar_start(i), 0.15);
            }
            Part::Sheen => {
                // A light glint sweeps across the Rust bar after it lands.
                let lt = local(ShotKind::Chart);
                let p = progress(lt, 1.6, 0.45);
                let len = full * BARS[0].1 / max;
                tf.translation = Vec3::new(x0 + len * ease_in_out_cubic(p), row(0), 1.5);
                tf.rotation = Quat::from_rotation_z(-0.35);
                if let Some(sp) = sprite.as_mut() {
                    sp.custom_size = Some(Vec2::new(10.0 * u, 70.0 * u));
                }
                alpha *= if p > 0.0 && p < 1.0 {
                    0.7 * (PI * p).sin()
                } else {
                    0.0
                };
            }
            Part::LogoDisc => {
                tf.translation = Vec3::new(0.0, logo_y, 1.0);
                tf.scale = Vec3::splat(
                    80.0 * u
                        * ease_out_back(progress(local(ShotKind::EndCard), 0.1, 0.55)).max(0.0),
                );
            }
            Part::LogoPlay => {
                let p = progress(local(ShotKind::EndCard), 0.25, 0.55);
                tf.translation = Vec3::new(0.0, logo_y, 2.0);
                tf.rotation = Quat::from_rotation_z(-PI / 2.0 * (1.0 - ease_out_back(p)));
                tf.scale = Vec3::splat(ease_out_back(p).max(0.0));
            }
            Part::Ripple(i) => {
                // Rings spread from the mark as the chime sounds.
                let p = progress(local(ShotKind::EndCard), 0.5 + 0.22 * i as f32, 1.1);
                tf.translation = Vec3::new(0.0, logo_y, 0.8);
                tf.scale = Vec3::splat(80.0 * u * (1.0 + 1.8 * ease_out_cubic(p)));
                alpha *= if p > 0.0 && p < 1.0 {
                    0.55 * (1.0 - p)
                } else {
                    0.0
                };
            }
            Part::EndUrl => alpha *= ease_out_cubic(progress(local(ShotKind::EndCard), 0.75, 0.5)),
            Part::EndRule => {
                let p = ease_out_cubic(progress(local(ShotKind::EndCard), 0.95, 0.5));
                if let Some(sp) = sprite.as_mut() {
                    sp.custom_size = Some(Vec2::new((300.0 * u * p).max(0.01), 4.0 * u));
                }
            }
            Part::Caption(_) => {}
        }
        if let Some(mut c) = color {
            c.0 = c.0.with_alpha(alpha);
        }
        if let Some(sp) = sprite.as_mut() {
            sp.color = sp.color.with_alpha(alpha);
        }
        if let Some(m) = material
            && let Some(mut mat) = materials.get_mut(&m.0)
        {
            mat.color = mat.color.with_alpha(alpha);
        }
    }
}

fn capture(mut commands: Commands, mut rec: ResMut<Rec>, mut exit: MessageWriter<AppExit>) {
    if rec.in_flight {
        return; // the previous frame is still being read back
    }
    if rec.warm < WARMUP {
        rec.warm += 1;
        return;
    }
    if rec.frame >= rec.timeline.frames() {
        println!("{} frames in {}", rec.frame, rec.dir.display());
        exit.write(AppExit::Success);
        return;
    }
    rec.in_flight = true;
    let n = rec.frame;
    commands
        .spawn(ReadbackOnce::texture(rec.target.clone()))
        .observe(move |ev: On<ReadbackComplete>, mut rec: ResMut<Rec>| {
            let f = rec.format;
            let pixels = unpad_rows(&ev.data, f.width, f.height, 4);
            let path = rec.dir.join(format!("frame-{n:04}.png"));
            let file = fs::File::create(&path).expect("create frame");
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), f.width, f.height);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_compression(png::Compression::Fast);
            enc.write_header()
                .and_then(|mut w| w.write_image_data(&pixels))
                .expect("write frame");
            rec.frame = n + 1;
            rec.in_flight = false;
        });
}
