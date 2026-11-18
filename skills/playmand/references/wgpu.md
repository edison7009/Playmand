# The GPU API: wgpu

Baseline: wgpu 30 (30.0.2), the version Bevy 0.20 renders with. Verified by the [wgpu example](../examples/gpu-wgpu/src/lib.rs): a compute shader and a render pass, both read back and checked, run in CI on software adapters (lavapipe on Linux, WARP on Windows) and locally on Metal.

Reach for wgpu directly when Bevy's materials are not enough: a custom render pass or post effect inside Bevy, GPU compute (particles, simulations, image processing), or rendering without an engine (a tool, a WebGPU page). In a Bevy project use the wgpu that Bevy re-exports (`bevy::render::render_resource`) rather than adding a second, different wgpu. For effects inside a web page that must run everywhere today, WebGL2 is still the safe target; see [web effects](web-wasm.md).

## Getting a device

```rust
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
```

`request_adapter` returns a `Result` in wgpu 30. On a machine or CI runner without a GPU, retry with `force_fallback_adapter: true` to get a software adapter; if there is still none, skip GPU tests loudly instead of failing in a way that looks like a code bug (the example skips unless `PLAYMAND_REQUIRE_GPU` is set, which CI sets).

## Reading results back

- Copy into a buffer created with `MAP_READ | COPY_DST`, `map_async`, then **`device.poll(wgpu::PollType::wait_indefinitely())`** before reading: nothing completes until the device is polled.
- `get_mapped_range` returns a `Result` in wgpu 30; `unmap()` before reusing the buffer.
- **Texture copies pad every row to 256 bytes** (`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`). A 3-pixel RGBA row is 12 bytes of pixels in 256 bytes of buffer: allocate `padded_row * height`, pass `bytes_per_row: Some(padded_row)`, and strip the padding when reading. This is the most common cause of skewed or garbled screenshots.

## Pipelines

- Compute: `create_compute_pipeline` with `layout: None` derives the bind group layout from the shader; get it back with `pipeline.get_bind_group_layout(0)`. Dispatch `len.div_ceil(workgroup_size)` groups and guard the tail in the shader (`if (id.x < arrayLength(&data))`).
- Render: a `RenderPassColorAttachment` in wgpu 30 has `depth_slice: None` alongside `view`, `resolve_target` and `ops`; `RenderPassDescriptor` implements `Default`, so fill the colour attachments and use `..Default::default()`.
- WGSL is the shader language everywhere (native, WebGPU); wgpu translates it for each backend.

## Pitfalls

- Mixing wgpu versions: Bevy 0.20 uses 30; a crate pulling wgpu 29 or 31 gives types that do not match.
- Forgetting the poll: `map_async` callbacks never fire and the test hangs.
- sRGB formats: `Rgba8UnormSrgb` stores colours with the sRGB curve, so 0.5 is not stored as 128. Use `Rgba8Unorm` when checking raw values.
