<div align="center">

<img src="assets/logo.svg" alt="Playmand logo" width="112">

# Playmand

### 面向 AI Agent 的 Rust 视觉创作技能 · 默认 Bevy

先找到惊艳的画面，再用 **Rust** 高性能地做出来：<br/>
游戏、视频、网页特效，和你心里想的一样好看。

[![Bevy 0.20.1](https://img.shields.io/badge/Bevy-0.20.1-eb6834)](https://bevy.org/)
[![Rust 1.97.1](https://img.shields.io/badge/Rust-1.97.1-b7410e?logo=rust)](https://rust-lang.org/)
[![Avian 0.8](https://img.shields.io/badge/Avian-0.8-4b7bec)](https://github.com/avianphysics/avian)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-skill-f5f5f5)](https://code.claude.com/docs/en/skills)
[![Codex](https://img.shields.io/badge/Codex-skill-f5f5f5)](https://learn.chatgpt.com/docs/build-skills)
[![MIT License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![skills.sh](https://skills.sh/b/edison7009/Playmand)](https://skills.sh/edison7009/Playmand)

[English](README.md) | **简体中文**

</div>

<p align="center">
  <img src="assets/demo-icons.webp" alt="两团在 3D 中滚动的星云（星球、经纬地球、带环行星、圆环）以旋涡、降雨、大爆炸、扫描四种方式汇聚成 Claude 和 ChatGPT 图标。" width="100%">
</p>

<p align="center"><a href="https://github.com/edison7009/Playmand/releases/download/demo-videos/playmand-icons-1080p.mp4"><b>▶ 观看 1080p 高清视频（MP4）</b></a></p>

<p align="center"><sub>用 Playmand 制作：3600 个 Bevy 粒子化作在 3D 中滚动的圆形星云，再以四种不同方式汇聚成 Claude 和 ChatGPT 图标。<br/>粉丝致敬作品，与 Anthropic、OpenAI 无关联，也未获其认可；Claude 与 ChatGPT 分别为其所有者的商标。</sub></p>

## 为什么在 AI 写代码的时代选 Bevy

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/scorecard-zh-dark.svg">
  <img alt="AI 编码时代的引擎评分：Bevy 在 AI 代码开发友好度（9.5）和 ECS / 数据驱动（10）上领先，可视化编辑器（5）较弱。" src="assets/scorecard-zh-light.svg" width="100%">
</picture>

- **AI 代码开发友好度（9.5）和 ECS / 数据驱动（10）两项最高。** Bevy 游戏完全由 Rust 代码构成：组件、系统和状态都写在代码里，没有藏在场景文件或编辑器面板里、Agent 看不到的设置。Cargo 的编译和测试能立刻告诉 Agent 哪里写错了。
- **短板是可视化编辑器（5），而 Agent 恰恰不需要它。** 编辑器是给人点来点去用的，Agent 直接写代码。画面这一侧由 Playmand 补上：用 Blender + MCP 做资产，用截图检查实际画面。
- **2D 和 3D 都够用（8–8.5）。** 足以做完整游戏，不只是原型。

<details>
<summary>评分表格</summary>

| 维度 | Bevy | Godot | Unity | Unreal |
| --- | ---: | ---: | ---: | ---: |
| AI 代码开发友好度 | **9.5** | 8.5 | 8 | 7.5 |
| ECS / 数据驱动 | **10** | 7 | 8 | 8 |
| 2D 游戏 | **8.5** | 9.5 | 8.5 | 7 |
| 3D 游戏 | **8** | 8 | 9 | 10 |
| 可视化编辑器 | **5** | 9.5 | 10 | 10 |

评分来自 AI 助手的主观对比，不是基准测试。修改分数后运行 `python assets/make_scorecard.py` 重新生成图表。
</details>

## 为什么是 Rust：C 级速度，编译器替 Agent 把关

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/langspeed-zh-dark.svg">
  <img alt="相对 C 的执行时间：Rust 1.04 倍，C++ 1.56 倍，Java 1.89 倍，Go 2.83 倍，C# 3.14 倍，JavaScript 6.52 倍，TypeScript 46.20 倍，Python 71.90 倍，Lua 82.91 倍。" src="assets/langspeed-zh-light.svg" width="100%">
</picture>

- **27 种语言中速度第 2、能效第 2。** 在上图的研究中，Rust 的执行时间是 C 的 1.04 倍，能耗是 C 的 1.03 倍，领先 C++（1.56 倍 / 1.34 倍），更远超常用的游戏脚本语言：C# 3.14 倍、JavaScript 6.52 倍、Python 71.90 倍、Lua 82.91 倍。
- **没有垃圾回收。** 不会因为回收内存突然卡一帧，游戏和长时间渲染的帧时间都更稳定。
- **编译器替 Agent 审代码。** 类型、所有权和借用检查在程序运行前就拦下空指针、数据竞争和内存错误。Agent 拿到的是精确的报错信息，而不是运行时崩溃，这正是 Agent 最擅长利用的反馈。
- **一套工具链。** `cargo` 负责构建、测试和格式化，`Cargo.lock` 锁定所有依赖，不需要每个项目各选一套构建系统。
- **代价也是真实的：编译慢、学习曲线陡。** 对人来说，这是不选 Rust 的主要原因；对 Agent 来说，严格反而是帮助，Bevy 的细节由 Playmand 补上。

| | Rust | C++ | C# | GDScript | Python | TypeScript |
| --- | --- | --- | --- | --- | --- | --- |
| 执行时间（相对 C） | **1.04 倍** | 1.56 倍 | 3.14 倍 | 未测¹ | 71.90 倍 | 46.20 倍 |
| 内存管理 | **所有权，无 GC** | 手动 | GC | 引用计数 | 引用计数 + GC | GC |
| 运行前能发现的错误 | **类型、所有权、借用** | 类型 | 类型 | 可选类型 | 默认没有 | 类型（运行时擦除） |
| 标准构建工具 | **cargo** | 没有（CMake 等） | dotnet | Godot 编辑器 | pip、uv 等 | npm、pnpm 等 |
| 编译速度 | 慢 | 慢 | 快 | 无需编译 | 无需编译 | 快 |

¹ GDScript 是解释型语言，不在该研究范围内。数据来自研究中 2017 年 Computer Language Benchmarks Game 的程序；运行 `python assets/make_langchart.py` 重新生成图表。

## 用 Playmand 做的演示

游戏和视频来自同一套代码：能实时游玩的 Bevy 场景，也能逐帧录制成无缝循环的动画。下面两个演示都由 AI 编程 Agent（Claude Code）借助本技能编写。

| 鹈鹕骑自行车 | ECS 星系 |
| :---: | :---: |
| <img src="assets/demo-pelican.webp" alt="一只鹈鹕在公路上蹬自行车，云和山丘在背景中滚动" width="100%"> | <img src="assets/demo-galaxy.webp" alt="由 4000 颗星星组成、正在旋转的发光螺旋星系" width="100%"> |
| 著名的"鹈鹕骑自行车"大模型测试（由 Simon Willison 推广）。这里不是画一张 SVG，而是一个会动的 Bevy 场景：车轮转速和路面速度匹配，双腿用两段式 IK 跟着踏板转。 | 4000 个星星实体，HDR 辉光，由一个并行 ECS 系统驱动。内圈星星转得更快，旋臂会先卷紧，每轮结束时又回到初始形状。 |
| [▶ 观看 1080p 高清视频（MP4）](https://github.com/edison7009/Playmand/releases/download/demo-videos/playmand-pelican-1080p.mp4) | [▶ 观看 1080p 高清视频（MP4）](https://github.com/edison7009/Playmand/releases/download/demo-videos/playmand-galaxy-1080p.mp4) |

源码：[`pelican.rs`](demos/src/bin/pelican.rs)、[`galaxy.rs`](demos/src/bin/galaxy.rs)，以及页首动画 [`icons.rs`](demos/src/bin/icons.rs)。在 `demos/` 目录运行 `cargo run --bin pelican`、`galaxy` 或 `icons`。加上 `-- --record frames` 可录制一个无缝循环，再用 `python demos/encode.py frames out.webp` 生成动画。

### 在浏览器里直接玩：用 Rust 构建的抽卡系统

<a href="https://playmand.com/zh/"><img src="assets/demo-cards-zh.webp" alt="五张带卡通角色的全息卡片并排展开，其中一张随鼠标倾斜" width="100%"></a>

<p align="center"><a href="https://playmand.com/zh/"><b>▶ 在线体验</b></a> · 撕开卡包，再把鼠标移到卡片上</p>

Rust 编译成 WebAssembly，在普通网页里直接用 WebGL2 绘制：没有游戏引擎，也没有 JavaScript 3D 库，代码仅 66 KB（WebAssembly + JS，gzip 压缩后）。沿随机裂口撕开铝箔卡包，五张卡从开口升起并依次翻成正面，每张都有真实厚度、随倾斜反光的透明涂层，以及浮在卡面上方的角色。角色用 Bevy 建模渲染。它最初是 [EchoBird](https://github.com/edison7009/EchoBird) 的 TokenPet 功能。源码与构建说明见 [`web/`](https://github.com/edison7009/Playmand/tree/main/web)，经验总结写进了技能的[网页特效指南](docs/zh-CN/playmand/references/web-wasm.md)。

### 两张图，一张动态海报

<a href="https://playmand.com/zh/"><img src="assets/demo-harbor.webp" alt="雨夜港口里站着一台胸口发出橙色光芒的巨型机甲，探照灯扫过雨幕，水从装甲上倾泻而下，起重机后方闪电划过" width="100%"></a>

这是一张动态海报，不是角色动画：机甲和港口是生图软件生成的两张静态图，机甲本身不做动作。动的部分由 Rust + WebAssembly 用 WebGL2 在它们周围实时绘制，代码仅 32 KB：三层雨、来回扫的探照灯、闪烁的起重机信号灯、雾、海面波纹、闪电，以及从装甲上流下的水。它放在 [playmand.com](https://playmand.com/zh/) 的第二屏。每一帧都是时间的函数，所以同一份代码也能录成 24 秒无缝循环的视频，音轨来自同一条时间线。方法见技能的[美术方向指南](docs/zh-CN/playmand/references/art-direction.md)。

### 用 Rust 离屏渲染的宣传片

<img src="assets/demo-promo.webp" alt="9 秒宣传片：Playmand 标题逐字弹入并迸发粒子，柱状图显示 Rust 的执行时间为 C 的 1.04 倍，并与 C++、C#、JavaScript 对比，最后是带播放按钮标志和 playmand.com 的结尾卡" width="100%">

游戏式视频：字母带弹性落位，粒子踩着节拍迸发，每次切镜头有色带擦除，柱子回弹后停稳，结尾卡水波纹扩散。一条时间线（镜头、交叉淡化、字幕、音效提示点、品牌包），用 Bevy 不开窗口渲染出 16:9、9:16、1:1 三种画幅，带音轨和 SRT 字幕，编码成有声音的 MP4。每一帧画面、每一句字幕、每一声音效都来自同一组数字：每根柱子到位时的"嗒"声正好落在那一帧上。源码：[`motion-video`](skills/playmand/examples/motion-video/src/lib.rs)；方法：[动态图形与视频制作指南](docs/zh-CN/playmand/references/motion-graphics.md)。

## 做一个游戏或视频

```text
/playmand 做一个俯视角 2D 动作游戏，有冲刺、近战、追踪玩家的敌人、掉落和升级。
明亮的像素风，打击反馈要有力。
```

```text
/playmand 做一段 6 秒无缝循环动画：我们的 logo 由发光粒子汇聚而成，
1920x1080，导出 MP4，另导出一份 WebP 放进 README。
```

```text
/playmand 给我们的应用做一条 20 秒发布视频，要 16:9 和 9:16 两版：
问题、产品、三个关键功能、结尾卡。用我们的品牌色和字体，
字幕烧进画面，配一段轻音乐，每次切镜头有一声音效。
```

Codex 中使用 `$playmand`。在 Claude Code 中一行即可安装（其他 Agent 见 [安装](#安装)）：

```text
/plugin install playmand --marketplace edison7009/Playmand
```

## Playmand 是什么

**Playmand 的价值在于画面。** 游戏逻辑和玩法，Agent 很快就能写出来；真正稀缺的是做得好看。所以 Playmand 先帮你找到想要的画面（你的参考图、你的生图软件、生图 API，或宿主自带的生图能力），再用 Rust 的高性能（Bevy、着色器、粒子、WebAssembly）按这个标准把它做出来，并拿成品和目标图对比。

**Playmand 是用 Rust 做游戏与视觉创作的 Skill，默认引擎为 Bevy。它帮 AI 按任务选对 Rust 工具（游戏和录制视频用 Bevy，物理用 Avian，音频用 Kira，调试工具用 egui，小型 2D 游戏用 Macroquad，自定义 GPU 工作用 wgpu，网页和 Tauri 应用里的特效用 Rust → WebAssembly），把设计实现为可玩的 2D / 3D 游戏、逐帧录制的动画，或页面里的特效。Playmand 的发展方向见[路线图](docs/zh-CN/roadmap.md)。**

核心是 Rust + Bevy 的具体开发能力：正确使用当前版本的 API、组织 ECS 与玩法状态、接入模型动画、实现 UI 和渲染、定位性能问题并构建游戏。Codex、Claude Code 及其他支持技能的工具是使用它的宿主。

**Blender + MCP 是资产制作的重要组成部分**：AI 通过已连接的 MCP 操作 Blender，制作和修改模型、材质、骨骼与动画，检查结果后导出 GLB 并接入 Bevy。技能覆盖从可编辑资产到游戏内呈现的技术衔接。

新建游戏默认 Rust + Bevy；已有 Bevy 工程保留其版本和结构。用户明确选用其他引擎时尊重选择，不自动迁移已有项目。技能不规定固定开发循环，也不把用户要求缩减成最小演示。

## 为什么是 Rust + Bevy + Agent

Rust 的吸引力首先在于运行效率和可靠性：编译为原生代码、无需垃圾回收，并能精细控制内存与资源；类型系统和所有权检查可以在编译阶段发现许多内存与并发错误。这些特性适合重视帧时间、资源占用和长期维护的游戏项目。[Rust 官方介绍](https://rust-lang.org/)

Bevy 的 ECS 架构围绕数据组织游戏逻辑，支持在条件允许时并行执行系统，模块化设计也便于按需求扩展。玩法、状态和系统行为可以通过 Rust 代码表达，方便 Agent 阅读、修改，并结合 Cargo 编译和测试获得反馈。[Bevy 官方介绍](https://bevy.org/learn/quick-start/introduction/)

代价是代码门槛较高：所有权、借用、生命周期，以及 Bevy 的查询、调度和资源管理，都需要开发者理解。构建耗时、版本变化和复杂错误排查也会增加迭代成本。运行效率高，并不意味着开发过程天然更快。

**这正是 Playmand 与 Agent 结合的核心：发挥 Rust + Bevy 的性能与工程优势，让 Agent 承担更多代码实现、API 查证和问题定位，降低驾驭这套技术栈的成本。** 用户把精力放在玩法、风格和体验判断上，Playmand 为 Agent 补充引擎与资产制作知识，帮助它少猜接口、少踩技术陷阱。实际开发效率与游戏质量，仍通过具体项目和对照评估检验。目前的对照评测里：答案能在依赖源码中查到的题，有无技能都做对了，技能主要省时间；设计题（升级手感）上，盲评起初更喜欢不用技能的方案，暴露了指南的空白；修正指南和示例后，技能组方案以 20 分对 16 分胜出。方法、原始结果和局限见[评估用例](docs/evaluation.md)。

## 提供什么

| 内容 | 具体帮助 |
| --- | --- |
| [Rust / Bevy 工具链](docs/zh-CN/playmand/references/bevy.md) | 新工程配置、版本/API 定位、feature 与插件兼容、编译及 Windows 链接 |
| [ECS 与玩法状态](docs/zh-CN/playmand/references/ecs.md) | Query 借用冲突、延迟命令、系统顺序、固定时间步输入、暂停与重开 |
| [Blender 与 MCP](docs/zh-CN/playmand/references/blender-mcp.md) | 连接与能力发现、场景编辑、模型/材质/动画制作、GLB 导出及 Bevy 接入 |
| [资源与动画](docs/zh-CN/playmand/references/assets-animation.md) | GLB 子场景、实例就绪、动画图/播放器接线、材质与骨骼实例隔离 |
| [渲染、UI 与性能](docs/zh-CN/playmand/references/rendering.md) | 2D/3D 相机、灯光、图层、UI 布局、截图与预览相机生命周期 |
| [美术方向：先有图，再动手](docs/zh-CN/playmand/references/art-direction.md) | 动手前先拿到目标效果图：来自用户的参考图、宿主自带的生图能力，给用户生图软件（Gemini、豆包、ChatGPT……）的提示词、用户接入的生图 API，或找来的现成图片作参考；然后定风格、做一致的素材、抠图，并和目标图对比验收 |
| [平面设计](docs/zh-CN/playmand/references/graphic-design.md) | 用一套版式输出海报、社交配图、链接分享图和幻灯片：300 dpi 带出血的印刷尺寸、各平台安全区、字号层级，以及重叠、可读字号和对比度检查，在 CPU 上渲染 |
| [App 界面升级](docs/zh-CN/playmand/references/app-ui.md) | 给已有应用换新界面：先在预览页里用它的主题和图标多版对比，从图标自动取品牌色，安静的卡片，选中不用高亮边框，悬停不位移，堆叠卡片目录，再不改行为地移植（在副本里出补丁，或直接在应用仓库里提交），在浏览器里模拟后端检查真实组件 |
| [素材流程](docs/zh-CN/playmand/references/asset-pipeline.md) | 素材从哪里来（用户、授权清晰的素材库、生成工具）、记录尺寸和授权的素材清单、模型/图片/音频检查脚本、四视图模型预览、在 Blender 里修整模型并接入 Bevy |
| [程序化写实](docs/zh-CN/playmand/references/procedural-realism.md) | 让代码做出的 3D 看起来真实：建模工具箱、损伤、烤入磨损、贴花、湿润材质、光、合成声音和审查接口 |
| [动态图形与视频制作](docs/zh-CN/playmand/references/motion-graphics.md) | 产品演示、讲解、数据故事和社交短视频：分镜、品牌包、节奏、16:9 / 9:16 / 1:1 安全区、字幕、音轨、导出检查 |
| [演示动画与视频录制](docs/zh-CN/playmand/references/video.md) | 离屏或窗口录制、无缝循环、辉光、带声音的 WebP/GIF/MP4 编码 |
| [用 Avian 做物理](docs/zh-CN/playmand/references/physics.md) | 刚体与碰撞体、碰撞事件与传感器、快速物体不穿墙、无窗口物理测试 |
| [用 Kira 做音频](docs/zh-CN/playmand/references/audio.md) | 按分贝管理音乐和音效声道、淡入淡出与交叉淡化、暂停、无声卡检查电平 |
| [用 egui 做调试面板](docs/zh-CN/playmand/references/egui.md) | bevy_egui 接入与调度、egui 版本对齐、点面板不影响游戏 |
| [用 Macroquad 做小型 2D 游戏](docs/zh-CN/playmand/references/macroquad.md) | 什么时候比 Bevy 合适、固定步长规则、小体积网页构建 |
| [用 wgpu 做自定义 GPU 工作](docs/zh-CN/playmand/references/wgpu.md) | 计算与渲染通道、读回结果、没有 GPU 时的处理 |
| [Rust → WebAssembly 网页特效](docs/zh-CN/playmand/references/web-wasm.md) | 网页和 Tauri 应用里的 WebGL2 特效：首帧快、体积、隐藏时暂停、释放 GPU 内存 |
| [游戏手感：奖励与升级](docs/zh-CN/playmand/references/feedback.md) | 分层反馈、预备与冲击、狂点时也跟得上、安全取消、减少动态效果 |
| [版本检查脚本](skills/playmand/scripts/inspect_bevy.py) | 从 Cargo 实际解析图报告 Bevy 版本、features、源码位置和文档入口 |
| [可运行示例](docs/zh-CN/playmand/README.md) | 在 Windows 和 Linux 的 CI 中运行的小型 Rust 工程：Bevy ECS、Avian 物理、Kira 音频、egui、Macroquad、wgpu、游戏手感、[平面设计](skills/playmand/examples/poster-design/src/lib.rs)、[视频](skills/playmand/examples/motion-video/src/lib.rs)，以及[整套技术栈一起运行](skills/playmand/examples/bevy-stack/src/lib.rs)（Bevy + Avian + Kira + egui 放在同一个应用里） |

技术示例基线为 **Bevy 0.20.1 / Rust 1.97.1**。其他版本需查对应源码，已有项目不应为套用示例而升级。辅助资料按问题读取，完整入口见 [SKILL.md](skills/playmand/SKILL.md)（英文，Agent 实际加载的版本）；中文副本见 [docs/zh-CN/playmand](docs/zh-CN/playmand/README.md)。

## 安装

**Claude Code（2.1.275 及以上）**：直接从本仓库安装插件：

```text
/plugin install playmand --marketplace edison7009/Playmand
```

较旧版本先运行 `/plugin marketplace add edison7009/Playmand`，再运行 `/plugin install playmand@playmand`。

**任何支持 Skill 的 Agent（通过 [skills.sh](https://skills.sh)）**：需要 Node.js：

```sh
npx skills add edison7009/Playmand
```

**Codex 和 Claude Code，使用自带安装器**：需要选择个人或项目范围，或不想装 Node.js 时使用：

要求已安装 Codex 或 Claude Code，以及 Python 3.9+。安装器仅使用 Python 标准库；游戏开发所需的引擎、编译器、图形与输入工具按目标项目准备。它不会自动安装或购买这些工具。

使用 Blender MCP 时，还需在实际宿主配置相应 MCP 服务并启用 Blender 端插件；技能安装不会自动完成连接。具体实现与接入边界见 [Blender MCP 指南](docs/zh-CN/playmand/references/blender-mcp.md)。

```sh
git clone https://github.com/edison7009/Playmand.git
cd Playmand
python scripts/install_skill.py --user --target both
```

也可下载 GitHub 的 ZIP，解压并进入目录后执行同一个 Python 命令。若只安装一个宿主，将 `both` 改成 `codex` 或 `claude`。不使用 Python 时，手动复制整个 `skills/playmand` 文件夹到下表目录。

| 宿主 | 个人安装（`--user`） | 项目安装 | 显式调用 |
| --- | --- | --- | --- |
| Codex | `~/.agents/skills/playmand/` | `.agents/skills/playmand/` | `$playmand 帮我完成……` |
| Claude Code | `~/.claude/skills/playmand/` | `.claude/skills/playmand/` | `/playmand 帮我完成……` |

`~` 表示当前用户主目录。个人安装可供该用户多个项目使用；要把技能随游戏项目共享给团队，改用项目安装：

```sh
python scripts/install_skill.py --project "已有游戏项目的绝对路径" --target both
```

两个范围二选一，避免同名重复安装。项目安装只复制到指定目录内；安装器不会改动游戏源码或宿主配置文件。普通文件占用安装路径、同名不同内容或指向项目外的路径会被拒绝。

目录规则参照 [Codex 官方 Skill 文档](https://learn.chatgpt.com/docs/build-skills) 与 [Claude Code 官方 Skill 文档](https://code.claude.com/docs/en/skills)，2026-09-13 核对。共享正文使用 `name` / `description`、MIT 许可标记和相对路径，不依赖宿主专有指令语法；`agents/openai.yaml` 是 Codex 附加显示信息。

安装后打开游戏项目的新会话，检查技能是否被发现，再显式调用。更旧或受组织策略限制的宿主需按其实际版本确认加载行为；文件复制成功不等于宿主已加载。这里指本地 Codex / Claude Code，云端任务还需将技能带入其实际执行环境。

安装器遇到相同内容会跳过，遇到同名但不同内容的目录会退出且不覆盖。示例构建产生的 `target` 和 Python 缓存不复制，也不作为内容冲突。更新时先 `git pull`，检查新旧差异并把旧安装目录移到技能发现目录之外留存，再重新安装；安装副本不自动同步。源文件统一维护在本仓库 `skills/playmand`。

## 首次构建较慢，之后很快

Rust 游戏第一次构建时，Cargo 要把所有依赖下载并编译一遍。Bevy 游戏这一步需要几分钟：在 Apple M4（10 核）上，依赖已下载的情况下，一个小型 Bevy 应用从零开始的调试构建用了 **6 分 15 秒**，`target/` 占用 6.8 GB。机器较慢或还需要下载依赖时会更久。这期间没有卡住，耐心等待即可。

之后 Cargo 会复用已编译的依赖，只重新编译你自己的代码。同一台机器上，修改游戏代码后再次构建只用了 **3 秒**。更轻的技术栈一开始就快：Macroquad 和 egui 示例从零构建分别只要 4 秒和 10 秒。

只有删除 `target/`、更换 Rust 工具链、新增或升级依赖、修改 feature，或第一次构建 `--release` 时，才会再经历一次较长的构建。技能会在首次构建前提醒你，不会让你莫名等待。

## 使用

Codex 新游戏示例：

```text
$playmand 用 Rust + Bevy 开发一个俯视角 2D 动作游戏。
需要冲刺、近战攻击、敌人追踪、掉落和升级，以及完整的开始、暂停和结算界面。
画面采用明亮的像素风格，重点做好打击反馈和操作手感。
```

现有工程示例：

```text
$playmand 修复当前 Bevy 工程：GLB 角色已经显示，但动画不播放。
沿用 Cargo.lock 的版本，检查场景实例、AnimationPlayer 和动画图的接线。
```

资产制作示例：

```text
$playmand 通过已连接的 Blender MCP 制作一个低多边形宝箱，包含开盖动画。
保留可编辑的 .blend，导出 GLB，接入当前 Bevy 工程，点击后播放开盖动画。
```

Claude Code 将 `$playmand` 换成 `/playmand`。其他宿主按其技能加载方式使用整个 `skills/playmand` 目录；本仓库安装器只提供 Codex 和 Claude Code 的目录适配，不宣称所有工具已测试兼容。

## 验证与维护

- [验证记录](VALIDATION.md)：已执行的安装、宿主与分发检查，以及当前验证范围。
- [评估用例](docs/evaluation.md)：用实际游戏任务评估正确性、开发耗时和人工接管情况。

维护者可在本地检查和打包：

```sh
python -B -m unittest discover -s tests -v
python scripts/build_package.py
```

真实 Bevy 示例另行运行（需 Rust 1.97.1 工具链）：

```sh
cd skills/playmand/examples/ecs-patterns
cargo test --locked
```

打包脚本使用明确的公开文件清单，生成 `dist/playmand-0.7.0.zip` 和 SHA-256 清单。GitHub Actions 在 Windows 与 Linux 上运行相同测试。宿主行为验证和引擎/GPU 验证的范围分别记录在 [VALIDATION.md](VALIDATION.md)，不以安装通过宣称任意游戏都能自动完成。

## 许可证

Playmand 采用 [MIT 许可证](LICENSE)。
