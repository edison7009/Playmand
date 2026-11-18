//! Bevy, Avian, Kira and egui in one game, checked together headless.
//!
//! Each crate has its own example; this one checks what only shows up when
//! they share an app: Bevy's `audio` feature left on next to bevy_kira_audio,
//! physics messages written in the fixed schedule and read in `Update`, a
//! sound played once per event at any frame rate, and clicks on an egui panel
//! that must not reach the game.
//!
//! The scene: a ball drops onto a pressure plate, the door opens and a click
//! plays. Left click fires unless egui wants the pointer.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui, input::egui_wants_any_pointer_input};
use bevy_kira_audio::prelude::*;

/// Mixer channel for short effects; music would get its own.
#[derive(Resource)]
pub struct Effects;

#[derive(Component)]
pub struct Plate;

/// What may press the plate. The plate also touches the floor it sits on, and
/// that contact is reported too, so check who stepped on it.
#[derive(Component)]
pub struct Ball;

#[derive(Component, Default)]
pub struct Door {
    pub open: bool,
}

/// The click sound, generated so the example ships no asset files.
#[derive(Resource)]
pub struct Click(pub Handle<AudioSource>);

/// What the game asked the mixer to play, for tests and debug overlays.
#[derive(Resource, Default)]
pub struct Played(pub Vec<Handle<AudioInstance>>);

#[derive(Resource, Default)]
pub struct Shots(pub u32);

/// The game itself: physics, audio and input, with no window or renderer.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_channel::<Effects>()
            .init_resource::<Played>()
            .init_resource::<Shots>()
            .add_systems(Startup, (load_click, spawn_scene))
            .add_systems(
                Update,
                (
                    open_door_on_plate,
                    // The game reads egui's verdict instead of guessing from
                    // pointer coordinates. bevy_egui updates it every frame.
                    fire.run_if(not(egui_wants_any_pointer_input)),
                ),
            );
    }
}

/// The debug panel, added only when the app also has `EguiPlugin` (a window
/// and a renderer). It runs in egui's own pass, not in `Update`.
pub struct DebugPanelPlugin;

impl Plugin for DebugPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, debug_panel);
    }
}

fn debug_panel(mut contexts: EguiContexts, shots: Res<Shots>, doors: Query<&Door>) -> Result {
    egui::Window::new("Debug").show(contexts.ctx_mut()?, |ui| {
        ui.label(format!("Shots: {}", shots.0));
        for door in &doors {
            ui.label(if door.open {
                "Door: open"
            } else {
                "Door: closed"
            });
        }
    });
    Ok(())
}

fn load_click(mut commands: Commands, mut sources: ResMut<Assets<AudioSource>>) {
    let rate = 48_000;
    let frames: Vec<Frame> = (0..rate / 20)
        .map(|i| {
            let t = i as f32 / rate as f32;
            Frame::from_mono((t * 2_000.0 * std::f32::consts::TAU).sin() * (1.0 - t * 20.0))
        })
        .collect();
    let sound = StaticSoundData {
        sample_rate: rate,
        frames: frames.into(),
        settings: StaticSoundSettings::new(),
        slice: None,
    };
    commands.insert_resource(Click(sources.add(AudioSource { sound })));
}

fn spawn_scene(mut commands: Commands) {
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    // A sensor: it reports the ball without stopping it, and it needs
    // `CollisionEventsEnabled` or no CollisionStart is ever written.
    commands.spawn((
        Plate,
        Sensor,
        Collider::cuboid(2.0, 0.2, 2.0),
        Transform::from_xyz(0.0, 0.1, 0.0),
        CollisionEventsEnabled,
    ));
    commands.spawn(Door::default());
    commands.spawn((
        Ball,
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));
}

/// Physics writes CollisionStart in the fixed schedule; this reads it in
/// `Update`. A frame may run zero or several fixed steps, and the reader still
/// sees every message exactly once.
fn open_door_on_plate(
    mut starts: MessageReader<CollisionStart>,
    plates: Query<(), With<Plate>>,
    balls: Query<(), With<Ball>>,
    mut doors: Query<&mut Door>,
    click: Res<Click>,
    effects: Res<AudioChannel<Effects>>,
    mut played: ResMut<Played>,
) {
    for start in starts.read() {
        let (a, b) = (start.collider1, start.collider2);
        let pressed =
            (plates.contains(a) && balls.contains(b)) || (plates.contains(b) && balls.contains(a));
        if !pressed {
            continue;
        }
        for mut door in &mut doors {
            if !door.open {
                door.open = true;
                played.0.push(effects.play(click.0.clone()).handle());
            }
        }
    }
}

fn fire(mouse: Res<ButtonInput<MouseButton>>, mut shots: ResMut<Shots>) {
    if mouse.just_pressed(MouseButton::Left) {
        shots.0 += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        input::{ButtonState, InputPlugin, mouse::MouseButtonInput},
        time::TimeUpdateStrategy,
    };
    use bevy_egui::{
        EguiContext,
        input::{EguiWantsInput, write_egui_wants_input_system},
    };
    use core::time::Duration;

    /// The whole stack without a window: Bevy, Avian, bevy_kira_audio and
    /// the game. With no sound card (CI), bevy_kira_audio logs a warning and
    /// keeps running; nothing here depends on hearing the sound.
    fn game(frame: f32) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            InputPlugin,
            AudioPlugin,
            PhysicsPlugins::default(),
            GamePlugin,
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            frame,
        )))
        // What bevy_egui's plugin sets up, in the same schedule; the context comes later.
        .init_resource::<EguiWantsInput>()
        .add_systems(PostUpdate, write_egui_wants_input_system);
        app.finish();
        app
    }

    fn run(app: &mut App, frame: f32, seconds: f32) {
        for _ in 0..(seconds / frame).round() as usize {
            app.update();
        }
    }

    fn door_open(app: &mut App) -> bool {
        app.world_mut()
            .query::<&Door>()
            .single(app.world())
            .unwrap()
            .open
    }

    #[test]
    fn the_plate_opens_the_door_and_clicks_once_at_any_frame_rate() {
        // 30 fps runs two fixed steps in some frames, 240 fps runs none in
        // most: the door must open once and the click must play once either way.
        for fps in [30.0, 60.0, 144.0, 240.0] {
            let frame = 1.0 / fps;
            let mut app = game(frame);
            run(&mut app, frame, 0.3);
            assert!(
                !door_open(&mut app),
                "{fps} fps: opened before the ball landed"
            );
            run(&mut app, frame, 2.5);
            assert!(door_open(&mut app), "{fps} fps: door still closed");
            let played = app.world().resource::<Played>().0.len();
            assert_eq!(played, 1, "{fps} fps: click played {played} times");
        }
    }

    /// Sends the button messages a window would. Pressing `ButtonInput`
    /// directly does not work: Bevy's input system clears `just_pressed` in
    /// `PreUpdate`, before the game reads it.
    fn click(app: &mut App) {
        for state in [ButtonState::Pressed, ButtonState::Released] {
            app.world_mut().write_message(MouseButtonInput {
                button: MouseButton::Left,
                state,
                window: Entity::PLACEHOLDER,
            });
            app.update();
        }
    }

    #[test]
    fn a_click_on_the_egui_panel_does_not_fire() {
        let frame = 1.0 / 60.0;
        let mut app = game(frame);
        app.update();
        click(&mut app);
        assert_eq!(
            app.world().resource::<Shots>().0,
            1,
            "a click on the game fires"
        );

        // A real egui context with a window under the pointer, as the debug
        // panel would leave it. bevy_egui's own system turns that into
        // EguiWantsInput, and the game's run condition reads it.
        let ctx = EguiContext::default();
        let mut panel = ctx.clone();
        let at = egui::pos2(50.0, 40.0);
        let input = |events| egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            events,
            ..Default::default()
        };
        for events in [vec![], vec![egui::Event::PointerMoved(at)]] {
            panel
                .get_mut()
                .run_ui(input(events), |ui| {
                    egui::Window::new("Debug")
                        .fixed_pos(egui::pos2(10.0, 10.0))
                        .show(ui.ctx(), |ui| ui.label("Shots"));
                })
                .drop_without_applying_deltas();
        }
        app.world_mut().spawn(panel);
        // bevy_egui updates EguiWantsInput in PostUpdate, so the game reads the
        // previous frame's value; a real pointer rests on the panel before it clicks.
        app.update();
        click(&mut app);
        assert!(
            app.world()
                .resource::<EguiWantsInput>()
                .wants_any_pointer_input()
        );
        assert_eq!(
            app.world().resource::<Shots>().0,
            1,
            "the click on the panel fired"
        );
    }
}
