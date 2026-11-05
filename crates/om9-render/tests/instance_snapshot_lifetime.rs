// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_render::{Camera, MeshData, Projection, scene::SceneBatch};

#[test]
fn static_index_heavy_mesh_uses_independent_arena_capacities() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(), flags: Default::default(),
        memory_budget_thresholds: Default::default(), backend_options: Default::default(), display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false,
        compatible_surface: None, apply_limit_buckets: false,
    })).ok() else { eprintln!("SKIP: no GPU adapter"); return };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(), required_limits: adapter.limits(), ..Default::default()
    })).unwrap();
    let mut batch = SceneBatch::new(device.clone(), queue.clone());
    assert_eq!(batch.gpu_buffer_capacity_bytes(), 0, "empty scenes must not allocate geometry buffers");
    let mut blank_fork = batch.fork_samples(1);
    assert!(blank_fork.geometry_buffer_handles().is_empty());
    let texture = |format| device.create_texture(&wgpu::TextureDescriptor {
        label: Some("empty scene"), size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
    });
    let colour = texture(wgpu::TextureFormat::Rgba8Unorm);
    let depth = texture(wgpu::TextureFormat::Depth32Float);
    let camera = Camera { eye: [0., 0., 10.], target: [0., 0., 0.], up: [0., 1., 0.],
        projection: Projection::Orthographic { vertical_span: 10. }, near: 0.1, far: 100. };
    let mut encoder = device.create_command_encoder(&Default::default());
    blank_fork.encode_pass(&mut encoder, &colour.create_view(&Default::default()), None,
        &depth.create_view(&Default::default()), camera, 64, 64, true).unwrap();
    assert_eq!(blank_fork.stats().draw_calls, 0);
    assert!(blank_fork.geometry_buffer_handles().is_empty());
    queue.submit([encoder.finish()]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    drop(blank_fork);
    let indices = (0..550_002).map(|index| index % 3).collect();
    let mesh = MeshData::new(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], indices);
    batch.add_static(&mesh, [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
    assert_eq!(batch.gpu_buffer_capacity_bytes(), 10 * 1024 * 1024,
        "2 MiB vertex and 8 MiB index chunk without an unused initial pair");
    assert_eq!(batch.pending_upload_bytes(), 3 * 12 + 550_002 * 4);
    batch.flush_uploads();
    let fork = batch.fork_samples(1);
    assert_eq!(fork.geometry_buffer_handles(), batch.geometry_buffer_handles(), "fork shares actual GPU arenas");
    assert_eq!(fork.pending_upload_bytes(), 0);
}

#[test]
fn replaced_snapshots_survive_until_queued_draws_finish() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(), flags: Default::default(),
        memory_budget_thresholds: Default::default(), backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false,
        compatible_surface: None, apply_limit_buckets: false,
    })).ok() else { eprintln!("SKIP: no GPU adapter"); return };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(), required_limits: adapter.limits(), ..Default::default()
    })).unwrap();
    let texture = |format, usage| device.create_texture(&wgpu::TextureDescriptor {
        label: Some("instance lifetime"), size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format, usage, view_formats: &[],
    });
    let colours = [0, 1].map(|_| texture(wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC));
    let depths = [0, 1].map(|_| texture(wgpu::TextureFormat::Depth32Float, wgpu::TextureUsages::RENDER_ATTACHMENT));
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("instance lifetime pixels"), size: 2 * 64 * 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
    });
    let mut batch = SceneBatch::new(device.clone(), queue.clone());
    assert_eq!(batch.gpu_buffer_capacity_bytes(), 0);
    batch.set_culling(false);
    let mut positions = vec![[0., 0., 0.]; 200_000];
    positions[..4].copy_from_slice(&[[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]]);
    let mesh = MeshData::new(positions, vec![0, 1, 2, 0, 2, 3]);
    let model = |x| [[1., 0., 0., x], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]];
    let object = batch.add_mesh(&mesh, model(-2.), [1., 0., 0., 1.]);
    assert_eq!(batch.gpu_buffer_capacity_bytes(), 10 * 1024 * 1024,
        "8 MiB vertex and 2 MiB index chunk without an unused initial pair");
    let camera = Camera { eye: [0., 0., 10.], target: [0., 0., 0.], up: [0., 1., 0.],
        projection: Projection::Orthographic { vertical_span: 10. }, near: 0.1, far: 100. };
    let mut encoder = device.create_command_encoder(&Default::default());
    for i in 0..2 {
        if i == 1 { batch.set_transform(object, model(2.)); }
        batch.encode_pass(&mut encoder, &colours[i].create_view(&Default::default()), None,
            &depths[i].create_view(&Default::default()), camera, 64, 64, true).unwrap();
        assert!(batch.stats().instance_snapshot_bytes > 0, "different transforms need different immutable snapshots");
    }
    assert!(batch.instance_snapshot_cpu_bytes() > 0);
    let before_evict = batch.gpu_buffer_capacity_bytes();
    batch.evict_instance_snapshot();
    assert_eq!(batch.instance_snapshot_cpu_bytes(), 0);
    assert!(batch.gpu_buffer_capacity_bytes() < before_evict);
    batch.clear();
    for (i, colour) in colours.iter().enumerate() {
        encoder.copy_texture_to_buffer(colour.as_image_copy(), wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout { offset: (i * 64 * 256) as u64, bytes_per_row: Some(256), rows_per_image: Some(64) },
        }, wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 });
    }
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback.slice(..).map_async(wgpu::MapMode::Read, move |result| { let _ = sender.send(result); });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let data = readback.slice(..).get_mapped_range().unwrap();
    let pixel = |frame: usize, x: usize| -> [u8; 4] {
        let start = frame * 64 * 256 + 32 * 256 + x * 4;
        data[start..start + 4].try_into().unwrap()
    };
    assert_ne!(pixel(0, 19), pixel(0, 45), "first frame must contain the object only on the left");
    assert_eq!(pixel(0, 19), pixel(1, 45), "second frame must draw the moved object on the right");
    assert_eq!(pixel(0, 45), pixel(1, 19), "background must remain unchanged after moving");
    drop(data);
    readback.unmap();
}
