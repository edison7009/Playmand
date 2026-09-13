# Playmand 0.2.0 验证

日期：2026-09-13。新版重点为 Rust + Bevy 专项指导、版本定位脚本与可执行 ECS 用法。

## 已执行的本地检查

| 检查 | 结果与范围 | 复现入口 |
| --- | --- | --- |
| Skill 格式 | 命名、frontmatter、占位符检查通过 | skill-creator 的 `quick_validate.py skills/playmand` |
| Python 回归 | 13 项通过：双宿主安装、冲突保护、中文路径/旧编码、包完整性、版本图解析、构建缓存排除 | `python -B -m unittest discover -s tests -v` |
| Rust / Bevy 真实执行 | Windows MSVC、Rust 1.95.0、Bevy 0.19.1，4 项 ECS 测试通过 | 在 `skills/playmand/examples/ecs-patterns` 执行 `cargo test --locked` |
| 版本定位脚本 | 对真实示例执行成功，报告 Bevy 0.19.1、启用 features、源码目录及对应版本文档 | `python -B skills/playmand/scripts/inspect_bevy.py --manifest-path skills/playmand/examples/ecs-patterns/Cargo.toml --filter-platform x86_64-pc-windows-msvc` |
| 分发内容 | ZIP 校验、SHA-256、包内相对链接与解压后双宿主安装通过；Cargo target / Python 缓存不进入安装和分发 | 包测试及 `python scripts/build_package.py` |

Python 版本图测试覆盖传递依赖、同名不同版本、同版本不同来源、无解析图的错误输入。脚本以 Cargo 元数据为依据，匹配 bevy / bevy_ 名称，其中可能包含第三方 crate；报告整个工作区，不冒充单个游戏二进制的精确依赖列表。path/git 修改版应优先看报告中的源码，发布版文档可能与其不同。

四项 Bevy 测试检查系统执行后的 World：

- 同时具有 Player / Enemy 标记的实体仍按明确互斥规则更新，不发生查询别名冲突。
- 链式后续系统在同一次 Update 看到 Commands 创建的实体。
- 同值写入不重复触发下游变化处理，真正改变值会触发。
- 连续三次进入/退出游玩状态，关卡实体每次创建一个、退出归零。

示例使用真实 Bevy、精简 features 和无窗口 App，未启动 GPU。GLB、动画、UI 与截图指南的 API 已对照 0.19.1 官方 crate 源码/示例，尚未在本版独立图形工程中逐项运行。

CI 配置覆盖 Windows / Linux 的 Python 3.9 / 3.13，以及两平台的 Rust 1.95.0 ECS 示例；远程实际结果以对应提交的 Actions 为准。本地 Windows 通过不替代 Linux 实测。

## 历史宿主测试与尚未证明的收益

[0.1.0 宿主记录](docs/host-validation.md) 保留此前 Codex / Claude Code 在 Python 小样中的技能发现与执行结果。它们使用旧正文，不是 0.2.0 的行为验证，也不能证明 Rust + Bevy 游戏开发能力。

本次尚未完成：新正文的跨模型宿主行为复测、GPU 画面/动画/操作手感与完整玩家包验收，以及有无技能的同条件 A/B 实验。没有“开发质量一定更高”“提效比例”或“100% 全自动”的结论。后续用 [Bevy 评估任务](docs/evaluation.md) 衡量实际游戏结果。

安装器保持不覆盖差异内容；磁盘或权限 I/O 故障仍可能留下部分复制内容，不具有跨目录事务保证。原始规划、机器日志、宿主轨迹及构建产物保留本地，不进入公开包。
