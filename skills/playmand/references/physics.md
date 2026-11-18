# Physics with Avian

Baseline: Avian 0.8.0 on Bevy 0.20.1 (Rust 1.97.1). Verified by the [physics example](../examples/physics-avian/src/lib.rs), which runs headless in CI on Windows and Linux. Avian has `avian2d` and `avian3d` crates with the same API; this page uses 3D.

Avian is an ECS-native physics engine for Bevy: bodies, colliders and velocities are ordinary components, and physics runs as Bevy systems. Use it for new Bevy games. If a project already uses Rapier (`bevy_rapier3d`, 0.37 supports Bevy 0.20), keep Rapier rather than switching.

## Setup

```toml
[dependencies]
bevy = "=0.20.1"
avian3d = "=0.8.0"
```

```rust
use avian3d::prelude::*;
use bevy::prelude::*;

App::new().add_plugins((DefaultPlugins, PhysicsPlugins::default()));
```

Pin the exact version: Avian's minor releases change APIs and each targets one Bevy version. Check a dependency's Bevy requirement on crates.io (or with `scripts/inspect_bevy.py`) before upgrading either.

## Bodies and colliders

```rust
commands.spawn((RigidBody::Static, Collider::cuboid(20.0, 1.0, 20.0), Transform::from_xyz(0.0, -0.5, 0.0)));
commands.spawn((RigidBody::Dynamic, Collider::sphere(0.5), Transform::from_xyz(0.0, 4.0, 0.0)));
```

- `RigidBody::Dynamic` moves under gravity and contacts, `Static` never moves, `Kinematic` moves only by the velocity you set.
- `Collider::cuboid` takes **full lengths**, not half extents: the floor above is 1 m thick with its top at y = 0.
- Spawn with `Transform`; Avian keeps its own `Position` / `Rotation` in sync. Read `Position` and `LinearVelocity` for physics state.
- Tune with `Restitution`, `Friction`, `Mass`, `GravityScale`, `LockedAxes`; push with `LinearVelocity` or `ConstantForce`. Global gravity is the `Gravity` resource (default −9.81 on y).

## Collision events

Events are only produced for entities with `CollisionEventsEnabled`. Read many at once as messages, or react per entity with an observer:

```rust
fn hits(mut reader: MessageReader<CollisionStart>) {
    for e in reader.read() { /* e.collider1, e.collider2 */ }
}

commands.spawn((Collider::cuboid(1.0, 0.1, 1.0), Sensor, CollisionEventsEnabled))
    .observe(|event: On<CollisionStart>| { /* someone stepped on the plate */ });
```

`CollisionEnd` is the matching event when contact stops. A `Sensor` reports overlaps but never pushes anything: use it for triggers, pickups and goal zones.

## Time and determinism

Physics runs in `FixedPostUpdate` on a fixed timestep, so results do not depend on the frame rate. Systems that read physics results each frame belong in `Update` or later; systems that apply forces belong in `FixedUpdate`.

For tests and recordings, drive time by hand: `MinimalPlugins + TransformPlugin + PhysicsPlugins`, then insert `TimeUpdateStrategy::ManualDuration(1/60 s)` and call `app.update()` per step. The same inputs then give bit-identical positions on the same machine (the example asserts this). Cross-machine determinism needs Avian's `enhanced-determinism` feature.

## Checking it

Assert outcomes, not frames: a dropped ball rests at its radius above the floor and stops moving; the first `CollisionStart` arrives when free fall predicts (t = √(2h/g)); a sensor sees a body pass without stopping it. Watch a debug view for feel: enable Avian's `debug-plugin` feature and add `PhysicsDebugPlugin` to draw colliders.

## Pitfalls

- A collider with no `RigidBody` is static.
- Tunnelling: fast, thin objects can pass through each other in one step. Make colliders thicker, lower speeds, or enable swept continuous collision (`SweptCcd`) on the fast body.
- Scaling a `Transform` scales its collider; keep visual meshes and colliders in the same units (1 unit = 1 m works best with the default gravity).
- Events silently never arrive without `CollisionEventsEnabled` on at least one of the two entities.
