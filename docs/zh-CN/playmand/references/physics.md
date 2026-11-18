# 用 Avian 做物理

基线：Avian 0.8.0、Bevy 0.20.1（Rust 1.97.1）。验证来源是[物理示例](../../../../skills/playmand/examples/physics-avian/src/lib.rs)，它在 CI 中于 Windows 和 Linux 上无窗口运行。Avian 有 `avian2d` 和 `avian3d` 两个 crate，接口相同，本页以 3D 为例。

Avian 是为 Bevy ECS 设计的物理引擎：刚体、碰撞体、速度都是普通组件，物理计算就是 Bevy 系统。新的 Bevy 游戏用它。如果项目已经在用 Rapier（`bevy_rapier3d` 0.37 支持 Bevy 0.20），就继续用 Rapier，不要换。

## 接入

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

锁定确切版本：Avian 的次版本会改接口，每个版本只对应一个 Bevy 版本。升级任何一方之前，先在 crates.io（或用 `scripts/inspect_bevy.py`）确认依赖要求的 Bevy 版本。

## 刚体与碰撞体

```rust
commands.spawn((RigidBody::Static, Collider::cuboid(20.0, 1.0, 20.0), Transform::from_xyz(0.0, -0.5, 0.0)));
commands.spawn((RigidBody::Dynamic, Collider::sphere(0.5), Transform::from_xyz(0.0, 4.0, 0.0)));
```

- `RigidBody::Dynamic` 受重力和接触影响；`Static` 永远不动；`Kinematic` 只按你设定的速度移动。
- `Collider::cuboid` 的参数是**完整长度**，不是半长：上面的地面厚 1 米，顶面在 y = 0。
- 用 `Transform` 生成，Avian 会自动同步它自己的 `Position` / `Rotation`。读取物理状态用 `Position` 和 `LinearVelocity`。
- 调参用 `Restitution`、`Friction`、`Mass`、`GravityScale`、`LockedAxes`；施力用 `LinearVelocity` 或 `ConstantForce`。全局重力是 `Gravity` 资源（默认 y 方向 −9.81）。

## 碰撞事件

只有带 `CollisionEventsEnabled` 的实体才会产生事件。批量处理用消息读取，针对单个实体用观察者：

```rust
fn hits(mut reader: MessageReader<CollisionStart>) {
    for e in reader.read() { /* e.collider1, e.collider2 */ }
}

commands.spawn((Collider::cuboid(1.0, 0.1, 1.0), Sensor, CollisionEventsEnabled))
    .observe(|event: On<CollisionStart>| { /* 有东西踩上了踏板 */ });
```

接触结束时对应的是 `CollisionEnd`。`Sensor` 只报告重叠、从不推开任何东西，适合做触发区、拾取物和终点区域。

## 时间与确定性

物理在 `FixedPostUpdate` 中按固定步长运行，结果与帧率无关。每帧读取物理结果的系统放在 `Update` 或之后；施加力的系统放在 `FixedUpdate`。

测试和录制时手动推进时间：用 `MinimalPlugins + TransformPlugin + PhysicsPlugins`，插入 `TimeUpdateStrategy::ManualDuration(1/60 秒)`，每步调用一次 `app.update()`。同一台机器上相同输入得到逐位相同的位置（示例里有断言）。跨机器的确定性需要开启 Avian 的 `enhanced-determinism` feature。

## 检查方法

断言结果，不要断言帧号：掉下的球最终停在地面上方一个半径的高度且不再移动；第一次 `CollisionStart` 出现在自由落体公式算出的时刻（t = √(2h/g)）；传感器能看到物体穿过但不会挡住它。手感要看调试视图：开启 Avian 的 `debug-plugin` feature 并加入 `PhysicsDebugPlugin`，就能画出碰撞体。

## 常见坑

- 触发器自己就触发了：传感器碰到它所在的地面或墙，也会报告这次接触。要检查进入的是哪个实体（`Player`、`Ball` 这类标记组件），而不只是传感器有没有参与。
- 没有 `RigidBody` 的碰撞体是静态的。
- 穿透：又快又薄的物体可能在一步之内互相穿过。加厚碰撞体、降低速度，或给快速物体开启扫掠式连续碰撞（`SweptCcd`）。
- 缩放 `Transform` 会同时缩放碰撞体；视觉网格和碰撞体用同一套单位（1 单位 = 1 米时与默认重力最匹配）。
- 两个实体里至少一个没有 `CollisionEventsEnabled` 时，事件会悄无声息地不出现。
