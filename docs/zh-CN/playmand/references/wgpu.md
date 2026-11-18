# GPU 接口：wgpu

基线：wgpu 30（30.0.2），即 Bevy 0.20 渲染所用的版本。验证来源是 [wgpu 示例](../../../../skills/playmand/examples/gpu-wgpu/src/lib.rs)：一个计算着色器和一个渲染通道，结果都读回 CPU 检查。它在 CI 中运行于软件适配器（Linux 用 lavapipe，Windows 用 WARP），本机在 Metal 上运行。

Bevy 的材质不够用时直接使用 wgpu：Bevy 里的自定义渲染通道或后处理、GPU 计算（粒子、模拟、图像处理）、不用引擎的渲染（工具、WebGPU 页面）。在 Bevy 项目里使用 Bevy 重新导出的 wgpu（`bevy::render::render_resource`），不要再加一个不同版本的 wgpu。网页里要求今天就能到处运行的特效，WebGL2 仍是稳妥的目标，见[网页特效](web-wasm.md)。

## 获取设备

```rust
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
```

wgpu 30 中 `request_adapter` 返回 `Result`。在没有 GPU 的机器或 CI 上，用 `force_fallback_adapter: true` 重试以获得软件适配器；仍然没有时，明确地跳过 GPU 测试，而不是以看起来像代码错误的方式失败（示例在没有设置 `PLAYMAND_REQUIRE_GPU` 时跳过，CI 会设置它）。

## 读回结果

- 复制到一个用 `MAP_READ | COPY_DST` 创建的缓冲区，调用 `map_async`，读取前先 **`device.poll(wgpu::PollType::wait_indefinitely())`**：不轮询设备，什么都不会完成。
- wgpu 30 中 `get_mapped_range` 返回 `Result`；复用缓冲区前先 `unmap()`。
- **纹理复制时每一行都要补齐到 256 字节**（`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`）。3 个像素宽的 RGBA 行只有 12 字节像素，却占缓冲区 256 字节：分配 `补齐后的行长 * 高度`，传入 `bytes_per_row: Some(补齐后的行长)`，读取时去掉填充。这是截图错位或花屏最常见的原因。

## 管线

- 计算：`create_compute_pipeline` 中 `layout: None` 会根据着色器推导绑定组布局，用 `pipeline.get_bind_group_layout(0)` 取回。调度 `len.div_ceil(workgroup_size)` 个工作组，并在着色器里处理尾部越界（`if (id.x < arrayLength(&data))`）。
- 渲染：wgpu 30 的 `RenderPassColorAttachment` 除了 `view`、`resolve_target`、`ops` 还有 `depth_slice: None`；`RenderPassDescriptor` 实现了 `Default`，填好颜色附件后用 `..Default::default()`。
- WGSL 在所有平台（原生、WebGPU）都是着色器语言，wgpu 会为各后端转换。

## 常见坑

- 混用 wgpu 版本：Bevy 0.20 用 30，引入 wgpu 29 或 31 的 crate 会带来对不上的类型。
- 忘了轮询：`map_async` 的回调永远不触发，测试卡住。
- sRGB 格式：`Rgba8UnormSrgb` 按 sRGB 曲线存储颜色，0.5 不会存成 128。检查原始数值时用 `Rgba8Unorm`。
