// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shared encoder cameras must match independent LOD0 renders, including transformed occurrences.
use om9_render::{Camera, MeshData, scene::SceneBatch};
const SIDE: u32 = 256;
fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    gpu_features(wgpu::Features::TIMESTAMP_QUERY)
}
fn gpu_features(features: wgpu::Features) -> Option<(wgpu::Device, wgpu::Queue)> {
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
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("retained navigation test"),
        required_features: adapter.features() & features,
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .ok()
}
#[test]
fn adapter_without_enabled_timestamp_feature_uses_the_explicit_fallback() {
    let Some((device, _)) = gpu_features(wgpu::Features::empty()) else {
        return;
    };
    assert!(om9_render::timing::GpuTimer::new(&device).is_none());
}
fn texture(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("navigation target"),
        size: wgpu::Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | if samples == 1 && format == wgpu::TextureFormat::Rgba8Unorm {
                wgpu::TextureUsages::COPY_SRC
            } else {
                wgpu::TextureUsages::empty()
            },
        view_formats: &[],
    })
}
struct Target {
    colour: wgpu::Texture,
    msaa: wgpu::Texture,
    depth: wgpu::Texture,
}
impl Target {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            colour: texture(device, wgpu::TextureFormat::Rgba8Unorm, 1),
            msaa: texture(device, wgpu::TextureFormat::Rgba8Unorm, 4),
            depth: texture(device, wgpu::TextureFormat::Depth32Float, 4),
        }
    }
    fn encode(&self, batch: &mut SceneBatch, encoder: &mut wgpu::CommandEncoder, camera: Camera) {
        batch
            .encode_pass(
                encoder,
                &self.msaa.create_view(&Default::default()),
                Some(&self.colour.create_view(&Default::default())),
                &self.depth.create_view(&Default::default()),
                camera,
                SIDE,
                SIDE,
                true,
            )
            .unwrap();
    }
    fn pixels(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u8> {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("navigation pixels"),
            size: (SIDE * SIDE * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            self.colour.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(SIDE * 4),
                    rows_per_image: Some(SIDE),
                },
            },
            wgpu::Extent3d {
                width: SIDE,
                height: SIDE,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let bytes = buffer.slice(..).get_mapped_range().unwrap().to_vec();
        buffer.unmap();
        bytes
    }
}
fn model(x: f32) -> [[f32; 4]; 4] {
    [
        [1., 0., 0., x],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]
}
#[test]
fn four_cameras_in_one_encoder_match_independent_full_quality_images() {
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP: no GPU adapter");
        return;
    };
    let mesh = MeshData::new(
        vec![[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]],
        vec![0, 1, 2],
    );
    let mut instanced = SceneBatch::with_samples(device.clone(), queue.clone(), 4);
    let source = instanced.add_mesh(&mesh, model(-1.5), [1., 0.2, 0.1, 1.]);
    let before = instanced.pending_upload_bytes();
    instanced
        .add_instance(source, model(1.5), [0.1, 0.8, 0.3, 1.])
        .unwrap();
    assert_eq!(
        before,
        instanced.pending_upload_bytes(),
        "an occurrence must not upload its mesh again"
    );
    let mut original = SceneBatch::with_samples(device.clone(), queue.clone(), 4);
    original.add_mesh(&mesh, model(-1.5), [1., 0.2, 0.1, 1.]);
    original.add_mesh(&mesh, model(1.5), [0.1, 0.8, 0.3, 1.]);
    // Exercise incremental admission, including a zero-byte service round.
    assert_eq!(
        instanced.flush_uploads_budget(0, std::time::Instant::now()),
        0
    );
    while instanced.pending_upload_bytes() > 0 {
        let admitted = instanced.flush_uploads_budget(
            16,
            std::time::Instant::now() + std::time::Duration::from_secs(1),
        );
        assert!(admitted > 0 && admitted <= 16);
    }
    let cameras = std::array::from_fn::<_, 4, _>(|i| Camera {
        eye: [i as f32 * 2., -2., 14.],
        ..Camera::default()
    });
    let targets = cameras.map(|_| Target::new(&device));
    let mut encoder = device.create_command_encoder(&Default::default());
    for (target, camera) in targets.iter().zip(cameras) {
        target.encode(&mut instanced, &mut encoder, camera);
    }
    queue.submit([encoder.finish()]);
    let mut distinct = Vec::new();
    for (target, camera) in targets.iter().zip(cameras) {
        let reference = Target::new(&device);
        let mut encoder = device.create_command_encoder(&Default::default());
        reference.encode(&mut original, &mut encoder, camera);
        queue.submit([encoder.finish()]);
        let pixels = target.pixels(&device, &queue);
        assert!(
            pixels
                .chunks_exact(4)
                .filter(|p| p[0] > 60 || p[1] > 60)
                .count()
                > 20
        );
        assert_eq!(
            pixels,
            reference.pixels(&device, &queue),
            "the shared encoder must keep this camera and both occurrence tints"
        );
        distinct.push(pixels);
    }
    assert_ne!(
        distinct[0], distinct[3],
        "the four targets must not all receive the last camera"
    );
}
#[test]
fn optional_gpu_timestamps_are_nonblocking_and_tag_the_submitted_frame() {
    let Some((device, queue)) = gpu() else {
        return;
    };
    let Some(mut timer) = om9_render::timing::GpuTimer::new(&device) else {
        eprintln!("SKIP: timestamp queries unsupported");
        return;
    };
    let colour = texture(&device, wgpu::TextureFormat::Rgba8Unorm, 1);
    let view = colour.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    timer.marker(&mut encoder, &view, false);
    timer.marker(&mut encoder, &view, true);
    timer.resolve(&mut encoder);
    queue.submit([encoder.finish()]);
    timer.submitted(73);
    assert!(!timer.available());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let (frame, ms) = timer.poll(&device, &queue).expect("timestamp result");
    assert_eq!(frame, 73);
    assert!(ms.is_finite() && ms >= 0.);
    assert!(timer.available());
}
