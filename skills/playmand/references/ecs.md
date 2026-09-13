# ECS、调度与玩法状态

以下符号以 Bevy 0.19.1 为基线。可运行代码见 [ecs-patterns](../examples/ecs-patterns/src/lib.rs)。

## 数据归属与 Rust 借用

- 实体上的能力/数据用 `Component`，如速度、生命、阵营；世界唯一的配置或共享进度用 `Resource`。不要把所有实体数据复制进一个全局可变对象绕开 ECS。
- `Query<&mut Transform, With<Player>>` 与 `Query<&mut Transform, With<Enemy>>` 不保证互斥：实体可以同时拥有两个标记。逻辑应互斥时用 `Without` 表达；确实需要重叠且顺序访问时使用 `ParamSet`，结束 `p0()` 的借用后再取 `p1()`。不要靠添加 `Mutex` 或 clone 整个世界消除 B0001。
- `Query::single()` 在 0.19.1 返回 `Result`。零个/多个玩家在加载或切场景时是否正常，由业务决定；正常的过渡状态用 `let Ok(...) = ... else { return; }`，不要把所有错误都无条件吞掉。
- Bevy 的 change detection 追踪可变解引用，不自动比较字段是否相等。重复写入相同 `Transform`/UI 数据也可能触发下游重算。对热点中的可比较值使用 `set_if_neq`，或先比较再写；新增组件仍会被视为变化。

## 调度不是函数声明顺序

```rust
app.add_systems(Update, (spawn_projectile, resolve_projectile).chain());
```

`Commands` 是延迟写入；在默认调度构建设置下，0.19.1 的 `.chain()` / `.after()` / `.before()` 会在必要的依赖边插入 `ApplyDeferred`，让下游系统看到前面的命令。仅写 `(a, b)` 不提供顺序。`chain_ignore_deferred()` 等明确省略该同步；关闭自动插入时也需自行处理。

只为存在数据依赖的系统排序；把全游戏链成串会损失并行机会。`.after(foo)` 不会自动注册 `foo`，且不能为两个不同 schedule 建立跨 schedule 的先后边。先定位系统实际所在的 `PreUpdate` / `FixedUpdate` / `Update` / `PostUpdate`。

修改 `Transform` 后立即读取 `GlobalTransform`，可能读到传播前结果。需要最终世界坐标的系统安排在 `PostUpdate` 的 `TransformSystems::Propagate` 之后；需要最终 UI 尺寸的读取安排在 `UiSystems::Layout` 之后。不要通过盲等若干帧掩盖顺序错误。

## 输入与时间步

连续运动使用时间步缩放，例如方向先 `normalize_or_zero()`，再乘速度和秒数，避免斜向更快。碰撞/物理模拟采用固定时间步时，在 `FixedUpdate` 使用固定模拟时间；摄像机、UI 和视觉插值可保持帧更新。

一帧中固定更新可能执行零次或多次。直接在 `FixedUpdate` 读取帧级 `just_pressed`，可能丢一次按下或重复应用。用与该版本官方固定时间步示例一致的输入采集顺序：把一次性动作记入待消费状态，模拟步消费后清除；连续方向可保留到下次采样。不要在第一步消费时顺带清掉仍按住的方向。

暂停用玩法状态及系统运行条件表达；明确哪些时钟继续。`Time<Virtual>` 可暂停/缩放，`Time<Real>` 不受游戏暂停影响，`Time<Fixed>` 用于固定模拟。菜单动画若应继续，不应误接暂停的模拟时钟。

## 状态、消息与重开

`States` / `NextState` 管理菜单、游玩、结算等真正互斥状态；`OnEnter` 创建该状态内容，`OnExit` 清理或使用 `DespawnOnExit`。`NextState::set` 请求转移，不能认为同一系统后半段已经进入新状态。已有 `DefaultPlugins` 与精简 `App` 的插件集合不同；精简测试需显式添加 `StatesPlugin`。

重开同时处理状态所属实体和持久 `Resource`：销毁关卡不自动清零比分、冷却或输入队列。选择哪些跨关保留由游戏设计决定。可执行示例验证状态退出清理；实际游戏另测比分等资源重置。

0.19.1 中缓冲消息用 `Message` / `MessageWriter` / `MessageReader`，观察者事件用 `Event` / `On`。前者适合调度中的批量处理，后者适合触发观察者；不要照搬旧版 `EventReader`。临时缓冲消息不适合当暂停菜单期间无限保存的任务队列，需要持久积压时显式存入资源。

依据：[Query](https://docs.rs/bevy/0.19.1/bevy/ecs/system/struct.Query.html)、[调度顺序与延迟命令](https://docs.rs/bevy/0.19.1/bevy/ecs/schedule/trait.IntoScheduleConfigs.html)、同版本 crate 的 `examples/ecs`、`examples/time`、`examples/movement/physics_in_fixed_timestep.rs`。项目使用其他版本时查对应源码。
