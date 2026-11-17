# Playmand 宣传片

一支 30 秒、1920×1080 的宣传片。画面是 Bevy 场景，按 [Playmand 视频录制说明](../skills/playmand/references/video.md) 用固定时钟逐帧截图，再交给 ffmpeg。不是外部生成的视频，也不是网页动画。

## 片子在说什么

从一句想法，到写在 Rust 里的场景，再到同一套代码既能实时游玩、也能逐帧录成视频。结尾是 Playmand 和 `github.com/edison7009/Playmand`。

文案没有写速度倍数、用户数，也没有把主观定位写成测试结论。「创作王炸 Skill」是这支片子的说法，不是评测结果。

## 重现

依赖：

- Rust 1.97.1（见 `rust-toolchain.toml`）
- Bevy 0.20.1（`Cargo.lock` 已锁定）
- 一个能开窗口的显示环境。无 GPU 时可用 Mesa lavapipe
- ffmpeg
- 重新生成音乐需要 Python 3、numpy、scipy
- 重新子集字体需要系统里的 Noto Sans CJK（Debian 包 `fonts-noto-cjk`）和 fontTools

```sh
cd promo
./render.sh frames          # 约 900 张 PNG。无 GPU 时脚本会改用 lavapipe
./encode.sh frames playmand-promo.mp4
```

只看实时预览、不落盘：

```sh
cargo run
```

抽查时间轴可以降低帧率，画面仍由时间决定，不由机器快慢决定：

```sh
cargo run -- --record frames --fps 2 --seconds 30
```

音乐是仓库里的 `assets/music/promo.wav`。要重新合成同一份：

```sh
python3 tools/compose_music.py assets/music/promo.wav
```

种子固定，输出应一致。曲是为本片写的，没有采样，也没有旁白。

## 这次实际走的技能步骤

1. 读 `skills/playmand/SKILL.md`，以及 `references/video.md`、`rendering.md`、`bevy.md`、`verification.md`。
2. 新工程用 Rust + Bevy 0.20.1，不改技能正文和 `demos/`。
3. 动画只读 `Clock`。`--record` 时每个 update 推进一帧，预热 120 次 update 后再截图，窗口 `scale_factor_override(1.0)`，分辨率 1920×1080。
4. 相机使用 `Bloom`（只让 HDR 颜色发光）、`Tonemapping::TonyMcMapface`、`DebandDither::Enabled`。
5. 粒子是共享网格的 ECS 实体，确定性 RNG，录出来的帧不随机器变化。
6. 中文用子集后的 Playmand Sans（来自 Noto Sans CJK SC，OFL）。标识用仓库 `assets/logo.svg` 栅格化的 PNG，不是另造的官方素材。
7. `ffmpeg -pix_fmt yuv420p` 编码，并混入上面的音乐。
8. 打开首帧、中段和结尾的实际 PNG 检查中文和构图。

Blender MCP 没有用。这支片子是程序化实时画面，技能说明里这类效果优先在 Bevy 里做。

## 已在本环境验证

- 适配器：llvmpipe（Mesa Vulkan 软件渲染）。没有独立 GPU。
- 输出帧：1920×1080 RGB PNG。
- 中文标题、代码卡片、游戏/视频分屏、结尾 URL 都读过实际帧。
- 无声卡。Bevy 会提示找不到音频设备；成片的声音是编码时混入的，不依赖 Bevy 音频。
- 日志里有 `No segmentation model for complex script: Chinese/Japanese`。抽帧看到中文仍然成形，这个警告没有变成缺字。

## 限制

- 软件渲染能出片，但是慢。有 GPU 时不要强制 `VK_ICD_FILENAMES`。
- 成片是线性的 30 秒，不是 README 那种首尾无缝循环。
- 片里的「游戏」是轨道上的程序化角色，用来说明同一套场景可以游玩，不是一个完整游戏。
- 字体子集只覆盖本片用到的字符。改文案后如果缺字，重新运行 `python3 tools/subset_fonts.py`。
