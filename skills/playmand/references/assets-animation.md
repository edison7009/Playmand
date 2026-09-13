# Bevy 资源、GLB 与动画

基线：Bevy 0.19.1。这里的符号已对照同版本 crate 源码；本仓库的无窗口 ECS 示例不验证 GLB 或 GPU 渲染。

## 加载与实例化是不同阶段

`AssetServer::load` 立即返回句柄，不代表资产加载完成。`get_load_states` 可区分根资产、直接依赖和递归依赖状态；需要整条依赖就绪时使用 `is_loaded_with_dependencies`。加载失败应呈现错误或回退，不能一直停在 Loading。

0.19.1 的 GLB 子场景加载写法：

```rust
commands.spawn(WorldAssetRoot(
    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/hero.glb")),
));
```

这段放在具有 `Commands` / `Res<AssetServer>` 的系统中，并启用 GLTF 与渲染所需插件/features。`Scene(0)` 只是示例，实际索引或名称来自目标文件。GLTF 可能依赖外部纹理或 bin 文件，分发时也必须带上。

等待 `On<WorldInstanceReady>` 后再搜索实例后代实体、网格和动画播放器。根资产加载、场景子实体创建、动画图绑定、GPU 第一帧显示各有不同的就绪条件。不要用“等待 32 帧”替代这些条件。

## 根资产、共享句柄与局部修改

需要同一文件的多个场景/动画时，可持有 `Handle<Gltf>`，待 `Assets<Gltf>::get` 成功后从 `scenes` / `named_scenes` 与 `animations` / `named_animations` 取得句柄。保留根句柄或实际使用的强句柄维持所需资产生命周期。不要每帧重新发起相同加载。

多个实体使用同一个网格/材质句柄会共享资产。改 `Assets<StandardMaterial>` 中该句柄的值会影响所有引用者。只有一个角色需要染色时，克隆对应材质值、`materials.add` 一次，再替换该实例网格上的 `MeshMaterial3d`；不要每帧生成材质。预览与主角色可以共享不可变资产，但需要独立的场景实体、骨骼姿态和播放状态。

反复切换角色时，按实例根清理所属实体，并移除不再使用的资源句柄/缓存项。仅隐藏实体不等于卸载资产；仅卸载 GLB 根也不保证所有子资产已无强引用。用相同状态下的实体、资产和活动相机计数判断生命周期，不能凭一次工作集上升判断泄漏。

## 让动画真正播到当前模型

0.19.1 的动画接线涉及：

1. 将目标 `Handle<AnimationClip>` 放入 `AnimationGraph::from_clip`，取得图和 `AnimationNodeIndex`；索引属于该图，不能拿任意动画数组下标代替。
2. 把图放入 `Assets<AnimationGraph>`，保留返回句柄。
3. 在 `WorldInstanceReady` 对应的实例后代中找到 `AnimationPlayer`；它未必在你创建的根实体上。
4. 给该播放器实体插入 `AnimationGraphHandle`，再 `player.play(index).repeat()`。切换和混合需要时参考 `AnimationTransitions`。

这个接线不负责自动重定向任意骨架。动画目标名称/路径、骨架层级、导出绑定姿势必须匹配。T-pose 先查 clip、图、player 和目标绑定；角色滑步再查动画根位移与代码移动是否重复叠加。不要为隐藏错误而给骨骼逐帧写全局硬编码姿势。

## 素材对游戏结果的影响

模型导入完成后核对尺寸、坐标轴、原点、法线、透明模式和纹理颜色空间；先调整明确的问题，避免同时改相机、曝光和材质。没有目标风格素材时，根据宿主已有生成工具或用户提供的资产准备素材，并在游戏中实际检查。占位几何能验证机制，但不等于达到了用户指定的画面风格。

API 与接线依据：[GLTF API](https://docs.rs/bevy/0.19.1/bevy/gltf/index.html)、[AssetServer](https://docs.rs/bevy/0.19.1/bevy/asset/struct.AssetServer.html)、同版本官方 `examples/animation/animated_mesh.rs` 与 `animated_mesh_control.rs`。通过 [版本定位工具](../scripts/inspect_bevy.py) 找到本地对应示例，避免依赖不断变化的 main 分支。
