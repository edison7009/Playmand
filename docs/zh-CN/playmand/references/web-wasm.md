# 在网页或 Tauri 应用里用 Rust → WebAssembly 做特效

验证来源：本仓库 [`web/`](https://github.com/edison7009/Playmand/tree/main/web) 中的卡包、全息卡片和体素宠物（在线演示：<https://playmand.com/zh/>），最初用于 EchoBird 桌面应用。Rust 1.93、wasm-bindgen 0.2.127、web-sys 0.3.104、WebGL2；在 macOS 的 Chromium 系浏览器中检查过，Windows WebView2 与 Linux WebKitGTK 未检查。

当特效要放进现有网页或 React/Tauri 窗口时用这条路线：奖励动画、3D 小伙伴、产品展示。游戏和录制视频仍默认用 Bevy；这里整套引擎太重，而布局、文字和无障碍本来就归宿主页面管。

## crate 的结构

- `crate-type = ["cdylib", "rlib"]`。把**纯逻辑**（计时、弹簧、布局、模型、姿态、颜色计算）放在能原生编译的模块里，WebGL 部分用 `#[cfg(target_arch = "wasm32")]` 隔开。这样 `cargo test` 在任何机器上都能跑，CI 不需要浏览器。
- 每个场景一个 `#[wasm_bindgen]` 结构体，接口尽量小：`new(canvas)`、`resize(w, h, dpr)`、指针事件、`frame(now_ms) -> u32`、`dispose()`。所有导出方法放在**同一个** `#[wasm_bindgen] impl` 块里，辅助函数挪到普通 `impl`，否则方法会悄悄不再导出。
- `frame` 返回**事件位**（开始撕、卡片就位、翻面中点、完成……）。页面根据事件位反应，不靠计时器猜，任何帧率下声音和界面都同步。
- 必须在用户手势里完成的事（解锁音频）由指针调用同步返回，比如 `timing() -> [cue_ms, ready_ms]`，不要等之后某一帧。
- 没有运动时按需绘制；只在动画期间跑 `requestAnimationFrame`，页面隐藏或画布滚出视野就停。`dispose()` 里释放 GPU 对象，再调用 wasm-bindgen 的 `free()`。
- 用户要读的文字（名称、数值、标签）由页面传入，画到 2D canvas 再上传成纹理，翻译仍由宿主应用负责。

## 做出真实感

| 效果 | 做法 |
| --- | --- |
| 卡片厚度 | 沿圆角矩形在两个面之间生成一圈三角形侧边。法线在顶点着色器里用 `normalize(p.xy - clamp(p.xy, -(half - r), half - r))` 重建，不需要法线属性。 |
| 透明涂层 | 把视线在卡面上反射，在反射空间里查两条柔和的“影棚窗户”光带，卡片倾斜时会滑过卡面。再加指针处的光斑、细亮边、顶边高光和略暗的底边。用类似 screen 的混合，浅色卡面上也看得出。 |
| 纵深 | 每张卡背后一层软的圆角投影（先画、关闭深度写入）；图片窗口做成内凹（在 2D 纹理里画内阴影）；角色单独一层浮在卡面上方，带模糊的轮廓投影。 |
| 悬停 | 卡片倾斜，让指针下的那个角**往后压**（俯仰约 ±17°、偏航约 ±19°），用弹簧过渡；未悬停的卡轻轻漂动几度。翻转了 π 的面（卡背）俯仰符号相反。减少动态效果时倾斜减半、关闭漂动。 |
| 从开口抽出 | 在片元着色器里用一个穿过开口、随包装移动的世界空间平面裁剪卡片。如果卡片升起时直接画到包装前面，看起来就像从下面钻上来。 |
| 体素 | 剔除相邻面，加逐顶点环境光遮蔽（并按遮蔽翻转四边形对角线），标记真正的外棱边画细高光，尺寸很小时用 `fwidth` 淡出。 |
| 对齐 three.js | Lambert 光照要除以 π，颜色是线性的（十六进制颜色要过 sRGB 曲线），输出再做 sRGB 转换。欧拉角 XYZ 顺序是 `Rx·Ry·Rz`。 |

## 从 JavaScript 精确移植

在 Node 里把原实现的输出（每个体素、枢轴、生成的轨迹）导出成文本并计算哈希（FNV-1a 就够），在 Rust 测试里断言同样的哈希。刻意对齐 JavaScript：用 f64；`Math.round` 的 0.5 向上取整（`(x + 0.5).floor()`）；`Map` 保持插入顺序，重新插入的键会排到末尾。这里的体素宠物照此移植，9390 行第一次就完全一致。

## 用 Bevy 做美术

用 Bevy 网格建模角色，每个图层（身体、眼睛、表情）用正交相机分别在黑底和白底各渲染一次，从差值算出精确透明度。输出小体积的 WebP 图层，由浏览器叠加，眼睛可以眨、可以跟着看。

## 首屏

立即显示一个 CSS 加载状态；用 `modulepreload` 和 `preload as=fetch crossorigin` 预载模块与 `.wasm`；只等首帧需要的图片，其余在后台加载；资源到齐后同步画出第一帧，不等 `requestAnimationFrame`。动画循环只保留一个。

## 检查方法

- 加固定时钟的测试模式：页面停掉自己的循环，由脚本以 `t += 16.7` 调用 `frame(t)`，每次截图都可复现。嵌入式浏览器面板可能把 `requestAnimationFrame` 限到每秒几帧，不要用它判断节奏。
- 在 `frame` 之后同一个任务里用 `canvas.toDataURL()` 截取原始分辨率（不需要 `preserveDrawingBuffer`），发给一个本地小服务存成 PNG。面板截图太小，看不清边缘和高光。
- 与参考画面在同一时刻并排对比；每个变体（每种卡面、每个物种、每个阶段）和指针的两个极端位置都要看。

## 体积

发布构建用 `opt-level = "s"`、LTO、单个 codegen unit、`panic = "abort"`（如果不能改工作区 profile，就在构建脚本里用 `CARGO_PROFILE_RELEASE_*` 环境变量设置），再用 `wasm-bindgen --remove-name-section --remove-producers-section`。这个演示因此从 194 KB 降到 134 KB；线上卡包页面是 148 KB 的 WebAssembly 加 34 KB 的 JS，gzip 后合计 66 KB。体素宠物放在 Cargo feature 后面（`--features pets`），带上后为 235 KB（gzip 92 KB），而它们取代的 three.js 场景是 516 KB（gzip 133 KB）。可选场景都放进 feature，每个页面只带自己要画的东西。

## 踩过的坑

- 自上而下生成的网格是顺时针绕序：给它设 `front_face(CW)`，否则画出的是背面。
- `img.decode()` 在隐藏的面板里可能一直不返回，改等 `onload`。
- web-sys 0.3.104 没有 `CanvasRenderingContext2d::letterSpacing`，用 `js_sys::Reflect` 设置；`measure_text` 需要开启 `TextMetrics` feature。
- 以 ES2020 为目标的打包器不接受顶层 await：导出一个 `load()` promise，创建场景前先 await 它。
- 截图工具会触发 `pointerleave`，可能打断脚本拖动；改由测试模式用脚本驱动。
