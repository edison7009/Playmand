//! Headless Avian physics recipes for Bevy. Each test checks one behaviour a
//! game relies on, with real Avian 0.8 on Bevy 0.20 and a fixed clock.

#[cfg(test)]
mod tests {
    use avian3d::prelude::*;
    use bevy::{prelude::*, time::TimeUpdateStrategy};
    use core::time::Duration;

    const STEP: f32 = 1.0 / 60.0;

    /// A headless app: no window, no renderer, time advanced by hand.
    fn physics_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                STEP,
            )));
        app.finish();
        app
    }

    fn run(app: &mut App, seconds: f32) {
        for _ in 0..(seconds / STEP).round() as usize {
            app.update();
        }
    }

    /// A 20 x 1 x 20 floor whose top face is at y = 0.
    fn spawn_floor(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((
                RigidBody::Static,
                Collider::cuboid(20.0, 1.0, 20.0),
                Transform::from_xyz(0.0, -0.5, 0.0),
            ))
            .id()
    }

    /// A ball of radius 0.5 dropped from `height`.
    fn spawn_ball(app: &mut App, height: f32) -> Entity {
        app.world_mut()
            .spawn((
                RigidBody::Dynamic,
                Collider::sphere(0.5),
                Transform::from_xyz(0.0, height, 0.0),
                CollisionEventsEnabled,
            ))
            .id()
    }

    fn height(app: &App, entity: Entity) -> f32 {
        app.world().get::<Position>(entity).unwrap().y
    }

    #[test]
    fn a_dropped_ball_comes_to_rest_on_the_floor_not_through_it() {
        let mut app = physics_app();
        spawn_floor(&mut app);
        let ball = spawn_ball(&mut app, 4.0);
        run(&mut app, 3.0);
        let y = height(&app, ball);
        assert!((y - 0.5).abs() < 0.05, "ball centre at {y}, expected 0.5");
        let speed = app.world().get::<LinearVelocity>(ball).unwrap().length();
        assert!(speed < 0.05, "ball still moving at {speed}");
    }

    /// Counts collision starts, read as messages, and when the first arrived.
    #[derive(Resource, Default)]
    struct Hits {
        count: usize,
        first_at: Option<f32>,
        pairs: Vec<(Entity, Entity)>,
    }

    fn record_hits(
        mut reader: MessageReader<CollisionStart>,
        mut hits: ResMut<Hits>,
        time: Res<Time>,
    ) {
        for event in reader.read() {
            hits.count += 1;
            hits.first_at.get_or_insert(time.elapsed_secs());
            hits.pairs.push((event.collider1, event.collider2));
        }
    }

    #[test]
    fn landing_reports_one_collision_start_when_free_fall_says_it_should() {
        let mut app = physics_app();
        app.init_resource::<Hits>().add_systems(Update, record_hits);
        let floor = spawn_floor(&mut app);
        let ball = spawn_ball(&mut app, 4.0);
        run(&mut app, 2.0);
        let hits = app.world().resource::<Hits>();
        assert_eq!(hits.count, 1, "a ball with no bounce touches down once");
        let (a, b) = hits.pairs[0];
        assert!((a, b) == (ball, floor) || (a, b) == (floor, ball));
        // The ball's bottom falls 3.5 m: t = sqrt(2 * 3.5 / 9.81) ≈ 0.845 s.
        let t = hits.first_at.unwrap();
        assert!((t - 0.845).abs() < 0.06, "first contact at {t:.3} s");
    }

    #[test]
    fn a_sensor_notices_the_ball_but_does_not_stop_it() {
        let mut app = physics_app();
        app.init_resource::<Hits>().add_systems(Update, record_hits);
        let plate = app
            .world_mut()
            .spawn((
                Sensor,
                Collider::cuboid(4.0, 0.2, 4.0),
                Transform::from_xyz(0.0, 2.0, 0.0),
                CollisionEventsEnabled,
            ))
            .id();
        let ball = spawn_ball(&mut app, 4.0);
        run(&mut app, 1.2);
        let hits = app.world().resource::<Hits>();
        assert!(
            hits.pairs.iter().any(|&(a, b)| a == plate || b == plate),
            "the sensor saw the ball pass"
        );
        assert!(height(&app, ball) < 0.0, "nothing held the ball up");
    }

    #[test]
    fn the_same_inputs_give_the_same_result_every_run() {
        let simulate = || {
            let mut app = physics_app();
            spawn_floor(&mut app);
            let balls: Vec<Entity> = (0..6)
                .map(|i| {
                    let x = i as f32 * 0.37 - 1.0;
                    app.world_mut()
                        .spawn((
                            RigidBody::Dynamic,
                            Collider::sphere(0.5),
                            Transform::from_xyz(x, 2.0 + i as f32 * 1.1, x * 0.5),
                        ))
                        .id()
                })
                .collect();
            run(&mut app, 2.5);
            balls
                .iter()
                .map(|&b| app.world().get::<Position>(b).unwrap().0)
                .collect::<Vec<Vec3>>()
        };
        assert_eq!(simulate(), simulate());
    }
}
