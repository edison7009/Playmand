# Rust 与 Bevy 工具链

## 建项目与锁版本

新项目采用 Rust + Bevy；保留用户指定的 2D/3D、平台和美术方向。先建立能启动目标渲染路径的工程，再按实际功能拆模块。一个 `Plugin` 可以组织输入、角色、战斗或 UI；小功能不需要抽成通用框架、脚本语言或自制引擎。

以下是本技能示例的版本基线，非永久“最新版”：

```toml
[package]
name = "my_game"
version = "0.1.0"
edition = "2024"
rust-version = "1.95"

[dependencies]
bevy = "=0.19.1"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

游戏提交 `Cargo.lock`。已有工程优先保留 features；新工程可先用默认功能跑通，再依据该版本 feature 表裁剪。照抄其他版本的 feature 名会让解析直接失败。依赖优化有助开发态运行，但首次构建更慢，不把第一次全编译计入热缓存迭代比较。

`cargo check --locked` 用于快速类型检查；`cargo run --locked` 进行开发态运行；分发和性能测量用 `cargo build --release --locked`。遵循项目已有构建约定。不要为了每次 UI 修改都验证 release 而无谓拉长迭代；也不要用未优化运行结果判断引擎性能。

## 从报错定位正确 API

运行 [inspect_bevy.py](../scripts/inspect_bevy.py)，或直接：

```sh
cargo metadata --format-version 1 --locked --offline --manifest-path /absolute/game/Cargo.toml
cargo tree --locked -d
cargo tree --locked -e features -i bevy
```

辅助脚本报告整个工作区解析图；交叉编译时增加 `--filter-platform <target-triple>`。多个版本不要只取列表第一项；用目标 package 的依赖链判断。`cargo tree -i` 存在歧义时使用 `bevy@版本`。插件依赖另一版 `bevy_ecs` 时，同名 `Component`/`Resource` trait 仍是不同类型。

在脚本输出的 `source_dir` 中用 `rg` 查符号；Bevy 顶层 crate 的 `examples/` 有对应版本示例，组件实现通常在 `bevy_ecs`、`bevy_ui`、`bevy_gltf` 等子 crate。先看函数签名、必要组件和插件注册，再改调用。E0432/E0599 常见于版本或 feature 不匹配；“System 不满足 trait”继续展开到具体参数，检查 Resource/Component derive、Query 项类型及借用。

0.19.1 的场景根与就绪信号是 `WorldAssetRoot` / `WorldInstanceReady`；旧教程常见 `SceneRoot` / `SceneInstanceReady`。不能只靠类型重命名完成迁移。[官方迁移指南](https://bevy.org/learn/migration-guides/) 应与版本源码一起看。

## 编译与平台陷阱

- Windows 的 Rust MSVC 工具链还需要兼容的 Visual C++ 链接器与 Windows SDK。先读链接错误；缺 `link.exe` 与 Rust 类型错误是两类问题。Cargo 不在 PATH 时检查 rustup 安装目录，而非重复安装整个环境。
- 在工作区根配置 profile，避免成员文件中的 profile 被忽略。沿用已有 target 目录；不同任务不要并发写同一构建目录造成锁等待。
- `bevy/dynamic_linking` 可作为开发选项评估，不能默认带入玩家包。它会改变运行时动态库依赖，直接搬走 exe 可能无法启动。
- GPU、音频、桌面窗口支持取决于执行环境。无窗口测试可以只启用所需 Bevy features；不要为让云端测试通过而从用户游戏删掉渲染或音频。
- Bevy 核心不等于已安装物理、导航或网络插件。先确认游戏是否需要，再查插件发布版支持的 Bevy 版本；不要仅凭 crate 名选择最新版。

出处：[Bevy 官方安装与开发配置](https://bevy.org/learn/quick-start/getting-started/setup/)、[Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)。技术基线核对日期：2026-09-13。
