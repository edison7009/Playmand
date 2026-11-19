# Playmand 0.5.0 验证

0.5.0 补齐素材能力：先问清用户要什么、有什么；美术谁来做；素材从哪里来、授权与素材清单；检查模型、图片和音频；在 Blender 里修整模型并接入 Bevy。网站第二屏上线了雨夜港口的动态海报。技术基线不变：**Bevy 0.20.1 / Rust 1.97.1**。

| 检查 | 结果与范围 | 复现入口 |
| --- | --- | --- |
| 素材检查脚本 | `check_asset.py`（仅标准库）有 10 个单元测试，覆盖 GLB 尺寸与三角面上限、厘米单位（180 个单位高的角色）与悬空原点、必需的 Draco 扩展判定失败、内嵌 JPEG 的尺寸与功能开关、无 scenes 时子节点不重复计算、PNG 假透明与调色板透明、24 位 EXTENSIBLE 格式 WAV、静音 WAV 输出合法 JSON、WAV 削波与开头静音、OGG Vorbis 的编码与时长；实际跑过一个 446 万三角面的 Blender 导出 GLB（测得高 23.8 米）、一条 24 秒音轨和 WebP/PNG 美术图 | `python3 -m unittest tests.test_check_asset` |
| 模型四视图预览 | `render_glb.py` 用 Blender 5.3 后台模式渲染同一个 GLB 的正面、侧面、背面、四分之三视图并拼成总览图，约 36 秒 | `blender -b -P skills/playmand/scripts/render_glb.py -- model.glb sheet.png` |
| 引擎相关结论 | 素材流程指南中的 Bevy 说明（不支持 Draco/meshopt/量化网格、默认只播 Ogg Vorbis、JPEG/WebP/KTX2 需功能开关、0.20 用 `WorldAssetRoot` 加载 glTF 场景）逐条对照 Bevy 0.20.1 源码核对 | `bevy_gltf-0.20.1/src/lib.rs`、`bevy-0.20.1/Cargo.toml` |
| 网站第二屏 | 雨夜港口动态海报单独打包（gzip 后 32 KB），滚动到附近才加载；首屏卡包仍为 66 KB。浏览器中验证了桌面 16:9、4:3、手机竖屏和中英文页面；CI 对该包跑 clippy | `web/build.sh`，`.github/workflows/check.yml` |

尚未验证：图生 3D 服务产出的模型从检查、Blender 修整到在 Bevy 中运行的完整流程（本次没有使用外部生成服务）；`render_glb.py` 在 Windows 和 Linux 上的运行。

---

# Playmand 0.4.0 验证

0.4.0 把 Playmand 从"Bevy 技能"扩展为 Rust 游戏与视觉创作的入口：入口新增按任务选工具的表，以及物理（Avian）、音频（Kira）、调试界面（egui）、小型 2D（Macroquad）、自定义 GPU（wgpu）、网页特效（Rust → WebAssembly）和游戏手感的参考资料。技术基线不变：**Bevy 0.20.1 / Rust 1.97.1**。

| 检查 | 结果与范围 | 复现入口 |
| --- | --- | --- |
| 各示例在 CI 中运行 | Windows 与 Linux：`ecs-patterns`、`physics-avian`、`feedback-tiers`、`bevy-stack`（Bevy + Avian + bevy_kira_audio + bevy_egui 一起）、`audio-kira`、`ui-egui`、`macroquad-2d`（含 wasm32 构建）、`gpu-wgpu`（Linux 用 lavapipe 软件 GPU、Windows 用 WARP）；`web/` 的 fmt、test、clippy | `.github/workflows/check.yml`，各示例目录 `cargo test --locked` |
| 整套依赖的一致性 | CI 检查 `bevy-stack` 不构建 Bevy 自带的音频，且 egui、kira 各只解析到一个版本 | 同上 |
| 视频制作 | `motion-video` 示例：同一条时间线在 macOS（Metal）上离屏渲染出 16:9、9:16、1:1 三种画幅，各 270 帧、无空白帧；带声音的 MP4 校验了时长 9.000 秒、帧数、H.264 与 AAC 48 kHz 音轨；解码后的音效与画面偏差 0 ms。CI 中测试时间线、字幕和音轨，并在 Linux 上编译渲染器 | `skills/playmand/examples/motion-video`，`skills/playmand/scripts/encode_mp4.swift --verify` |
| 网页特效 | `web/` 的卡包演示（WebGL2，代码 gzip 后 58 KB）部署在 playmand.com，并在浏览器中逐帧检查过 | `web/build.sh` |
| 有无技能的对照评测 | 三批共 7 题；只要答案能在依赖源码或 README 中查到，两组都做对，技能主要减少试错；设计题上技能让方案更完整，评测也发现并修正了指南的空白 | [评估用例](docs/evaluation.md) |

尚未验证：视频的 ffmpeg 编码路径（本机没有 ffmpeg，用的是 AVFoundation）与 Windows / Linux 上的实际渲染出图；bevy_kira_audio 在真实声卡上的播放（CI 没有声卡，只检查游戏请求播放了什么）；bevy_egui 面板在真实窗口中的显示；wgpu 在真实独立显卡上的结果；Blender MCP 的连接与建模。

以下 0.3.0 说明保留为历史：

# Playmand 0.3.0 验证

0.3.0 将技术基线升级到 **Bevy 0.20.1 / Rust 1.97.1**（Bevy 0.20.1 声明的最低 Rust 版本）。ECS 示例改用新版本依赖后重新执行；参考资料中出现的 Bevy 类型、方法与字段已逐项对照 0.20.1 crate 源码，并按官方 0.19→0.20 迁移指南补充调度弱顺序、观察者泛型、`NextState::set_if_different`、`Interaction` 弃用、默认字号与 2D 色调映射等变化。旧名 `SceneRoot` / `SceneInstanceReady` 仅作为旧教程识别保留。

新增 [演示动画与视频录制](skills/playmand/references/video.md) 指南。固定时钟逐帧截图、无缝循环与 WebP 编码已用 `demos/` 的鹈鹕骑车（100 帧）和 ECS 星系（150 帧，4000 实体 + Bloom）在 macOS Metal 上实际录制，并逐帧检查首、中、末帧；另以 1920×1080 / 30fps 录制三个演示，并用 AVFoundation（`demos/encode_mp4.swift`）编码为 H.264 MP4，经系统解码生成缩略图确认可播放；录制中发现窗口被遮挡会产生黑帧，改为置顶窗口、单截图在途后逐帧检查无黑帧。ffmpeg 编码与 Blender 渲染路径未在本机执行。

以下 0.2.1 说明保留为历史：

0.2.1 新增 Blender + MCP 专项指南、入口路由和资产接入示例。本次只改变指令、文档、显示信息与包版本；执行安装/分发回归和 Skill 格式检查。下方四项 Bevy 执行记录来自 0.2.0，Rust 示例未改动。

Blender MCP 工具映射已核对上游文档与服务源码，也阅读了用户提供的本地 3d-creation 及 Blender 建模参考；仅吸收独立部件、可重跑编辑、视觉检查的设计思路，未复制原技能文件。本会话没有可调用的 Blender MCP，连接、建模、GLB 导出与 Bevy 图形接入仍为未实测，新增 B08 评估任务跟踪。

日期：2026-09-13。新版重点为 Rust + Bevy 专项指导、版本定位脚本与可执行 ECS 用法。

## 已执行的本地检查

| 检查 | 结果与范围 | 复现入口 |
| --- | --- | --- |
| Skill 格式 | 命名、frontmatter、占位符检查通过 | skill-creator 的 `quick_validate.py skills/playmand` |
| Python 回归 | 13 项通过：双宿主安装、冲突保护、中文路径/旧编码、包完整性、版本图解析、构建缓存排除 | `python -B -m unittest discover -s tests -v` |
| Rust / Bevy 真实执行 | 0.3.0：macOS aarch64、Rust 1.97.1、Bevy 0.20.1，4 项 ECS 测试与 `cargo fmt --check` 通过（2026-11-17）；0.2.x 曾在 Windows MSVC、Bevy 0.19.1 通过 | 在 `skills/playmand/examples/ecs-patterns` 执行 `cargo test --locked` |
| 版本定位脚本 | 对真实示例执行成功，报告 Bevy 0.20.1、启用 features、源码目录及 docs.rs 0.20.1 文档 | `python -B skills/playmand/scripts/inspect_bevy.py --manifest-path skills/playmand/examples/ecs-patterns/Cargo.toml --filter-platform aarch64-apple-darwin` |
| 分发内容 | ZIP 校验、SHA-256、包内相对链接与解压后双宿主安装通过；Cargo target / Python 缓存不进入安装和分发 | 包测试及 `python scripts/build_package.py` |

Python 版本图测试覆盖传递依赖、同名不同版本、同版本不同来源、无解析图的错误输入。脚本以 Cargo 元数据为依据，匹配 bevy / bevy_ 名称，其中可能包含第三方 crate；报告整个工作区，不冒充单个游戏二进制的精确依赖列表。path/git 修改版应优先看报告中的源码，发布版文档可能与其不同。

四项 Bevy 测试检查系统执行后的 World：

- 同时具有 Player / Enemy 标记的实体仍按明确互斥规则更新，不发生查询别名冲突。
- 链式后续系统在同一次 Update 看到 Commands 创建的实体。
- 同值写入不重复触发下游变化处理，真正改变值会触发。
- 连续三次进入/退出游玩状态，关卡实体每次创建一个、退出归零。

示例使用真实 Bevy、精简 features 和无窗口 App，未启动 GPU。GLB、动画、UI 与截图指南的 API 已对照 0.20.1 官方 crate 源码，尚未在本版独立图形工程中逐项运行。

CI 配置覆盖 Windows / Linux 的 Python 3.9 / 3.13，以及两平台的 Rust 1.97.1 ECS 示例；远程实际结果以对应提交的 Actions 为准。0.3.0 的 Bevy 示例仅在本地 macOS 实测，Windows / Linux 以 CI 结果为准。

## 历史宿主测试与尚未证明的收益

[0.1.0 宿主记录](docs/host-validation.md) 保留此前 Codex / Claude Code 在 Python 小样中的技能发现与执行结果。它们使用旧正文，不是 0.2.0 的行为验证，也不能证明 Rust + Bevy 游戏开发能力。

本次尚未完成：新正文的跨模型宿主行为复测、GPU 画面/动画/操作手感与完整玩家包验收，以及有无技能的同条件 A/B 实验。没有“开发质量一定更高”“提效比例”或“100% 全自动”的结论。后续用 [Bevy 评估任务](docs/evaluation.md) 衡量实际游戏结果。

安装器保持不覆盖差异内容；磁盘或权限 I/O 故障仍可能留下部分复制内容，不具有跨目录事务保证。原始规划、机器日志、宿主轨迹及构建产物保留本地，不进入公开包。
