//! The one-pixel grid must be antialiased (user report 2026-10-19: "the grid is too jagged").
//!
//! A GPU line is always one device pixel wide. With a single sample a diagonal segment fills whole
//! pixels, which is the visible staircase in the viewport. The document viewport draws into a 4x
//! multisampled pair and resolves it into the texture egui paints; this test proves that the overlay
//! pipeline and the resolve path turn the partial coverage of a diagonal line into intermediate grey
//! levels, and that the single-sample path does not.

use om9_render::overlay::OverlayBatch;
use om9_render::{Camera, Projection};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()?;
    let flags = adapter.get_texture_format_features(wgpu::TextureFormat::Rgba8Unorm).flags;
    if !flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
        eprintln!("the adapter has no 4x multisampling; the manual viewport check covers this machine");
        return None;
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("antialiasing"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .ok()?;
    Some((device, queue))
}

fn texture(device: &wgpu::Device, label: &str, format: wgpu::TextureFormat, samples: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: if format == wgpu::TextureFormat::Rgba8Unorm {
            // The resolved texture is the one read back, so it needs COPY_SRC as well.
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
        } else {
            wgpu::TextureUsages::RENDER_ATTACHMENT
        },
        view_formats: &[],
    })
}

/// Draws one 45-degree line through a cleared frame and returns the resolved pixels.
fn render_line(samples: u32) -> Option<Vec<u8>> {
    let (device, queue) = gpu()?;
    let resolve = texture(&device, "AA resolve", wgpu::TextureFormat::Rgba8Unorm, 1);
    let resolve_view = resolve.create_view(&wgpu::TextureViewDescriptor::default());
    let colour = texture(&device, "AA colour", wgpu::TextureFormat::Rgba8Unorm, samples);
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth = texture(&device, "AA depth", wgpu::TextureFormat::Depth32Float, samples);
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());

    // A render pass cannot mix sample counts, so clear the colour and depth attachments the same way.
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("AA clear") });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("AA clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &colour_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    queue.submit(std::iter::once(encoder.finish()));

    let camera = Camera {
        eye: [0.0, 0.0, 10.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        projection: Projection::Orthographic { vertical_span: 10.0 },
        near: 0.1,
        far: 100.0,
    };
    let mut overlay = OverlayBatch::with_samples(device.clone(), queue.clone(), samples);
    overlay.line([-4.0, -4.0, 0.0], [4.0, 4.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
    overlay.upload();
    // A single-sample attachment cannot have a resolve target; it is read back directly.
    let readback_texture = if samples > 1 {
        overlay.render_resolved(&colour_view, &resolve_view, &depth_view, camera, WIDTH, HEIGHT);
        &resolve
    } else {
        overlay.render(&colour_view, &depth_view, camera, WIDTH, HEIGHT);
        &colour
    };

    // Read the resolve target back into tightly packed RGBA8 rows.
    let row_bytes = WIDTH * 4;
    let padded_row = row_bytes.div_ceil(256) * 256;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("AA readback"),
        size: u64::from(padded_row) * u64::from(HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("AA readback") });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: readback_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(encoder.finish()));
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("map");
    let data = slice.get_mapped_range().expect("mapped range");
    let mut pixels = Vec::with_capacity((row_bytes * HEIGHT) as usize);
    for row in 0..HEIGHT {
        let start = (row * padded_row) as usize;
        pixels.extend_from_slice(&data[start..start + row_bytes as usize]);
    }
    drop(data);
    readback.unmap();
    Some(pixels)
}

/// Pixels that are neither background nor a fully covered line: partial coverage, i.e. antialiasing.
fn partial_coverage_pixels(pixels: &[u8]) -> usize {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| (8..=247).contains(&pixel[0]))
        .count()
}

#[test]
fn a_diagonal_line_resolves_to_partial_coverage() {
    let Some(plain) = render_line(1) else {
        return;
    };
    let Some(smooth) = render_line(4) else {
        return;
    };
    let hard_edges = partial_coverage_pixels(&plain);
    let soft_edges = partial_coverage_pixels(&smooth);
    eprintln!("diagonal line: 1 sample {hard_edges} partial pixels, 4 samples {soft_edges}");
    assert!(
        hard_edges < 20,
        "one sample must rasterize the line as whole pixels, got {hard_edges} partial pixels"
    );
    assert!(
        soft_edges > 100,
        "four samples must resolve the line's edges to grey levels, got {soft_edges} partial pixels"
    );
}
