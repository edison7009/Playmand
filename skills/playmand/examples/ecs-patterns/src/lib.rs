//! Small, headless Bevy recipes. Each test checks an observable ECS invariant.

#[cfg(test)]
mod tests {
    use bevy::{prelude::*, state::app::StatesPlugin};

    #[derive(Component)]
    struct Player;
    #[derive(Component)]
    struct Enemy;
    #[derive(Component, Debug, PartialEq)]
    struct Health(i32);

    fn heal_player_damage_enemy(
        mut players: Query<&mut Health, With<Player>>,
        mut enemies: Query<&mut Health, (With<Enemy>, Without<Player>)>,
    ) {
        for mut health in &mut players {
            health.0 += 1;
        }
        for mut health in &mut enemies {
            health.0 -= 1;
        }
    }

    #[test]
    fn filters_make_mutable_queries_disjoint_even_with_both_markers() {
        let mut app = App::new();
        app.add_systems(Update, heal_player_damage_enemy);
        let player = app.world_mut().spawn((Player, Health(10))).id();
        let enemy = app.world_mut().spawn((Enemy, Health(10))).id();
        let both = app.world_mut().spawn((Player, Enemy, Health(10))).id();
        app.update();
        assert_eq!(app.world().get::<Health>(player), Some(&Health(11)));
        assert_eq!(app.world().get::<Health>(enemy), Some(&Health(9)));
        assert_eq!(app.world().get::<Health>(both), Some(&Health(11)));
    }

    #[derive(Component)]
    struct Projectile;
    #[derive(Resource, Default)]
    struct Observed(usize);

    fn spawn_projectile(mut commands: Commands) {
        commands.spawn(Projectile);
    }
    fn count_projectiles(query: Query<&Projectile>, mut observed: ResMut<Observed>) {
        observed.0 = query.iter().count();
    }

    #[test]
    fn chained_consumer_sees_commands_in_the_same_update() {
        let mut app = App::new();
        app.init_resource::<Observed>()
            .add_systems(Update, (spawn_projectile, count_projectiles).chain());
        app.update();
        assert_eq!(app.world().resource::<Observed>().0, 1);
        app.update();
        assert_eq!(app.world().resource::<Observed>().0, 2);
    }

    #[derive(Component, PartialEq)]
    struct Caption(String);
    #[derive(Resource)]
    struct DesiredCaption(String);

    fn sync_caption(desired: Res<DesiredCaption>, mut captions: Query<&mut Caption>) {
        for mut caption in &mut captions {
            caption.set_if_neq(Caption(desired.0.clone()));
        }
    }
    fn count_caption_changes(
        query: Query<&Caption, Changed<Caption>>,
        mut observed: ResMut<Observed>,
    ) {
        observed.0 += query.iter().count();
    }

    #[test]
    fn unchanged_value_does_not_retrigger_downstream_work() {
        let mut app = App::new();
        app.init_resource::<Observed>()
            .insert_resource(DesiredCaption("score 0".into()))
            .add_systems(Update, (sync_caption, count_caption_changes).chain());
        app.world_mut().spawn(Caption("score 0".into()));
        app.update(); // Added components count as changed.
        assert_eq!(app.world().resource::<Observed>().0, 1);
        app.update(); // Identical assignment must not trigger a second refresh.
        assert_eq!(app.world().resource::<Observed>().0, 1);
        app.world_mut().resource_mut::<DesiredCaption>().0 = "score 1".into();
        app.update();
        assert_eq!(app.world().resource::<Observed>().0, 2);
    }

    #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
    enum Screen {
        #[default]
        Menu,
        Playing,
    }
    #[derive(Component)]
    struct Level;
    fn spawn_level(mut commands: Commands) {
        commands.spawn((Level, DespawnOnExit(Screen::Playing)));
    }

    #[test]
    fn leaving_and_reentering_game_does_not_accumulate_level_entities() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin)
            .init_state::<Screen>()
            .add_systems(OnEnter(Screen::Playing), spawn_level);
        app.update();
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<NextState<Screen>>()
                .set(Screen::Playing);
            app.update();
            assert_eq!(
                app.world_mut().query::<&Level>().iter(app.world()).count(),
                1
            );
            app.world_mut()
                .resource_mut::<NextState<Screen>>()
                .set(Screen::Menu);
            app.update();
            assert_eq!(
                app.world_mut().query::<&Level>().iter(app.world()).count(),
                0
            );
        }
    }
}
