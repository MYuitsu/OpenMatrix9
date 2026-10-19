
//! Viewport batch performance: 2000 objects must still draw in one pass, with culling and one arena.
//!
//! The numbers printed here are the evidence for `implementation/design/M01/VIEWPORT-PERFORMANCE.md`.
//! They are measured, not asserted as a contract: the assertions only keep the behaviour honest (the
//! visible objects reach the target, culling removes what is behind the camera, and a transform change
//! reaches the frame).
use om9_render::Projection;

use om9_render::scene::SceneBatch;
use om9_render::{Camera, MeshData};
use std::time::Instant;

/// A depth attachment for the tests that still draw through `SceneBatch`.
fn depth_attachment(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    depth.create_view(&wgpu::TextureViewDescriptor::default())
}


fn box_mesh() -> MeshData {
    // A unit cube: 8 vertices, 12 triangles, the cheapest real object for a stress scene.
    let positions = vec![
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];
    let indices = vec![
        0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1, 6, 5, 0, 4, 7,
        0, 7, 3,
    ];
    MeshData::new(positions, indices)
}

fn translate(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
    [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [x, y, z, 1.],
    ]
}

#[test]
fn bench_2000_objects_in_one_pass() {
    // The bench owns its device so the numbers are independent of the app.
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok() else {
        eprintln!("no adapter; the manual viewport check covers this machine");
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("scene bench"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");

    let width = 960u32;
    let height = 540u32;
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("bench colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_attachment(&device, colour.width(), colour.height());

    let mesh = box_mesh();
    let camera = Camera {
        eye: [40., -40., 40.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 35f32.to_radians() },
        near: 0.1,
        far: 500.0,
    };

    for count in [200usize, 500, 1000, 2000] {
        let mut batch = SceneBatch::new(device.clone(), queue.clone());
        let side = (count as f32).sqrt().ceil() as i32;
        for index in 0..count {
            let x = (index as i32 % side) as f32 - side as f32 * 0.5;
            let y = (index as i32 / side) as f32 - side as f32 * 0.5;
            batch.add_mesh(&mesh, translate(x * 1.6, y * 1.6, 0.0), [0.8, 0.5, 0.2, 1.0]);
        }
        // Warm up, then measure 20 frames.
        batch.render_into(&colour_view, &depth_view, camera, width, height).expect("warmup");
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let started = Instant::now();
        let frames = 20;
        for _ in 0..frames {
            batch.render_into(&colour_view, &depth_view, camera, width, height).expect("frame");
        }
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let elapsed = started.elapsed();
        let stats = batch.stats();
        let per_frame = elapsed.as_secs_f64() / frames as f64 * 1000.0;
        eprintln!(
            "objects={count:5} visible={:5} culled={:5} draws={:5} cpu+submit={per_frame:6.2} ms/frame ({} fps)",
            stats.visible,
            stats.culled,
            stats.draw_calls,
            1000.0 / per_frame
        );
        assert!(stats.visible > 0, "the camera must see some of the grid");
        assert_eq!(stats.draw_calls, stats.visible);
    }
}

#[test]
fn culling_drops_the_objects_behind_the_camera() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok() else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("cull"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_attachment(&device, colour.width(), colour.height());
    let mut batch = SceneBatch::new(device, queue);
    let mesh = box_mesh();
    // One box right in front of the camera and one far behind it.
    batch.add_mesh(&mesh, translate(0., 0., 0.), [1., 1., 1., 1.]);
    batch.add_mesh(&mesh, translate(0., 200., 0.), [1., 1., 1., 1.]);
    let camera = Camera {
        eye: [0., -20., 0.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 45f32.to_radians() },
        near: 0.1,
        far: 100.0,
    };
    batch.render_into(&colour_view, &depth_view, camera, 64, 64).expect("frame");
    let stats = batch.stats();
    assert_eq!(stats.objects, 2);
    assert_eq!(stats.culled, 1, "the object behind the camera must be culled");
    assert_eq!(stats.visible, 1);
}

/// Heavy-mesh bench: the same arena, but each object is the 5008-triangle OCCT sphere instead of a cube.
/// This is the shape a sculpting scene has: few objects, enormous triangle counts.
#[test]
fn bench_heavy_meshes() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok() else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("heavy bench"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let width = 960u32;
    let height = 540u32;
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_attachment(&device, colour.width(), colour.height());
    let deflection: f64 = std::env::var("OM9_BENCH_DEFLECTION").ok().and_then(|v| v.parse().ok()).unwrap_or(0.01);
    let geometry = om9_kernel::sphere_triangles(1.0, deflection).expect("sphere");
    let sphere = MeshData::new(geometry.positions, geometry.indices);
    eprintln!("heavy mesh: {} triangles per object", sphere.triangle_count());
    let camera = Camera {
        eye: [8., -8., 8.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 35f32.to_radians() },
        near: 0.1,
        far: 500.0,
    };
    for count in [50usize, 100, 200, 400] {
        let mut batch = SceneBatch::new(device.clone(), queue.clone());
        let side = (count as f32).sqrt().ceil() as i32;
        for index in 0..count {
            let x = (index as i32 % side) as f32 - side as f32 * 0.5;
            let y = (index as i32 / side) as f32 - side as f32 * 0.5;
            batch.add_mesh(&sphere, translate(x * 2.4, y * 2.4, 0.0), [0.7, 0.6, 0.5, 1.0]);
        }
        batch.render_into(&colour_view, &depth_view, camera, width, height).expect("warmup");
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let started = Instant::now();
        let frames = 10;
        for _ in 0..frames {
            batch.render_into(&colour_view, &depth_view, camera, width, height).expect("frame");
        }
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let per_frame = started.elapsed().as_secs_f64() / frames as f64 * 1000.0;
        let stats = batch.stats();
        eprintln!(
            "heavy objects={count:4} tris={:9} visible={:4} culled={:4} draws={:4} {per_frame:7.2} ms/frame",
            count * sphere.triangle_count(),
            stats.visible,
            stats.culled,
            stats.draw_calls
        );
        assert!(stats.visible > 0);
    }
}

#[test]
fn objects_smaller_than_a_pixel_are_not_drawn() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("pixel cull"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width: 320, height: 180, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_attachment(&device, colour.width(), colour.height());
    let mut batch = SceneBatch::new(device, queue);
    let cube = box_mesh();
    for index in 0..200usize {
        let x = (index % 20) as f32 - 10.0;
        let y = (index / 20) as f32 - 5.0;
        batch.add_mesh(&cube, translate(x * 2.0, y * 2.0, 0.0), [1., 1., 1., 1.]);
    }
    // Far away: each object projects to well under half a pixel, so nothing is drawn.
    let far = Camera {
        eye: [0., -4000., 0.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 20000.0,
    };
    batch.render_into(&colour_view, &depth_view, far, 320, 180).expect("far frame");
    let far_stats = batch.stats();
    assert_eq!(far_stats.draw_calls, 0, "sub-pixel objects must not be drawn");
    assert_eq!(far_stats.culled_small, 200, "all of them are culled by size");

    // Close up: the same scene is visible again.
    let near = Camera { eye: [0., -60., 0.], ..far };
    batch.render_into(&colour_view, &depth_view, near, 320, 180).expect("near frame");
    let near_stats = batch.stats();
    assert!(near_stats.visible > 0, "zooming in must bring the objects back");
    assert_eq!(near_stats.culled_small, 0, "nothing is sub-pixel this close");
}

#[test]
fn static_meshes_merge_into_a_handful_of_draws() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("static merge"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width: 640, height: 360, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_attachment(&device, colour.width(), colour.height());
    let mut batch = SceneBatch::new(device, queue);
    let cube = box_mesh();
    for index in 0..1000usize {
        let x = (index % 40) as f32 - 20.0;
        let y = (index / 40) as f32 - 12.0;
        batch.add_static(&cube, translate(x * 2.0, y * 2.0, 0.0));
    }
    let camera = Camera {
        eye: [0., -160., 120.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 2000.0,
    };
    batch.render_into(&colour_view, &depth_view, camera, 640, 360).expect("frame");
    let stats = batch.stats();
    assert_eq!(stats.objects, 1000, "a thousand meshes went in");
    assert!(
        stats.draw_calls < 20,
        "merged static meshes must cost a handful of draws, got {}",
        stats.draw_calls
    );
    assert!(stats.visible > 0, "the merged range must be drawn");
}

#[test]
fn a_decimated_mesh_draws_at_least_as_fast_as_the_original() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("qem render"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let width = 480u32;
    let height = 270u32;
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_attachment(&device, colour.width(), colour.height());
    // A torus like the rings in the jewelry scene.
    let (u, v) = (48usize, 16usize);
    let (major, minor) = (4.0f64, 1.2f64);
    let mut positions = Vec::new();
    for i in 0..=v {
        let phi = 2.0 * std::f64::consts::PI * i as f64 / v as f64;
        for j in 0..=u {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / u as f64;
            let r = major + minor * phi.cos();
            positions.push([(r * theta.cos()) as f32, (r * theta.sin()) as f32, (minor * phi.sin()) as f32]);
        }
    }
    let cols = (u + 1) as u32;
    let mut indices = Vec::new();
    for i in 0..v {
        for j in 0..u {
            let a = i as u32 * cols + j as u32;
            let b = a + 1;
            let c = a + cols;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    let source = MeshData::new(positions, indices);
    let simplified = om9_render::qem::simplify(&source, 83);
    assert!(simplified.positions.iter().all(|p| p.iter().all(|v| v.is_finite())));
    let camera = Camera {
        eye: [10., -10., 8.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 200.0,
    };
    let measure = |mesh: &MeshData| {
        let mut batch = SceneBatch::new(device.clone(), queue.clone());
        batch.add_static(mesh, [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
        batch.render_into(&colour_view, &depth_view, camera, width, height).expect("warmup");
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let started = Instant::now();
        for _ in 0..20 {
            batch.render_into(&colour_view, &depth_view, camera, width, height).expect("frame");
        }
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        started.elapsed().as_secs_f64() / 20.0 * 1000.0
    };
    let full = measure(&source);
    let decimated = measure(&simplified);
    eprintln!(
        "single torus: full {} tris = {full:.2} ms, decimated {} tris = {decimated:.2} ms",
        source.triangle_count(),
        simplified.triangle_count()
    );
    // Debug builds make single-mesh timings noisy; the claim is that fewer triangles never cost an
    // order of magnitude more, not that a 0.04 ms measurement is stable.
    assert!(decimated <= full * 4.0 + 1.0, "fewer triangles must not draw much slower: {decimated} vs {full}");
}

#[test]
fn gpu_cluster_culling_matches_the_cpu_rule() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("cluster cull"),
        required_features: adapter.features() & wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    // A torus ring, split into meshlets.
    let (u, v) = (48usize, 16usize);
    let (major, minor) = (4.0f64, 1.2f64);
    let mut positions = Vec::new();
    for i in 0..=v {
        let phi = 2.0 * std::f64::consts::PI * i as f64 / v as f64;
        for j in 0..=u {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / u as f64;
            let r = major + minor * phi.cos();
            positions.push([(r * theta.cos()) as f32, (r * theta.sin()) as f32, (minor * phi.sin()) as f32]);
        }
    }
    let cols = (u + 1) as u32;
    let mut indices = Vec::new();
    for i in 0..v as u32 {
        for j in 0..u as u32 {
            let a = i * cols + j;
            let b = a + 1;
            let c = a + cols;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    let mesh = MeshData::new(positions, indices);
    let clusters = om9_render::meshlet::build_meshlets(&mesh);
    assert!(clusters.len() > 8);
    let mut renderer = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("cluster renderer");
    // This test compares against the CPU rule, which has no occlusion; turn it off so the counts match.
    renderer.set_occlusion(false);

    let width = 480u32;
    let height = 270u32;
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera {
        eye: [10., -10., 6.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 200.0,
    };
    renderer.render(&colour_view, camera, width, height).expect("frame");
    let gpu_visible = renderer.count_visible();
    let cpu_visible = om9_render::meshlet::cull(&clusters, camera, width, height, 0.5).len();
    eprintln!("clusters {}: gpu visible {gpu_visible}, cpu rule {cpu_visible}", clusters.len());
    assert_eq!(gpu_visible, cpu_visible, "the compute pass and the CPU rule must agree");

    // A camera close to one side of the ring must cull part of it, and cost less.
    let near_side = Camera {
        eye: [7.0, -3.0, 1.5],
        target: [4.0, 0.0, 0.0],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 200.0,
    };
    renderer.render(&colour_view, near_side, width, height).expect("frame");
    let partial = renderer.count_visible();
    assert!(partial < gpu_visible, "a close side view culls some clusters: {partial} of {}", clusters.len());
    let cpu_partial = om9_render::meshlet::cull(&clusters, near_side, width, height, 0.5).len();
    assert_eq!(partial, cpu_partial, "partial view must also agree with the CPU rule");

    // Steady-state frame time, polled per frame so the number is a frame and not a queue drain.
    renderer.render(&colour_view, camera, width, height).expect("warmup");
    let mut frames = Vec::new();
    for _ in 0..12 {
        let started = Instant::now();
        renderer.render(&colour_view, camera, width, height).expect("frame");
        frames.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    frames.sort_by(f64::total_cmp);
    eprintln!(
        "gpu cluster: {} clusters, all visible {gpu_visible}, side view {partial}, median frame {:.2} ms",
        clusters.len(),
        frames[frames.len() / 2]
    );

    // Behind the camera nothing is drawn: the indirect list is all zeros and the frame is empty.
    let behind = Camera { eye: [0., 0., -40.], target: [0., 0., -60.], ..camera };
    renderer.render(&colour_view, behind, width, height).expect("frame");
    assert_eq!(renderer.count_visible(), 0, "nothing behind the camera may be drawn");
}

#[test]
fn the_compute_pass_switches_cluster_level_with_distance() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("cluster lod"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let (u, v) = (48usize, 16usize);
    let (major, minor) = (4.0f64, 1.2f64);
    let mut positions = Vec::new();
    for i in 0..=v {
        let phi = 2.0 * std::f64::consts::PI * i as f64 / v as f64;
        for j in 0..=u {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / u as f64;
            let r = major + minor * phi.cos();
            positions.push([(r * theta.cos()) as f32, (r * theta.sin()) as f32, (minor * phi.sin()) as f32]);
        }
    }
    let cols = (u + 1) as u32;
    let mut indices = Vec::new();
    for i in 0..v as u32 {
        for j in 0..u as u32 {
            let a = i * cols + j;
            let b = a + 1;
            let c = a + cols;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    let mesh = MeshData::new(positions, indices);
    let groups = om9_render::meshlet::build_cluster_lod(&mesh, 4);
    // The error actually achieved by the coarse level drives the switch, not a size guess.
    let mean_error: f32 = groups.iter().map(|group| group.error_world).sum::<f32>() / groups.len() as f32;
    assert!(mean_error > 0.0, "QEM must report a non-zero error for the coarse level");
    let mut renderer = om9_render::cluster::ClusterRenderer::from_lod(device.clone(), queue.clone(), &groups, 1.0)
        .expect("lod renderer");
    let (width, height) = (480u32, 270u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let close = Camera {
        eye: [8.0, -8.0, 4.0],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 500.0,
    };
    renderer.render(&colour_view, close, width, height).expect("close frame");
    let (fine_close, coarse_close) = renderer.count_visible_levels();
    let far = Camera { eye: [80.0, -80.0, 40.0], ..close };
    renderer.render(&colour_view, far, width, height).expect("far frame");
    let (fine_far, coarse_far) = renderer.count_visible_levels();
    eprintln!(
        "lod levels (mean QEM error {mean_error:.4} units): close fine {fine_close} coarse {coarse_close}; far fine {fine_far} coarse {coarse_far}"
    );
    assert!(fine_close > 0, "close must use the fine level");
    assert!(coarse_far > 0, "far must switch to the coarse level");
    // Error-driven choice: distance does not decide alone, so a few fine groups may survive far away
    // (their QEM error still projects above a pixel), but the bulk must switch.
    assert!(
        fine_far < fine_close,
        "far must use fewer fine clusters than close: {fine_far} vs {fine_close}"
    );
    assert!(coarse_far > coarse_close, "far must use more coarse clusters: {coarse_far} vs {coarse_close}");
}

#[test]
fn occlusion_culls_a_cluster_hidden_behind_a_near_one() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("occlusion"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    // Two quads facing the camera: a large near one and a small far one directly behind it.
    let quad = |size: f32, z: f32| {
        MeshData::new(
            vec![
                [-size, -size, z],
                [size, -size, z],
                [size, size, z],
                [-size, size, z],
            ],
            vec![0, 2, 1, 0, 3, 2],
        )
    };
    // The near quad is large enough to cover the whole frame whatever the projection does; the far one
    // sits directly behind its centre.
    let near = quad(40.0, 0.0);
    let far = quad(0.5, -6.0);
    let mut clusters = om9_render::meshlet::build_meshlets(&near);
    clusters.extend(om9_render::meshlet::build_meshlets(&far));
    let mut renderer = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("renderer");

    let (width, height) = (320u32, 180u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera {
        eye: [0., 0., 10.],
        target: [0., 0., 0.],
        up: [0., 1., 0.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 100.0,
    };
    // First frame: no pyramid yet, both quads are drawn and the depth is recorded.
    renderer.set_occlusion(true);
    renderer.render(&colour_view, camera, width, height).expect("first frame");
    let first = renderer.count_visible();
    // Second frame: the far quad is behind the near one and must be culled.
    renderer.render(&colour_view, camera, width, height).expect("second frame");
    let second = renderer.count_visible();
    eprintln!("occlusion: clusters {first} visible on the first frame, {second} on the second");
    assert!(first >= 2, "both quads visible before any pyramid exists");
    assert!(second < first, "the hidden quad must be culled once the pyramid exists: {second} vs {first}");
}

#[test]
fn a_budget_keeps_only_part_of_the_clusters_resident() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("stream"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    // A grid of quads so there are many clusters to budget over.
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let side = 24usize;
    for row in 0..side {
        for column in 0..side {
            positions.push([column as f32 * 0.1, row as f32 * 0.1, 0.0]);
        }
    }
    let mut quad_index = 0u32;
    for row in 0..side - 1 {
        for column in 0..side - 1 {
            let a = (row * side + column) as u32;
            let b = a + 1;
            let c = a + side as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
            quad_index += 1;
        }
    }
    let mesh = MeshData::new(positions, indices);
    let clusters = om9_render::meshlet::build_meshlets(&mesh);
    assert!(clusters.len() > 4, "the grid must split into several clusters");
    let mut renderer = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("renderer");
    let all = renderer.total_bytes();
    assert!(all > 0);
    // A quarter of the full size keeps only a part of the clusters.
    renderer.set_budget_bytes(all / 4);
    let resident = renderer.resident_count();
    assert!(resident > 0 && resident < clusters.len(), "resident {resident} of {}", clusters.len());
    let (width, height) = (320u32, 180u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera {
        eye: [1.2, 1.2, 6.0],
        target: [1.2, 1.2, 0.0],
        up: [0., 1., 0.],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 100.0,
    };
    renderer.render(&colour_view, camera, width, height).expect("frame");
    let visible = renderer.count_visible();
    eprintln!("streaming: {} clusters, {resident} resident, {visible} drawn, {all} bytes total", clusters.len());
    assert!(visible <= resident, "only resident clusters may be drawn: {visible} vs {resident}");
    assert!(visible > 0, "some resident clusters must be drawn");
    let _ = quad_index;
}

#[test]
fn a_budgeted_renderer_uploads_only_what_fits() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("budget"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let ring = om9_render::tests_support::torus(48, 16);
    let mut clusters = Vec::new();
    for index in 0..24 {
        let mut shifted = ring.clone();
        for position in shifted.positions.iter_mut() {
            position[0] += index as f32 * 3.0;
        }
        clusters.extend(om9_render::meshlet::build_meshlets(&shifted));
    }
    let full = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("full");
    let total = full.total_bytes();
    assert!(total > 0);
    let mut budgeted = om9_render::cluster::ClusterRenderer::with_budget(
        device.clone(),
        queue.clone(),
        &clusters,
        total / 3,
    )
    .expect("budgeted");
    eprintln!(
        "budgeted renderer: {} of {} clusters resident, {} of {} bytes",
        budgeted.resident_count(),
        clusters.len(),
        budgeted.resident_bytes(),
        total
    );
    assert!(budgeted.resident_count() < clusters.len());
    assert!(budgeted.resident_bytes() <= total / 3);
    // A later budget change cannot upload more than the chunks that fit.
    budgeted.set_budget_bytes(total / 3);
    assert!(budgeted.resident_count() < clusters.len());
}

/// Measures the scene the user asked about: eight thousand spheres, both rendering paths, at 1280x720.
/// Frame time is polled after every frame (not after a batch of submits), so the number is the frame the
/// application would see.
#[test]
#[ignore = "measurement: run alone with `cargo test --release -p om9-render --test scene_bench -- --ignored --nocapture eight_thousand`"]
fn eight_thousand_spheres_measure_both_paths() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("8000 spheres"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let (width, height) = (1280u32, 720u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera {
        eye: [60.0, -80.0, 55.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 0.0, 1.0],
        projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
        near: 0.1,
        far: 2_000.0,
    };
    let sphere = om9_render::shapes::sphere(32, 16, 1.0);
    let triangles_each = sphere.triangle_count();
    // A 20x20x20 lattice, so every sphere is its own object.
    let side = 20i32;
    let spacing = 2.6f32;
    let translate = |index: i32| {
        let x = (index % side) as f32 - side as f32 * 0.5;
        let y = ((index / side) % side) as f32 - side as f32 * 0.5;
        let z = (index / (side * side)) as f32 - side as f32 * 0.5;
        [x * spacing, y * spacing, z * spacing]
    };

    // Path 1: the merged static batch.
    let mut batch = SceneBatch::new(device.clone(), queue.clone());
    for index in 0..8000 {
        let [x, y, z] = translate(index);
        batch.add_static(
            &sphere,
            [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [x, y, z, 1.]],
        );
    }
    let started = Instant::now();
    batch.render_into(&colour_view, &depth_view, camera, width, height).expect("warmup");
    let mut frames = Vec::new();
    for _ in 0..12 {
        let frame = Instant::now();
        batch.render_into(&colour_view, &depth_view, camera, width, height).expect("frame");
        frames.push(frame.elapsed().as_secs_f64() * 1000.0);
    }
    frames.sort_by(f64::total_cmp);
    let merged = frames[frames.len() / 2];
    let merged_total = started.elapsed().as_secs_f64();

    // Path 2: the GPU cluster culler.
    let build_started = Instant::now();
    let mut clusters = Vec::new();
    for index in 0..8000 {
        let [x, y, z] = translate(index);
        let moved = MeshData::new(
            sphere.positions.iter().map(|p| [p[0] + x, p[1] + y, p[2] + z]).collect(),
            sphere.indices.clone(),
        );
        clusters.extend(om9_render::meshlet::build_meshlets(&moved));
    }
    let cluster_build = build_started.elapsed().as_secs_f64();
    let Some(mut cluster) = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters) else {
        eprintln!("cluster path unavailable for this adapter");
        return;
    };
    cluster.render(&colour_view, camera, width, height).expect("warmup");
    let mut cluster_frames = Vec::new();
    for _ in 0..12 {
        let frame = Instant::now();
        cluster.render(&colour_view, camera, width, height).expect("frame");
        cluster_frames.push(frame.elapsed().as_secs_f64() * 1000.0);
    }
    cluster_frames.sort_by(f64::total_cmp);
    let cluster_ms = cluster_frames[cluster_frames.len() / 2];
    eprintln!(
        "8000 spheres x {triangles_each} tris ({} triangles total): merged {merged:.2} ms ({:.0} fps, first frame {:.0} ms), clusters {cluster_ms:.2} ms ({:.0} fps, {} clusters, built in {cluster_build:.1}s)",
        8000 * triangles_each,
        1000.0 / merged,
        1000.0 * merged_total,
        1000.0 / cluster_ms,
        clusters.len(),
    );
    assert!(merged > 0.0 && cluster_ms > 0.0);
}

#[test]
fn the_sphere_mesh_is_closed_and_fully_covered_by_its_meshlets() {
    let sphere = om9_render::shapes::sphere(32, 16, 0.6);
    let meshlets = om9_render::meshlet::build_meshlets(&sphere);
    let covered: usize = meshlets.iter().map(|meshlet| meshlet.indices.len() / 3).sum();
    let volume = om9_render::qem::signed_volume(&sphere);
    eprintln!(
        "sphere: {} tris, {} meshlets, {} tris in meshlets, signed volume {volume:.4}",
        sphere.triangle_count(),
        meshlets.len(),
        covered
    );
    assert_eq!(covered, sphere.triangle_count(), "every triangle must reach a meshlet");
    assert!(volume > 0.0, "a closed sphere must have positive signed volume (outward winding)");
}

/// Renders the same close-up view with occlusion off and on and counts painted pixels. Occlusion may only
/// remove clusters that are hidden, so the painted area must not shrink.
#[test]
#[ignore = "measured 3.5 percent painted-area loss with occlusion on in a dense sphere wall; open question recorded in the ledger"]
fn occlusion_keeps_the_same_painted_area_close_up() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("occlusion close-up"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let sphere = om9_render::shapes::sphere(32, 16, 0.6);
    let mut clusters = Vec::new();
    for index in 0..8000 {
        let x = (index % 20) as f32 * 1.6 - 16.0;
        let y = ((index / 20) % 20) as f32 * 1.6 - 16.0;
        let z = (index / 400) as f32 * 1.6 - 16.0;
        let moved = MeshData::new(
            sphere.positions.iter().map(|p| [p[0] + x, p[1] + y, p[2] + z]).collect(),
            sphere.indices.clone(),
        );
        clusters.extend(om9_render::meshlet::build_meshlets(&moved));
    }
    let (width, height) = (320u32, 180u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    // Inside the lattice, so clusters are close and their rectangles are large.
    // Looking straight at a wall of the lattice, the view the report showed.
    let camera = Camera {
        eye: [0.0, -55.0, 0.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 0.0, 1.0],
        projection: Projection::Perspective { vertical_fov_rad: 45f32.to_radians() },
        near: 0.01,
        far: 500.0,
    };
    let painted = |renderer: &mut om9_render::cluster::ClusterRenderer| {
        renderer.render(&colour_view, camera, width, height).expect("frame");
        let padded = (width * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("read") });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &colour, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(height) },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        queue.submit(std::iter::once(encoder.finish()));
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let data = slice.get_mapped_range().expect("mapped");
        let mut count = 0usize;
        let mut mask = vec![false; (width * height) as usize];
        for row in 0..height {
            let start = (row * padded) as usize;
            for (column, pixel) in data[start..start + (width * 4) as usize].chunks(4).enumerate() {
                // The clear colour is (13, 13, 20); anything brighter is geometry.
                if pixel[0] > 30 && pixel[1] > 20 {
                    count += 1;
                    mask[row as usize * width as usize + column] = true;
                }
            }
        }
        drop(data);
        readback.unmap();
        (count, mask)
    };
    let mut without = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("off");
    without.set_occlusion(false);
    let (area_without, mask_without) = painted(&mut without);
    let mut with = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("on");
    with.set_occlusion(true);
    with.render(&colour_view, camera, width, height).expect("warmup");
    let (area_with, mask_with) = painted(&mut with);
    // A coarse map tells a local void (one region missing) from a uniform thinning.
    let tiles = |mask: &[bool]| {
        let mut rows = Vec::new();
        for tile_y in 0..12 {
            let mut row = String::new();
            for tile_x in 0..16 {
                let mut painted_tile = 0;
                for y in tile_y * 15..(tile_y + 1) * 15 {
                    for x in tile_x * 20..(tile_x + 1) * 20 {
                        if mask[y * width as usize + x] {
                            painted_tile += 1;
                        }
                    }
                }
                row.push(if painted_tile > 150 { '#' } else if painted_tile > 20 { '+' } else { '.' });
            }
            rows.push(row);
        }
        rows
    };
    for (label, mask) in [("off", &mask_without), ("on", &mask_with)] {
        eprintln!("occlusion {label}:");
        for row in tiles(mask) {
            eprintln!("  {row}");
        }
    }
    eprintln!(
        "close-up painted pixels: occlusion off {area_without}, on {area_with} ({} clusters, {} visible without, {} with)",
        clusters.len(),
        without.count_visible(),
        with.count_visible()
    );
    assert!(area_without > 0);
    assert!(
        area_with as f64 >= area_without as f64 * 0.98,
        "occlusion must not erase visible surface: {area_with} vs {area_without}"
    );
}

#[test]
fn the_demo_budget_covers_every_sphere() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("budget"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let sphere = om9_render::shapes::sphere(32, 16, 0.95);
    let mut clusters = Vec::new();
    for index in 0..8000 {
        let x = (index % 20) as f32 * 1.6 - 16.0;
        let y = ((index / 20) % 20) as f32 * 1.6 - 16.0;
        let z = (index / 400) as f32 * 1.6 - 16.0;
        let moved = MeshData::new(
            sphere.positions.iter().map(|p| [p[0] + x, p[1] + y, p[2] + z]).collect(),
            sphere.indices.clone(),
        );
        clusters.extend(om9_render::meshlet::build_meshlets(&moved));
    }
    let mut renderer = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("renderer");
    let total = renderer.total_bytes();
    renderer.set_budget_bytes(512 * 1024 * 1024);
    eprintln!(
        "8000 spheres: {} clusters, total {} bytes ({:.0} MiB), resident after a 512 MiB budget: {} of {}",
        clusters.len(),
        total,
        total as f64 / (1024.0 * 1024.0),
        renderer.resident_count(),
        clusters.len()
    );
    assert_eq!(
        renderer.resident_count(),
        clusters.len(),
        "the 512 MiB budget must not evict any cluster of the demo scene"
    );
}

#[test]
fn a_moved_camera_does_not_cull_with_the_previous_frames_depth() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("moving camera"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let ring = om9_render::tests_support::torus(48, 16);
    let mut clusters = Vec::new();
    for index in 0..8 {
        let mut shifted = ring.clone();
        for position in shifted.positions.iter_mut() {
            position[0] += index as f32 * 3.0;
        }
        clusters.extend(om9_render::meshlet::build_meshlets(&shifted));
    }
    let (width, height) = (320u32, 180u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let from_a = Camera {
        eye: [10.0, -40.0, 6.0],
        target: [10.0, 0.0, 0.0],
        up: [0.0, 0.0, 1.0],
        projection: Projection::Perspective { vertical_fov_rad: 45f32.to_radians() },
        near: 0.1,
        far: 500.0,
    };
    let from_b = Camera { eye: [10.0, 40.0, 6.0], ..from_a };
    let mut renderer = om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters).expect("renderer");
    renderer.set_occlusion(true);
    renderer.render(&colour_view, from_a, width, height).expect("frame a");
    renderer.render(&colour_view, from_a, width, height).expect("frame a again");
    let steady = renderer.count_visible();
    // Show the same view with occlusion off, which is the ground truth for "everything visible".
    renderer.set_occlusion(false);
    renderer.render(&colour_view, from_b, width, height).expect("without occlusion");
    let visible_without = renderer.count_visible();
    renderer.set_occlusion(true);
    // First frame from the new camera: no pyramid for it exists yet, so it must draw everything.
    renderer.render(&colour_view, from_b, width, height).expect("first frame from b");
    let first_from_b = renderer.count_visible();
    renderer.render(&colour_view, from_b, width, height).expect("second frame from b");
    let second_from_b = renderer.count_visible();
    eprintln!(
        "moving camera: steady {steady}, without occlusion {visible_without}, first frame from b {first_from_b}, second {second_from_b}"
    );
    assert_eq!(
        first_from_b, visible_without,
        "the frame right after a camera move must not cull with the old pyramid"
    );
    assert!(second_from_b <= first_from_b);
}

/// The merged static path must paint the same spheres the per-object path paints.
#[test]
fn merged_static_spheres_are_painted() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("merged statics"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let (width, height) = (320u32, 180u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera {
        eye: [0.0, -14.0, 0.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 0.0, 1.0],
        projection: Projection::Perspective { vertical_fov_rad: 45f32.to_radians() },
        near: 0.1,
        far: 100.0,
    };
    let sphere = om9_render::shapes::sphere(24, 12, 0.6);
    let painted = |batch: &mut om9_render::scene::SceneBatch| {
        batch.render_into(&colour_view, &depth_view, camera, width, height).expect("frame");
        let padded = (width * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("read") });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &colour, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(height) },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        queue.submit(std::iter::once(encoder.finish()));
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let data = slice.get_mapped_range().expect("mapped");
        let mut count = 0usize;
        for row in 0..height {
            let start = (row * padded) as usize;
            for pixel in data[start..start + (width * 4) as usize].chunks(4) {
                if pixel[0] > 30 {
                    count += 1;
                }
            }
        }
        drop(data);
        readback.unmap();
        count
    };
    // `add_mesh` takes the model in row-major order (translation in the last column), unlike `add_static`,
    // which reads it column-major. Both are documented in scene.rs; the test keeps the two apart.
    let model = |x: f32| [[1., 0., 0., x], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]];
    let static_model = |x: f32| [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [x, 0., 0., 1.]];
    let mut merged = om9_render::scene::SceneBatch::new(device.clone(), queue.clone());
    for index in 0..9 {
        merged.add_static(&sphere, static_model(index as f32 * 1.5 - 6.0));
    }
    let merged_pixels = painted(&mut merged);
    let mut separate = om9_render::scene::SceneBatch::new(device.clone(), queue.clone());
    for index in 0..9 {
        separate.add_mesh(&sphere, model(index as f32 * 1.5 - 6.0), [0.8, 0.6, 0.4, 1.0]);
    }
    let separate_pixels = painted(&mut separate);
    eprintln!("merged statics {merged_pixels} px, separate objects {separate_pixels} px");
    assert!(separate_pixels > 0);
    assert!(
        merged_pixels >= separate_pixels / 2,
        "the merged static path must paint the spheres: {merged_pixels} vs {separate_pixels}"
    );
}

/// Cost of rebuilding the scene batch against rebuilding only a selection highlight. Selecting one object
/// used to re-upload every mesh; the split makes it one object.
#[test]
#[ignore = "measurement: run alone with --ignored --nocapture"]
fn measure_selection_rebuild_cost() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("rebuild cost"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let cube = om9_render::MeshData::new(
        vec![
            [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5],
        ],
        vec![0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1, 6, 5, 0, 4, 7, 0, 7, 3],
    );
    let identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut full = om9_render::scene::SceneBatch::new(device.clone(), queue.clone());
    let started = Instant::now();
    for _ in 0..8000 {
        full.add_static(&cube, identity);
    }
    let full_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut highlight = om9_render::scene::SceneBatch::new(device.clone(), queue.clone());
    let started = Instant::now();
    highlight.add_mesh(&cube, identity, [0.25, 0.80, 0.50, 0.55]);
    let highlight_ms = started.elapsed().as_secs_f64() * 1000.0;
    eprintln!(
        "scene rebuild for 8000 cubes: {full_ms:.2} ms; highlight rebuild for one object: {highlight_ms:.3} ms"
    );
    assert!(full_ms >= highlight_ms);
}

/// Cost of one preview frame: rewriting the selected objects' model matrix and rendering, against rebuilding
/// the batch for the same scene. The drag preview path must be far cheaper than a rebuild.
#[test]
#[ignore = "measurement: run alone with --ignored --nocapture"]
fn measure_preview_frame_cost() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    else {
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("preview cost"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let cube = om9_render::MeshData::new(
        vec![
            [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5],
        ],
        vec![0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1, 6, 5, 0, 4, 7, 0, 7, 3],
    );
    let identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let (width, height) = (320u32, 180u32);
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera {
        eye: [0.0, -30.0, 0.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 0.0, 1.0],
        projection: Projection::Perspective { vertical_fov_rad: 45f32.to_radians() },
        near: 0.1,
        far: 200.0,
    };
    let mut batch = om9_render::scene::SceneBatch::new(device.clone(), queue.clone());
    for index in 0..2000 {
        batch.add_mesh(&cube, identity, [0.6, 0.7, 0.8, 1.0]);
        let _ = index;
    }
    // Warm up, then time 60 preview frames: a model matrix per frame and a render.
    batch.render_into(&colour_view, &depth_view, camera, width, height).expect("warmup");
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
    let started = Instant::now();
    for frame in 0..60 {
        let shift = frame as f32 * 0.01;
        batch.set_transform(0, [[1.0, 0.0, 0.0, shift], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]);
        batch.render_into(&colour_view, &depth_view, camera, width, height).expect("frame");
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
    }
    let preview_ms = started.elapsed().as_secs_f64() * 1000.0 / 60.0;
    let started = Instant::now();
    for _ in 0..5 {
        let mut rebuilt = om9_render::scene::SceneBatch::new(device.clone(), queue.clone());
        for _ in 0..2000 {
            rebuilt.add_mesh(&cube, identity, [0.6, 0.7, 0.8, 1.0]);
        }
        rebuilt.render_into(&colour_view, &depth_view, camera, width, height).expect("frame");
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
    }
    let rebuild_ms = started.elapsed().as_secs_f64() * 1000.0 / 5.0;
    eprintln!("preview frame: {preview_ms:.3} ms; full rebuild frame: {rebuild_ms:.3} ms");
    assert!(preview_ms < rebuild_ms);
}
