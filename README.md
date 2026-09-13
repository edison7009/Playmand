# Playmand

把 WithYou 中有依据的游戏开发经验，整理成 Codex、Claude Code 可读取的技能，让 AI 少重复摸索，并把改动推进到可试玩、可验证的结果。

当前是 **Skill 0.1.0**：一个共享技能、按需阅读的经验手册和安装脚本，采用 [MIT 许可证](LICENSE)。CLI 执行核心、MCP 和自动环境安装仍属于后续方向。

## 内容与范围

- [技能入口](skills/playmand/SKILL.md)：明确任务、发现已有工具、实施、运行、观察、保存和交付。
- [排错手册](skills/playmand/references/troubleshooting.md)：导航、草稿覆盖、焦点、布局、模型显示、资源释放和运行异常。
- [Bevy 经验](skills/playmand/references/bevy.md)：按实际版本查 API、调度/资产就绪和原生打包。
- [验证方法](skills/playmand/references/verification.md)：区分代码、输入、GPU、系统窗口与性能证据。
- [经验更新规则](skills/playmand/references/learning.md)：每次开发留下复现与证据，有依据地更新共享技能。

一般流程可用于其他引擎，但专项经验主要来自 Windows Rust/Bevy。它不会强制改用 Bevy，也不依赖 WithYou 仓库、私有素材或本机路径。它指导 AI 使用真实可用的工具，本身不会增加截图、输入控制或引擎操作接口。“100% 控制”仍是长期能力覆盖目标。

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

## 如何判断它有用

先做相同任务的 A（原始 AI）与 B（AI + Skill）对照；不以技能字数或规则数量衡量价值。[评估用例](docs/evaluation.md) 定义起点、通过标准和记录方式，[验证记录](VALIDATION.md) 区分本轮检查与尚未完成的实测。

维护经验时参考 [来源与边界](docs/provenance.md)。共享包包含归纳后的方法，不包含 WithYou 的代码、素材、个人配置或原始开发对话。

## 本地检查与打包

```sh
python -B -m unittest discover -s tests -v
python scripts/build_package.py
```

打包脚本使用明确的公开文件清单，生成 `dist/playmand-0.1.0.zip` 和 SHA-256 清单。GitHub Actions 在 Windows 与 Linux 上运行相同测试。宿主行为验证和引擎/GPU 验证的范围分别记录在 [VALIDATION.md](VALIDATION.md)，不以安装通过宣称任意游戏都能自动完成。
