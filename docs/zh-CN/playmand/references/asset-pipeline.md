# 素材：获取、检查、接入

已验证的部分：[`scripts/check_asset.py`](../../../../skills/playmand/scripts/check_asset.py) 有针对 GLB、PNG、WAV 的单元测试，并实际跑过一个 450 万三角面的 Blender 导出模型、一条音轨和港口场景的美术图；[`scripts/render_glb.py`](../../../../skills/playmand/scripts/render_glb.py) 用 Blender 5.3 跑过同一个导出模型。引擎相关的说明来自 Bevy 0.20.1 的 glTF 扩展支持表和默认功能。本页其余内容是工作流程指引。

精美的角色、生物、场景和实拍素材来自外部：用户、艺术家、素材库，或用户运行的生成工具。然后由代码让它们动起来、打光、加特效、保证性能并发布。先问清作品类型和用户手上有什么（见[技能入口](../README.md#先问清用户要什么有什么)），本页讲的是接下来的步骤。

## 素材从哪里来

按这个顺序：

1. **用户自己的文件**：美术图、模型、实拍、产品图、logo、字体、音乐、配音。直接用；如果不是用户自己的作品，问清授权。
2. **授权清晰的免费素材库。** 一定要看素材本身页面上写的授权；网站整体的口碑不等于授权。
   - CC0（没有任何条件，可以商用）：Poly Haven（环境光照、贴图、模型）、ambientCG（PBR 贴图）、Kenney 和 Quaternius（游戏模型、精灵图、界面）。
   - 授权混杂，要逐个确认：Freesound（从 CC0 到禁止商用都有）、OpenGameArt、Sketchfab 上可下载的模型。
   - 字体：Google Fonts（大多是 SIL 开放字体许可）。
3. **用户运行或接入的生成工具**：生图与改图、图生 3D（把一张图或三视图变成带贴图的 GLB，有的还能自动绑骨骼）、音效、音乐、配音。用户自己用网页版，或用自己的 key 接入；第一次调用前先说明费用。商用前要看各服务的条款。
4. **代码**：特效、着色器、程序化贴图、合成的声音、动态图形、简单的风格化造型。不包括细节丰富的角色或场景（见[美术谁来做](art-direction.md#美术谁来做)）。

候选素材还不算素材。生成或下载的候选先放在临时文件夹里，只有用户挑中的才放进项目，其余删掉。下载由用户决定：先说明文件名、来源和大小。

## 维护素材清单

每个素材在 `assets/ASSETS.md`（或项目里对应的文件）里占一行，素材进来时就写，不要等到最后：

| 名称 | 文件 | 在游戏里的尺寸 | 来源 | 授权 | 作者 | 改动 |
| --- | --- | --- | --- | --- | --- | --- |
| hero | models/hero.glb | 1.8 米高 | 由 art/hero.png 图生 3D | 该服务的条款 | 生成 | 减面到 4 万，手部分离 |
| 雨声循环 | audio/rain.ogg | 24 秒循环 | 代码合成 | 项目授权 | | |
| 木箱 | models/crate.glb | 0.8 米 | Kenney，Survival Kit | CC0 | Kenney | 无 |

- **在游戏里的尺寸**（米、像素或秒）是代码缩放的依据；没有它，素材就会大小不对。
- 有条件的授权（CC BY 要署名，CC BY-SA 要以同样方式共享）需要把署名随作品一起发布，比如放在制作人员名单里或 `CREDITS.md`。禁止商用（NC）的素材不能用在商业作品里。
- key 和 token 不能写进清单，也不能进仓库。

## 每个素材用之前都要检查

能加载不等于能用。要量一量，再看一看：

```sh
python3 scripts/check_asset.py models/hero.glb --expect-height 1.8 --max-tris 60000 --max-texture 2048
python3 scripts/check_asset.py art/hero.png --require-alpha
python3 scripts/check_asset.py audio/hit.wav --max-peak -1
blender -b -P scripts/render_glb.py -- models/hero.glb hero-sheet.png
```

`check_asset.py` 只用 Python 标准库，报告文件里实际有什么；任何一项设定的上限不达标就返回 1，可以用来卡住构建：

- **模型（glTF/GLB）**：三角面数、顶点数、网格数、材质数、内嵌图片及其尺寸、骨骼与关节数、动画片段及其时长，并根据节点变换算出模型的实际尺寸（米）。以下情况会提出警告：模型看起来是厘米单位（超过 100 米高）；最低点不在原点；缺法线或贴图坐标；GLB 的图片放在文件外面；用了 Bevy 加载不了的扩展（Draco 和 meshopt 压缩、量化网格、basisu 和 WebP 贴图扩展），或需要开功能开关的扩展。
- **图片**：尺寸、有没有透明通道；PNG 还会统计真正透明的比例。有透明通道但每个像素都不透明，说明背景被烤进去了。
- **音频**：WAV 的时长、峰值和响度、削波的采样数、开头的静音长度；Ogg 的编码和时长。

`render_glb.py` 从正面、侧面、背面和四分之三角度把模型渲染到一张总览图上。和目标图并排看：比例、缺面或翻面、悬空的零件、朝向反了、材质导入后变成灰色。

尤其是图生 3D 的结果：预计会有几十万个三角面、整个模型是一个融合的网格、手指粘在一起、贴图里烤进了光照，有时还用了 Draco 压缩。先检查，再决定修什么。

## 在 Blender 里修整和准备

Blender 可以用脚本在后台运行（`blender -b -P fix.py -- in.glb out.glb`），所以修整步骤可以重复执行，并放进仓库：

- 缩放到米，把原点放在脚底或底座；静态道具可以应用变换，绑了骨骼的角色不要这样做。
- 把三角面减到项目的预算以内（应用 Decimate 修改器），减完再检查一次轮廓。
- 要由代码驱动动作的，在关节处把模型切开，把各部分挂到放在真实转轴上、以关节命名的空物体下（`Head`、`Shoulder.L`、`Elbow.L`……）。引擎转动这些空物体；刚性的机械不需要蒙皮。
- 需要独立运动的部件单独分开：能握拳的手、门、炮塔、轮子。
- 发光部件用自发光材质（发光很强时加 `KHR_materials_emissive_strength`），玻璃和贴花设置正确的透明模式。
- 给 Bevy 用的导出不要用 Draco 或 meshopt 压缩，导出后再检查一次。

需要完整骨骼和行走动作的人形角色，用用户运行的自动绑骨骼服务或工具；之后用 `check_asset.py` 检查关节数和动画片段。

## 接入引擎

Bevy 0.20.1：

- 加载场景：`commands.spawn(WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/hero.glb"))))`。0.20 里 glTF 场景是 `WorldAsset`；旧的教程和示例写的是 `SceneRoot`，这个类型已经不存在了。glTF 和 Bevy 一样是 Y 轴朝上、以米为单位；模型朝向反了，在根实体上加一个旋转即可，不需要重新导出。
- 用代码按名字驱动关节：场景生成后，按 `Name`（"Head"、"Elbow.L"）找到对应实体，每帧根据动作函数设置它们的 `Transform`。带关键帧的动画片段用 `AnimationGraph` 和 `AnimationPlayer` 播放。
- 格式的功能开关：默认构建能加载 PNG 和 HDR 图片，能播放 Ogg Vorbis。JPEG、WebP、KTX2 贴图，以及 WAV、MP3、FLAC 音频，都要开各自的功能（`jpeg`、`webp`、`ktx2`、`wav`、`mp3`、`flac`）。
- 自发光部件要在相机上开辉光才会发光；透明贴花要用 `AlphaMode::Blend`。

场景就绪、动画片段、动画图和混合的细节见[Bevy 资产与动画](assets-animation.md)。

用 Rust + WebAssembly 做网页：完整的 Bevy 构建有好几 MB。网页上优先用图片和特效（见[网页特效](web-wasm.md)）；在网页上放 3D 模型是一个单独的决定（用 Bevy 跑 WebGL2，或在自己的渲染器里写一个小的 glTF 加载器），要和体积一起权衡。

## 常见坑

- 在没人看过的模型上直接开工：结果大了 100 倍、朝向反了，或者手是一整坨。
- 网页工具导出的 GLB 用了 Draco 压缩：Bevy 加载不了，要重新导出。
- 素材没有记录授权，或者把禁止商用的素材用在了商业作品里。
- 生成的候选素材留在项目里，或者没经用户挑选就留了下来。
- 因为没向用户要素材，就用代码去雕一个细节丰富的角色。
