---
name: playmand
license: MIT
description: 用 Rust + Bevy 开发 2D/3D 游戏，提供版本匹配、ECS 与系统调度、输入与状态、GLB 动画、UI 渲染、性能和原生构建的专项知识。用于新建 Bevy 游戏或实现、修复现有 Rust/Bevy 项目；显式使用本技能新建游戏且未指定引擎时默认 Rust + Bevy。
---

# Playmand

Rust + Bevy 游戏开发技能。用引擎知识和可验证的代码帮助实现用户的游戏设计。Codex、Claude Code 等只是宿主；正文与工具不依赖某个模型、MCP 或编辑器。

新建游戏默认 Rust + Bevy，按用户要求完成玩法、画面和内容。已有 Bevy 项目沿用锁定版本及结构；用户明确选择其他引擎时尊重选择，已有其他引擎项目不自动迁移。小样用于验证技术疑点，不替代用户要求的完整游戏，也不规定固定开发轮次或汇报模板。

## 先解决版本问题

读取游戏的 `Cargo.toml`、`Cargo.lock` 与 `rust-toolchain.toml`。API、第三方插件和 feature 以实际解析版本为准，不能把网上旧版示例混入当前工程。新项目先确认目标平台，选定可用稳定版本并生成锁文件。

需要定位 crate 源码和启用的 feature 时，可运行随技能安装的辅助脚本（路径相对技能目录）：

```sh
python scripts/inspect_bevy.py --manifest-path /absolute/game/Cargo.toml
```

它调用离线、锁定的 `cargo metadata`，输出工作区解析到的 Bevy crate、源码目录和版本文档链接。缺少依赖时给出 Cargo 原始错误；按项目需要获取依赖后再运行。[工具链指南](references/bevy.md) 包含构建配置和 API 查找方法。

## 按当前技术问题读取

| 工作 | 专项资料 |
| --- | --- |
| 新建工程、版本/API 报错、编译慢、Windows 链接、插件兼容 | [Rust 与 Bevy 工具链](references/bevy.md) |
| 组件/资源设计、Query 冲突、Commands 可见性、帧时序、输入、暂停/重开 | [ECS、调度与玩法状态](references/ecs.md) |
| GLB、场景实例、动画、骨骼、共享资产与实例隔离 | [资源与动画](references/assets-animation.md) |
| 2D/3D 相机、灯光、UI、黑屏、预览、多 pass、帧率 | [渲染、UI 与性能](references/rendering.md) |
| 选择逻辑/画面检查、玩家包资源路径 | [Bevy 验证与分发](references/verification.md) |
| 跨页草稿、焦点、复杂布局、模型显示异常的补充诊断 | [症状索引](references/troubleshooting.md) |

只读取相关资料。技术示例基线为 **Bevy 0.19.1 / Rust 1.95.0**，不代表应升级现有项目。不同版本先查该版本源码与迁移指南。

## 可执行的 ECS 用法

[独立示例](examples/ecs-patterns/src/lib.rs) 用真实 Bevy 验证：查询互斥、延迟命令对后续系统可见、避免无效 change detection、状态退出清理。用于学习或缩小对应故障，不是强制套用的游戏模板。

```sh
cd examples/ecs-patterns
cargo test --locked
```

命令从技能目录执行；首次需要获取依赖及满足示例 Rust 版本。示例不启动窗口，不能证明 GPU 画面、操作手感或游戏完成度。游戏任务仍使用项目的真实运行入口验证相应结果。
