# Playmand

Playmand 是面向 **Codex 和 Claude Code** 的独立游戏开发 Skill，帮助 AI 完成需求拆解、代码实现、运行试玩、问题修复与验证交付。

**目标是让 AI 更高效地把游戏想法变成可试玩、可验证、可持续迭代的游戏，减少重复探索、无效重试和人工接管。**

用户提出创作方向和反馈，AI 使用项目现有工具推进开发，并通过实际操作、画面与运行状态检查结果。Playmand 将这套工作方式整理成可复用的指令和按需阅读的手册。

## 帮助 AI 做什么

- [开发流程](skills/playmand/SKILL.md)：明确需求和验收标准，先打通最小可玩流程，再逐步扩展功能。
- [排错手册](skills/playmand/references/troubleshooting.md)：导航、草稿覆盖、焦点、布局、模型显示、资源释放和运行异常。
- [Bevy 经验](skills/playmand/references/bevy.md)：按实际版本查 API、调度/资产就绪和原生打包。
- [验证方法](skills/playmand/references/verification.md)：区分代码、输入、GPU、系统窗口与性能证据。
- [持续积累](skills/playmand/references/learning.md)：记录有效的启动、复现和修复方法，让后续开发复用已验证的做法。

适用于新建游戏、迭代玩法、调整 UI、接入模型与动画、排查性能和准备交付。当前专项指南侧重 Windows Rust/Bevy；已有项目沿用自己的技术栈。

## 安装

要求已安装 Codex 或 Claude Code，以及 Python 3.9+。安装器仅使用 Python 标准库；游戏开发所需的引擎、编译器、图形与输入工具按目标项目准备。它不会自动安装或购买这些工具。

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

目录规则参照 [Codex 官方 Skill 文档](https://learn.chatgpt.com/docs/build-skills) 与 [Claude Code 官方 Skill 文档](https://code.claude.com/docs/en/skills)，2026-09-13 核对。共享正文只使用通用 `name` / `description` 和相对路径，不依赖宿主专有指令语法；`agents/openai.yaml` 是 Codex 附加显示信息。

安装后打开游戏项目的新会话，检查技能是否被发现，再显式调用。更旧或受组织策略限制的宿主需按其实际版本确认加载行为；文件复制成功不等于宿主已加载。这里指本地 Codex / Claude Code，云端任务还需将技能带入其实际执行环境。

安装器遇到相同内容会跳过，遇到同名但不同内容的目录会退出且不覆盖。更新时先 `git pull`，检查新旧差异并把旧安装目录移到技能发现目录之外留存，再重新安装；安装副本不自动同步。源文件统一维护在本仓库 `skills/playmand`。

## 从一个小任务开始

Codex 示例：

```text
$playmand 在当前游戏加入一个可收集道具及计数 UI。
沿用已有引擎和美术风格。通过玩家输入完成收集，计数只增加一次，重置后恢复。
请先说明假设与验收方式，完成实现、实际运行和必要验证，并记录可复用的新经验。
```

Claude Code 将首行开头换成 `/playmand` 即可。也可直接请求修复现有游戏问题，技能允许按相关描述自动匹配，实际是否触发以宿主为准。

## 验证与维护

- [验证记录](VALIDATION.md)：已执行的安装、宿主与分发检查，以及当前验证范围。
- [评估用例](docs/evaluation.md)：用实际游戏任务评估正确性、开发耗时和人工接管情况。

维护者可在本地检查和打包：

```sh
python -B -m unittest discover -s tests -v
python scripts/build_package.py
```

打包脚本使用明确的公开文件清单，生成 `dist/playmand-0.1.0.zip` 和 SHA-256 清单。GitHub Actions 在 Windows 与 Linux 上运行相同测试。宿主行为验证和引擎/GPU 验证的范围分别记录在 [VALIDATION.md](VALIDATION.md)，不以安装通过宣称任意游戏都能自动完成。

## 许可证

Playmand 采用 [MIT 许可证](LICENSE)。
