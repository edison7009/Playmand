//! Headless wgpu recipes: a compute shader and a render pass, both read back
//! to the CPU and checked. wgpu 30 is what Bevy 0.20 renders with, and the
//! same API sits under custom Bevy render passes and WebGPU pages.
//!
//! Without a GPU adapter the tests skip with a note, unless the environment
//! sets PLAYMAND_REQUIRE_GPU (CI does, after installing a software Vulkan
//! driver on Linux; Windows has the WARP software adapter built in).

use std::borrow::Cow;

/// A device and queue from any adapter, preferring a real GPU and falling
/// back to a software one. `None` when the machine has neither.
pub fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = [false, true].into_iter().find_map(|fallback| {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            force_fallback_adapter: fallback,
            ..Default::default()
        }))
        .ok()
    });
    let Some(adapter) = adapter else {
        assert!(
            std::env::var_os("PLAYMAND_REQUIRE_GPU").is_none(),
            "PLAYMAND_REQUIRE_GPU is set but no GPU adapter was found"
        );
        eprintln!("no GPU adapter on this machine: skipping");
        return None;
    };
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

/// Copies `buffer` (MAP_READ) back to the CPU and waits for it.
fn read(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<u8> {
    buffer.map_async(wgpu::MapMode::Read, .., |result| {
        result.expect("map the readback buffer")
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("wait for the GPU");
    let bytes = buffer.get_mapped_range(..).expect("mapped range").to_vec();
    buffer.unmap();
    bytes
}

/// Doubles every number on the GPU and returns the result.
pub fn double_on_gpu(device: &wgpu::Device, queue: &wgpu::Queue, input: &[f32]) -> Vec<f32> {
    use wgpu::util::DeviceExt;
    const SHADER: &str = "
        @group(0) @binding(0) var<storage, read_write> data: array<f32>;
        @compute @workgroup_size(64)
        fn main(@builtin(global_invocation_id) id: vec3<u32>) {
            if (id.x < arrayLength(&data)) { data[id.x] = data[id.x] * 2.0; }
        }";
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("double"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("double"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let size = std::mem::size_of_val(input) as u64;
    let data = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("data"),
        contents: bytemuck::cast_slice(input),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: data.as_entire_binding(),
        }],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((input.len() as u32).div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&data, 0, &readback, 0, size);
    queue.submit([encoder.finish()]);
    bytemuck::cast_slice(&read(device, &readback)).to_vec()
}

/// Clears a `width` x `height` RGBA texture to `color` and reads its pixels
/// back, one tight row of `width * 4` bytes after another.
pub fn clear_and_read(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    color: wgpu::Color,
) -> Vec<u8> {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    // A texture copy must start every row on a 256-byte boundary.
    let tight = width * 4;
    let padded =
        tight.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pixels"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let bytes = read(device, &readback);
    // Drop the padding at the end of each row.
    bytes
        .chunks(padded as usize)
        .flat_map(|row| row[..tight as usize].to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_compute_shader_doubles_every_number() {
        let Some((device, queue)) = device() else {
            return;
        };
        let input: Vec<f32> = (0..1000).map(|i| i as f32 * 0.5).collect();
        let output = double_on_gpu(&device, &queue, &input);
        let expected: Vec<f32> = input.iter().map(|v| v * 2.0).collect();
        assert_eq!(output, expected);
    }

    #[test]
    fn a_cleared_texture_reads_back_with_row_padding_removed() {
        let Some((device, queue)) = device() else {
            return;
        };
        // 3 pixels wide: 12 bytes a row, padded to 256 for the copy.
        let color = wgpu::Color {
            r: 1.0,
            g: 0.5,
            b: 0.0,
            a: 1.0,
        };
        let pixels = clear_and_read(&device, &queue, 3, 2, color);
        assert_eq!(pixels.len(), 3 * 2 * 4);
        for px in pixels.chunks(4) {
            assert_eq!(px[0], 255);
            assert!((127..=128).contains(&px[1]), "green {}", px[1]);
            assert_eq!((px[2], px[3]), (0, 255));
        }
    }
}
