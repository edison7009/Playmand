# ECS, scheduling and gameplay state

Symbols below use Bevy 0.20.1 as the baseline. Runnable code: [ecs-patterns](../examples/ecs-patterns/src/lib.rs).

## Data ownership and Rust borrows

- Per-entity data or abilities, such as velocity, health or faction, are a `Component`; one-of-a-kind world configuration or shared progress is a `Resource`. Do not copy every entity's data into one global mutable object to get around the ECS.
- `Query<&mut Transform, With<Player>>` and `Query<&mut Transform, With<Enemy>>` are not guaranteed disjoint: one entity can carry both markers. When the logic should be exclusive, say so with `Without`; when overlap is real and access is sequential, use `ParamSet` and finish the `p0()` borrow before taking `p1()`. Do not silence B0001 by adding a `Mutex` or cloning the whole world.
- `Query::single()` returns a `Result` in 0.20.1. Whether zero or several players are normal during loading or a scene switch is a game decision; for a normal transition use `let Ok(...) = ... else { return; }`, and do not swallow every error unconditionally.
- Bevy's change detection tracks mutable dereferences; it does not compare field values. Writing the same `Transform` or UI data again can still trigger downstream work. In hot paths use `set_if_neq` for comparable values, or compare before writing; a newly added component still counts as changed.

## The schedule is not the order functions are declared

```rust
app.add_systems(Update, (spawn_projectile, resolve_projectile).chain());
```

`Commands` are deferred writes. With the default schedule build settings, 0.20.1's `.chain()` / `.after()` / `.before()` insert `ApplyDeferred` on the dependency edges that need it, so downstream systems see earlier commands. Writing just `(a, b)` gives no order. `chain_ignore_deferred()` and similar explicitly skip that sync; if automatic insertion is turned off, handle it yourself. 0.20 adds `chain_weak()` / `after_weak()` / `before_weak()`, and Bevy's own rendering and `PostUpdate` UI schedules now use weak ordering. Your own `.chain()` keeps its meaning, but if you rely on "the built-in system always finishes first" and that dependency is not visible in ECS access, add an explicit `.after(...)`.

0.20 observer syntax: `On<Add, A>` → `On<Add<A>>`, and `On<E, B>` no longer takes a bundle generic. For state transitions, `NextState::set_if_neq` → `set_if_different` (the old name is deprecated); `set_if_neq` on components and resources is unaffected. `iter_many*` now returns a `Result` per item.

Order only systems with a real data dependency; chaining the whole game loses parallelism. `.after(foo)` does not register `foo`, and cannot create an ordering edge across two different schedules. First find out which of `PreUpdate` / `FixedUpdate` / `Update` / `PostUpdate` the system actually runs in.

Reading `GlobalTransform` right after changing `Transform` can return the value from before propagation. Systems that need final world positions run after `TransformSystems::Propagate` in `PostUpdate`; reads of final UI sizes run after `UiSystems::Layout`. Do not hide an ordering bug by waiting a few frames.

## Input and timesteps

Scale continuous motion by the timestep: for example `normalize_or_zero()` the direction first, then multiply by speed and seconds, so diagonals are not faster. When collision or physics simulation uses a fixed timestep, use fixed simulation time in `FixedUpdate`; cameras, UI and visual interpolation can stay on the frame update.

A frame can run the fixed update zero or several times. Reading the per-frame `just_pressed` directly in `FixedUpdate` can drop a press or apply it twice. Follow the input-gathering order of that version's official fixed-timestep example: record one-shot actions as pending state, consume and clear them in the simulation step, and keep continuous directions until the next sample. Do not clear a direction that is still held when the first step consumes it.

Express pause with gameplay states and system run conditions, and decide which clocks keep running. `Time<Virtual>` can pause and scale, `Time<Real>` ignores game pause, and `Time<Fixed>` drives fixed simulation. Menu animations that should keep playing must not be wired to the paused simulation clock.

## States, messages and restarting

`States` / `NextState` manage truly exclusive states such as menu, playing and results; `OnEnter` creates that state's content, and `OnExit` cleans it up or `DespawnOnExit` does. `NextState::set` requests a transition; the rest of the same system is not yet in the new state. `DefaultPlugins` and a trimmed test `App` register different plugins; a trimmed test must add `StatesPlugin` explicitly.

Restarting handles both state-owned entities and persistent `Resource`s: despawning the level does not reset score, cooldowns or input queues. What carries across levels is a game design decision. The runnable example checks state-exit cleanup; test resource resets such as score in the real game.

In 0.20.1, buffered messages use `Message` / `MessageWriter` / `MessageReader`, and observer events use `Event` / `On`. The first suits batch processing inside the schedule, the second triggering observers; do not copy the old `EventReader`. Temporary buffered messages are not a queue that survives an open pause menu indefinitely; when you need a persistent backlog, store it in a resource.

Sources: [Query](https://docs.rs/bevy/0.20.1/bevy/ecs/system/struct.Query.html), [schedule ordering and deferred commands](https://docs.rs/bevy/0.20.1/bevy/ecs/schedule/trait.IntoScheduleConfigs.html), and the same version's `examples/ecs`, `examples/time` and `examples/movement/physics_in_fixed_timestep.rs`. Check that version's source when the project uses a different one.
