//! M00-L4-03-02-04 — device loss and fallback renderer.
//!
//! Two facts the gate needs, measured instead of assumed:
//!
//! 1. **fallback**: request a software adapter (`force_fallback_adapter: true`) and render one frame on
//!    it. If the machine has none, the probe says so instead of pretending a fallback exists.
//! 2. **loss**: register the device-lost callback, render a frame, call `Device::destroy()` (the only
//!    way to make wgpu lose a device through its public API), check that the destroyed device can no
//!    longer poll or draw, then build a fresh device and render again. That is the recovery path an
//!    application needs when a driver resets.
//!
//! Feasibility only: `native/`, not product code.
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use wgpu::util::DeviceExt;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

struct Handles {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex: wgpu::Buffer,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
}

fn build(adapter: &wgpu::Adapter) -> Handles {
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("loss-probe"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("triangle"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @vertex
            fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
                return vec4<f32>(position, 1.0);
            }
            @fragment
            fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.9, 0.5, 0.1, 1.0); }
            "#
            .into(),
        ),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("triangle"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: 12,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            })],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let vertices: [[f32; 3]; 3] = [[0.0, 0.6, 0.0], [-0.6, -0.5, 0.0], [0.6, -0.5, 0.0]];
    let vertex_data: Vec<u8> = vertices
        .iter()
        .flat_map(|position| position.iter().flat_map(|value| value.to_le_bytes()))
        .collect();
    let vertex = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertices"),
        contents: &vertex_data,
        usage: wgpu::BufferUsages::VERTEX,
    });
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("target"),
        size: wgpu::Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(WIDTH) * u64::from(HEIGHT) * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    Handles { device, queue, pipeline, vertex, texture, view, readback }
}

/// Renders one frame and returns the painted pixel count, or `None` when the device refuses.
fn render(handles: &Handles) -> Option<u64> {
    let mut encoder = handles
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &handles.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&handles.pipeline);
        pass.set_vertex_buffer(0, handles.vertex.slice(..));
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &handles.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &handles.readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
    );
    handles.queue.submit(std::iter::once(encoder.finish()));
    if handles
        .device
        .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
        .is_err()
    {
        return None;
    }
    let slice = handles.readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    if handles
        .device
        .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
        .is_err()
    {
        return None;
    }
    let data = slice.get_mapped_range().ok()?;
    let mut painted = 0u64;
    for pixel in data.chunks_exact(4) {
        if pixel[0] > 120 {
            painted += 1;
        }
    }
    drop(data);
    handles.readback.unmap();
    Some(painted)
}

fn request(power: wgpu::PowerPreference, fallback: bool) -> Option<(wgpu::Instance, wgpu::Adapter)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: power,
        force_fallback_adapter: fallback,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()?;
    Some((instance, adapter))
}

fn main() {
    // 1. the normal adapter renders, so the later failure is attributable to the destroy.
    let (_instance, adapter) = request(wgpu::PowerPreference::HighPerformance, false).expect("primary adapter");
    let handles = build(&adapter);
    let before = render(&handles);
    let lost_callbacks = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&lost_callbacks);
    handles.device.set_device_lost_callback(move |_reason, _message| {
        counter.fetch_add(1, Ordering::SeqCst);
    });

    // 2. destroy the device: the only public way to make wgpu lose one.
    handles.device.destroy();
    let after_destroy_poll = handles
        .device
        .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
        .is_ok();
    let after_destroy_draw = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(&handles)))
        .unwrap_or(None);
    let callbacks = lost_callbacks.load(Ordering::SeqCst);

    // 3. recovery: a fresh adapter and a fresh pipeline must render again.
    let started = Instant::now();
    let (_new_instance, new_adapter) = request(wgpu::PowerPreference::HighPerformance, false).expect("adapter after loss");
    let recovered_handles = build(&new_adapter);
    let recovered = render(&recovered_handles);
    let recovery_seconds = started.elapsed().as_secs_f64();

    // 4. fallback: a software adapter, if the machine offers one.
    let fallback = match request(wgpu::PowerPreference::LowPower, true) {
        Some((_instance, fallback_adapter)) => {
            let info = fallback_adapter.get_info();
            let fallback_handles = build(&fallback_adapter);
            let started = Instant::now();
            let painted = render(&fallback_handles);
            let seconds = started.elapsed().as_secs_f64();
            Some(format!(
                "{{\"name\": \"{}\", \"backend\": \"{:?}\", \"device_type\": \"{:?}\", \"frame_seconds\": {seconds:.6}, \"painted_pixels\": {}}}",
                info.name,
                info.backend,
                info.device_type,
                painted.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string())
            ))
        }
        None => None,
    };

    println!(
        "{{\"primary\": {{\"painted_before_loss\": {}}}, \
         \"loss\": {{\"device_lost_callbacks\": {callbacks}, \"poll_ok_after_destroy\": {after_destroy_poll}, \
         \"draw_after_destroy\": {}, \"recovered\": {}, \"recovered_painted\": {}, \"recovery_seconds\": {recovery_seconds:.4}}}, \
         \"fallback\": {}}}",
        before.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
        after_destroy_draw.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
        recovered.is_some(),
        recovered.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
        fallback.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
    );
}
