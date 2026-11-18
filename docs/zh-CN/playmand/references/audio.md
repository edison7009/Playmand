# 用 Kira 做音频

基线：Kira 0.12.6；在 Bevy 0.20 中通过 bevy_kira_audio 0.27.1 使用。验证来源是[音频示例](../../../../skills/playmand/examples/audio-kira/src/lib.rs)，它在 CI 中于 Windows 和 Linux 上无声卡运行 Kira。bevy_kira_audio 插件本身没有在那里测试（它需要音频设备），下面的接入方式来自 0.27.1 的源码和 README。

游戏需要的不只是“播一个声音”时用 Kira：在曲目间淡入淡出的音乐、各有音量的混音轨（音乐、音效、语音）、菜单打开时全部暂停、滤波和混响。

## 在 Bevy 中

```toml
[dependencies]
bevy = { version = "=0.20.1", default-features = false, features = ["2d", "3d", "ui"] }
bevy_kira_audio = "=0.27.1"  # 默认支持 ogg；按需加 "mp3"、"wav"、"flac"
```

**关掉 Bevy 自带的 `audio` feature**：它在 Bevy 默认 feature 里，与 bevy_kira_audio 冲突。去掉 `audio` 后的 Bevy 默认 feature 是 `2d`、`3d`、`ui`。

```rust
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

App::new().add_plugins((DefaultPlugins, AudioPlugin));

fn start_music(assets: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(assets.load("music/theme.ogg"))
        .looped()
        .fade_in(AudioTween::new(Duration::from_secs(2), AudioEasing::OutPowi(2)))
        .with_volume(-6.0); // 分贝
}
```

把声音分到不同声道（用 `add_audio_channel::<T>()` 注册 `AudioChannel<Music>`、`AudioChannel<Effects>`），设置里的滑杆只改一个声道，暂停菜单只停另一个。

## Kira 怎么混音

- 音量用**分贝**：0 dB 不变，−6 dB 约为一半振幅，`Decibels::SILENCE`（−60 dB）为静音。滑杆按分贝变化听起来才自然，线性的 0–1 不行。
- 每个变化都可以带一个 `Tween`（时长和缓动）：淡入淡出、交叉淡化、压低背景音乐都用 tween，不用每帧手写。
- 声音在音轨上播放；音轨汇入父音轨，最终汇入主音轨。**音轨先跑自己的效果器，再乘自己的音量**，所以挂在同一条音轨上的电平表、压限器或可视化看到的是推子之前的信号。要放到父音轨上（示例是在主音轨上测电平）。
- 单次音效播完会自动停止（`PlaybackState::Stopped`）；暂停的声音保持位置，恢复后从原处继续。

## 不用扬声器也能检查

`AudioManager::<MockBackend>` 配合 `MockBackendSettings { sample_rate }` 可以在没有声卡的情况下运行整个混音器：调用 `backend_mut().on_start_processing()` 和 `backend_mut().process()` 推进一个块（`internal_buffer_size` 帧）。用 `StaticSoundData { frames, .. }` 生成测试音，不用加载文件；挂一个记录电平的小 `Effect`，断言玩家会听到的结果：电平、淡入淡出、静音、播放位置。真实的混音效果仍要在游戏里用耳朵听。

## 常见坑

- 没声音也没报错：Bevy 的 `audio` feature 还开着；资源格式对应的 feature（`mp3`、`wav`、`flac`）没开；或者浏览器要等用户操作后才允许播放声音（网页构建）。
- 用线性数值设音量：在 Kira 里 `0.5` 不是一半音量，要用分贝。
- 每次进入某个状态都重新开始播放音乐：保留句柄，先检查是否已经在播。
