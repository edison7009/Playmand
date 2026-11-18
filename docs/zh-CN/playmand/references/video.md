# 演示动画与视频录制

基线：Bevy 0.20.1。窗口截图录制已用仓库 `demos/` 中的鹈鹕骑车、ECS 星系与粒子图标三个场景在 macOS（Metal）实际录制验证，包括 1920×1080 / 30fps。离屏录制、音轨和带声音的 MP4 编码已由[视频示例](../../../../skills/playmand/examples/motion-video/src/bin/render.rs)在 16:9、9:16、1:1 三种画幅下验证。ffmpeg 编码与 Blender 渲染路径未在该次验证中执行。给人看的视频怎么规划、怎么配字幕和声音、怎么适配画幅，见[动态图形与视频制作](motion-graphics.md)。

用 Bevy 做视频，本质是把场景写成“时间 → 画面”的函数，再逐帧离线渲染。和游戏的区别只在时钟：游戏跟随真实时间与输入，录制跟随固定帧号。

## 离屏渲染（视频推荐用这个）

渲染到图像里而不是窗口，每帧读回一次，这一帧存盘之后时钟才前进。输出不会被任何窗口挡住，不需要显示器，而且是示例里最快的方式（Apple M4 上 270 帧 1920×1080 约 4 秒）。

```rust
use bevy::{app::ScheduleRunnerPlugin, camera::RenderTarget, prelude::*, window::ExitCondition, winit::WinitPlugin};
use bevy::render::{gpu_readback::{ReadbackComplete, ReadbackOnce}, render_resource::{TextureFormat, TextureUsages}};

App::new()
    .add_plugins(DefaultPlugins
        .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
        .disable::<WinitPlugin>())
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO));

// 初始化：相机画到一张允许 GPU 复制出来的图像上。
let mut image = Image::new_target_texture(w, h, TextureFormat::Rgba8UnormSrgb, None);
image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
let target = images.add(image);
commands.spawn((Camera2d, RenderTarget::Image(target.clone().into())));

// 每帧：场景摆好第 n 帧之后（同时只有一个读回在进行）：
commands.spawn(ReadbackOnce::texture(target.clone()))
    .observe(move |ev: On<ReadbackComplete>, mut rec: ResMut<Rec>| {
        let pixels = unpad_rows(&ev.data, w, h, 4);   // 见下
        save_png(&rec.dir, n, &pixels);
        rec.frame = n + 1;                             // 这时时钟才前进
        rec.in_flight = false;
    });
```

- **行填充**：读回的数据每一行都会补齐到 256 字节的整数倍。1920×4 = 7680 字节不需要补，但 1080×4 = 4320 会补到 4352，所以 9:16 和方形画面如果不从每行里只取 `width × 4` 字节，就会歪斜。
- **预热**：第一次截取前先渲染约 30 次更新，让管线、字体和字形图集准备好。
- 目标格式是 `Rgba8UnormSrgb` 时，读回的就是 sRGB 的 RGBA 字节，直接写成 PNG 即可。
- 用 CSS 字体族列表指定字体需要开启 Bevy 的 `system_font_discovery` feature（Linux 上还需要 fontconfig）；按句柄加载字体文件则不需要。

## 窗口截图，逐帧截图

需要真实窗口时（例如游戏自带的录制模式）用这种方式。


动画系统只读一个时钟资源，不直接读 `Time`：

```rust
#[derive(Resource, Default)]
struct Clock { t: f32 }

// 录制时：时钟等于下一张要截取的帧号
fn step_clock(rec: Res<Recording>, mut clock: ResMut<Clock>) {
    clock.t = rec.frame as f32 / rec.fps as f32;
}
```

交互预览时用 `time.elapsed_secs() % loop_seconds` 写同一个 `Clock`。这样同一套动画代码既能实时观看，也能逐帧导出，且每台机器录出的帧完全一致。

在 `Last` 中请求以帧号命名的截图，并且只在上一张截完后才推进帧号：

```rust
use bevy::render::view::screenshot::{Capturing, Screenshot, save_to_disk};

fn capture(
    mut commands: Commands,
    mut rec: ResMut<Recording>,
    pending: Query<(), Or<(With<Screenshot>, With<Capturing>)>>,
    mut exit: MessageWriter<AppExit>,
) {
    rec.update += 1;
    if rec.update <= rec.warmup || !pending.is_empty() {
        return; // 预热中，或上一张截图仍在回读
    }
    if rec.frame < rec.frames {
        let path = rec.dir.join(format!("frame-{:04}.png", rec.frame));
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        rec.frame += 1;
    } else {
        exit.write(AppExit::Success);
    }
}
```

要点：

- **预热**：前若干次 update 不截图（演示用 90 次），等管线编译、网格和材质就绪，避免开头出现空白帧。
- **等落盘再退出**：最后一帧请求后，查询 `With<Capturing>` 为空再写 `AppExit::Success`。截图是 GPU 回读加异步保存，请求返回不代表文件已写入。
- **精确像素**：窗口用 `WindowResolution::new(960, 540).with_scale_factor_override(1.0)`，否则 HiDPI 屏会按缩放因子输出 2 倍尺寸。
- **回读开销**：[渲染指南](rendering.md) 提醒游戏中不要每帧截图；离线录制可以每帧截，因为画面只由帧号决定，录制变慢不会改变结果。高分辨率或带辉光的场景只保留一个截图在途：查询 `Or<(With<Screenshot>, With<Capturing>)>` 为空时才请求下一帧并推进帧号。
- **高清输出**：保持场景逻辑尺寸不变，用 `WindowResolution::new(1920, 1080).with_scale_factor_override(2.0)` 让 960×540 的场景按 2 倍渲染，布局和文字位置不变。
- **窗口被遮挡会出黑帧（macOS）**：录制窗口被其他窗口盖住后不再呈现，之后的截图全黑。录制时设 `window_level: WindowLevel::AlwaysOnTop`，并在录完后统计全黑帧（例如 Pillow `convert("L").getbbox() is None`），有黑帧就重录。
- 录制期间会打开真实窗口并占用 GPU；无 GPU 的环境无法用此方法验证画面。

## 无缝循环

README 动图、社交媒体短片通常需要首尾无缝。让每个周期运动在一个循环时长 `L` 内走完整数个周期：

- 旋转：角度 `= 2π · k · t / L`，k 为整数（星系内外圈分别取 3、2、1 圈）。
- 滚动背景：图案周期 `P` 与速度满足 `speed · L = P`，位置取 `base - (speed · t) % P`。
- 车轮与路面联动：轮子转角 `= -speed · t / r`；若轮辐 8 等分，需使 `speed · L / r` 为 `π/4` 的整数倍。
- 随机分布用固定种子的确定性 RNG，保证每次运行的场景相同。

录制帧数为 `L · fps`，不包含第 `L · fps` 帧本身（它等于第 0 帧）。

## 编码

- **动图（README、网页）**：Pillow 可直接生成循环 WebP 或 GIF，无需 ffmpeg。WebP 对辉光、渐变的画质和体积明显优于 GIF；扁平色块画面 GIF 也可接受。720×405、25fps、4–6 秒，WebP 约 1–2.5 MB。
- **视频（MP4，macOS 无需 ffmpeg）**：技能自带的 [`scripts/encode_mp4.swift`](../../../../skills/playmand/scripts/encode_mp4.swift)（`swiftc -O` 编译）用 AVFoundation 把 PNG 帧编码为 H.264，`--audio` 可把 WAV 音轨编码为 AAC 一起写入，`--repeat` 重复循环以延长时长，`--verify` 校验成品文件；1080p30 用 12–16 Mbps 画质已足够。
- **视频（MP4，ffmpeg）**：有 ffmpeg 时，`ffmpeg -framerate 25 -i frame-%04d.png -c:v libx264 -pix_fmt yuv420p -crf 18 out.mp4`；要加声音时再加 `-i soundtrack.wav -c:a aac -shortest`。`yuv420p` 保证浏览器与社交平台可播放；宽高需为偶数。
- 先缩放再编码，用 LANCZOS 等高质量重采样；辉光场景降低质量参数前先比较暗部色带。

## 画面效果

- 2D 辉光：相机加 `Bloom`、`Tonemapping::TonyMcMapface`、`DebandDither::Enabled`，发光物体使用分量大于 1 的 HDR 颜色。依据同版本官方 `examples/2d/bloom_2d.rs`。
- 2D 拼装角色：用 `Rectangle`、`Circle`、`Ellipse`、`Triangle2d`、`Annulus`、`Capsule2d` 等基元组合；长条用单位宽度矩形加缩放和旋转摆放，两端补圆形让关节圆润。肢体跟随目标点时用两段式 IK 求膝/肘位置。
- 大量相同物体：共享 Mesh 与按颜色分桶的材质，便于批处理；每帧更新用 `par_iter_mut`。

## 验证

- 打开首帧、中间帧和末帧的实际图片检查构图、遮挡和空白帧，不以“录制命令成功”代替画面检查。
- 核对帧数、尺寸、帧率和循环标记（动图 `loop=0`）。
- 将末帧与首帧对比，确认循环处没有跳变。
- 在目标页面（如 GitHub README）实际预览动图是否自动播放、尺寸是否合适。

## Blender 渲染路径

需要电影级光照、材质或复杂角色动画时，可通过已连接的 Blender MCP 设置相机路径与关键帧并渲染序列或视频，流程与边界见 [Blender 与 MCP](blender-mcp.md)。实时、程序化、需要与游戏共用代码的效果优先在 Bevy 中完成。
