# Playmand

**Playmand 是 Rust + Bevy 游戏开发 Skill，目标是让 AI 更熟练地使用这套技术栈，把游戏设计实现为可玩的 2D / 3D 游戏。**

核心是 Rust + Bevy 的具体开发能力：正确使用当前版本的 API、组织 ECS 与玩法状态、接入模型动画、实现 UI 和渲染、定位性能问题并构建游戏。Codex、Claude Code 及其他支持技能的工具是使用它的宿主。

**Blender + MCP 是资产制作的重要组成部分**：AI 通过已连接的 MCP 操作 Blender，制作和修改模型、材质、骨骼与动画，检查结果后导出 GLB 并接入 Bevy。技能覆盖从可编辑资产到游戏内呈现的技术衔接。

新建游戏默认 Rust + Bevy；已有 Bevy 工程保留其版本和结构。用户明确选用其他引擎时尊重选择，不自动迁移已有项目。技能不规定固定开发循环，也不把用户要求缩减成最小演示。

## 为什么是 Rust + Bevy + Agent

Rust 的吸引力首先在于运行效率和可靠性：编译为原生代码、无需垃圾回收，并能精细控制内存与资源；类型系统和所有权检查可以在编译阶段发现许多内存与并发错误。这些特性适合重视帧时间、资源占用和长期维护的游戏项目。[Rust 官方介绍](https://rust-lang.org/)

Bevy 的 ECS 架构围绕数据组织游戏逻辑，支持在条件允许时并行执行系统，模块化设计也便于按需求扩展。玩法、状态和系统行为可以通过 Rust 代码表达，方便 Agent 阅读、修改，并结合 Cargo 编译和测试获得反馈。[Bevy 官方介绍](https://bevy.org/learn/quick-start/introduction/)

代价是代码门槛较高：所有权、借用、生命周期，以及 Bevy 的查询、调度和资源管理，都需要开发者理解。构建耗时、版本变化和复杂错误排查也会增加迭代成本。运行效率高，并不意味着开发过程天然更快。

**这正是 Playmand 与 Agent 结合的核心：发挥 Rust + Bevy 的性能与工程优势，让 Agent 承担更多代码实现、API 查证和问题定位，降低驾驭这套技术栈的成本。** 用户把精力放在玩法、风格和体验判断上，Playmand 为 Agent 补充引擎与资产制作知识，帮助它少猜接口、少踩技术陷阱。实际开发效率与游戏质量，仍通过具体项目和对照评估检验。

## 提供什么

| 内容 | 具体帮助 |
| --- | --- |
| [Rust / Bevy 工具链](skills/playmand/references/bevy.md) | 新工程配置、版本/API 定位、feature 与插件兼容、编译及 Windows 链接 |
| [ECS 与玩法状态](skills/playmand/references/ecs.md) | Query 借用冲突、延迟命令、系统顺序、固定时间步输入、暂停与重开 |
| [Blender 与 MCP](skills/playmand/references/blender-mcp.md) | 连接与能力发现、场景编辑、模型/材质/动画制作、GLB 导出及 Bevy 接入 |
| [资源与动画](skills/playmand/references/assets-animation.md) | GLB 子场景、实例就绪、动画图/播放器接线、材质与骨骼实例隔离 |
| [渲染、UI 与性能](skills/playmand/references/rendering.md) | 2D/3D 相机、灯光、图层、UI 布局、截图与预览相机生命周期 |
| [版本检查脚本](skills/playmand/scripts/inspect_bevy.py) | 从 Cargo 实际解析图报告 Bevy 版本、features、源码位置和文档入口 |
| [可运行 Rust 示例](skills/playmand/examples/ecs-patterns/src/lib.rs) | 四项真实 Bevy ECS 行为测试，可用于理解 API 或缩小故障 |

技术示例基线为 **Bevy 0.19.1 / Rust 1.95.0**。其他版本需查对应源码，已有项目不应为套用示例而升级。辅助资料按问题读取，完整入口见 [SKILL.md](skills/playmand/SKILL.md)。

## 安装

要求已安装 Codex 或 Claude Code，以及 Python 3.9+。安装器仅使用 Python 标准库；游戏开发所需的引擎、编译器、图形与输入工具按目标项目准备。它不会自动安装或购买这些工具。

使用 Blender MCP 时，还需在实际宿主配置相应 MCP 服务并启用 Blender 端插件；技能安装不会自动完成连接。具体实现与接入边界见 [Blender MCP 指南](skills/playmand/references/blender-mcp.md)。

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

真实 Bevy 示例另行运行（需 Rust 1.95.0 工具链）：

```sh
cd skills/playmand/examples/ecs-patterns
cargo test --locked
```

打包脚本使用明确的公开文件清单，生成 `dist/playmand-0.2.1.zip` 和 SHA-256 清单。GitHub Actions 在 Windows 与 Linux 上运行相同测试。宿主行为验证和引擎/GPU 验证的范围分别记录在 [VALIDATION.md](VALIDATION.md)，不以安装通过宣称任意游戏都能自动完成。

## 许可证

Playmand 采用 [MIT 许可证](LICENSE)。
