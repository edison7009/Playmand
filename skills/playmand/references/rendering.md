# Bevy 渲染、UI 与性能

基线：Bevy 0.19.1。先处理当前游戏具体画面，按项目版本核对字段。以下是源码核对过的技术索引，GPU 效果需在目标游戏运行验证。

## 2D / 3D 的实际接线

2D 从 `Camera2d` 配合 `Sprite` 或 `Mesh2d` 开始；3D 使用 `Camera3d`、`Mesh3d`、`MeshMaterial3d<StandardMaterial>`、`Transform`。现代 Bevy 通过 required components 补齐部分基础组件，不需要恢复旧教程里的各种 `*Bundle`。材质类型必须与渲染路径匹配。

```rust
commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

相机朝向使用 `looking_at` 时确保目标与位置不重合。透视/正交投影按游戏设计选；视角、景深、色彩和反馈仍需围绕用户的美术目标调整，不能把“有一个立方体”当作画面完成。

3D 黑屏或物体纯黑先分开查：

| 现象 | Bevy 检查点 |
| --- | --- |
| 完全没有目标场景 | 活动 `Camera`、渲染目标、Transform/GlobalTransform、投影近平面/远平面、资源加载错误 |
| 只有部分对象消失 | `Visibility` 与继承可见性、包围盒/剔除、`RenderLayers`、负缩放背面、透明模式 |
| 几何存在但材质发黑 | 法线/贴图、灯光、曝光、环境光；0.19.1 全局环境光为 `GlobalAmbientLight` |
| 阴影没有出现 | 当前版本灯光字段、材质/网格投射接收设置与阴影距离；0.19.1 灯光使用 `shadow_maps_enabled` |
| 场景出现两次/覆盖 | 多相机的 target、order、清屏设置与图层；先排除同一实体被多个 pass 绘制 |

`RenderLayers` 不会沿实体层级自动继承。GLB 根设置图层后，实例内部 mesh 仍可能留在默认层；在实例就绪后处理实际后代。光源与相机/网格图层也要匹配。[RenderLayers 文档](https://docs.rs/bevy/0.19.1/bevy/camera/visibility/struct.RenderLayers.html) 解释交集语义。

## Bevy UI

0.19.1 的布局主组件是 `Node`；文本使用 `Text`、`TextFont`、`TextColor` 等。不要混用旧版 `Style` / `TextBundle` 字段。布局用父容器约束表达，区分百分比尺寸与像素尺寸；绝对定位的装饰与内容流分开。

按钮可从 `Changed<Interaction>` 消费按下/悬停状态。主题更新只写改变的值，不在每帧重建整个 UI 树。焦点、文本编辑、提交错误和鼠标悬停是不同状态；英文字符输入通过不证明中文 IME 路径正常。

读取实际尺寸用布局后的 `ComputedNode`，在 `PostUpdate` / `UiSystems::Layout` 之后；注意物理像素与 UI 逻辑坐标/DPI 换算。多相机情况下在相应 UI 根指定 `UiTargetCamera`，避免主 HUD 意外出现在角色预览纹理。

## 截图与离屏预览

当前版本的官方 `examples/window/screenshot.rs` 演示：

```rust
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
// 在系统中，一次性发出请求：
commands.spawn(Screenshot::primary_window()).observe(save_to_disk("capture.png"));
```

截图经过渲染和 GPU 回读，命令返回不代表文件已保存。只在目标资源/场景就绪后请求，等待捕获和落盘，并打开实际图片检查。离屏 target 与主窗口选择必须一致。不要每帧请求截图造成回读堆积。

渲染到纹理的预览涉及 `Image` 资产、`RenderTarget`、相机及 UI 显示；按同版本 `examples/ui/render_ui_to_texture.rs` 接线。关闭面板时仅隐藏 UI 不会自动关闭相机；按生命周期设置 `Camera::is_active`，并停止无用姿态更新。静态预览可在变化后刷新；动画预览仍要持续渲染，不能用静止截图代替。

## 有测量依据的优化

先区分编译慢、主线程模拟慢、渲染提交慢和 GPU 慢。可用 `FrameTimeDiagnosticsPlugin` 看总体帧时间，需要归因时使用目标环境已有 profiler；FPS 不是某一个系统或 GPU pass 的耗时。

- ECS 热点：每帧全表扫描、重复字符串分配、无效 change detection、大范围串行系统依赖。
- 渲染热点：额外相机、阴影灯光、透明过绘、高分辨率预览纹理和过重后处理。
- 资源热点：重复 `Assets::add`、长期强句柄缓存、关卡退出后残留实体/任务。

复用资产、收窄查询和关闭无用相机应服务于测到的问题。不要为提帧擅自减少用户要求的画质或内容。对比使用相同 release 配置、场景、硬件、窗口尺寸、前后台状态和预热条件；离线视频的编码帧率不能当运行帧率。

查找入口：同版本官方 `examples/2d`、`examples/3d`、`examples/ui`、`examples/window/screenshot.rs`；[Bevy UI API](https://docs.rs/bevy/0.19.1/bevy/ui/index.html)。
