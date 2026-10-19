//! A jewelry-sized archive: many small ring and stone meshes, imported and drawn in four views.
//!
//! The counts come from the environment so the same test covers a smoke run and the gigabyte run:
//!
//! ```text
//! OM9_JEWELRY_RINGS=200  OM9_JEWELRY_STONES=400  OM9_JEWELRY_U=48  OM9_JEWELRY_V=16
//! cargo test --release -p om9-3dm --test jewelry_scene -- --nocapture
//! ```
//!
//! The four views orbit, pan and zoom while measuring, which is exactly the gesture the viewport has to
//! stay smooth under.
use om9_render::Projection;

use om9_3dm::{flatten, read_meshes, write_jewelry_fixture};
use om9_render::scene::SceneBatch;
use om9_render::Camera;
use std::time::Instant;

fn env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

fn orbit(eye: [f32; 3], target: [f32; 3], step: f32) -> [f32; 3] {
    let dx = eye[0] - target[0];
    let dy = eye[1] - target[1];
    let (sin, cos) = step.sin_cos();
    [target[0] + dx * cos - dy * sin, target[1] + dx * sin + dy * cos, eye[2]]
}

#[test]
fn the_jewelry_archive_imports_and_draws_in_four_views() {
    let rings = env("OM9_JEWELRY_RINGS", 40);
    let stones = env("OM9_JEWELRY_STONES", 80);
    let ring_u = env("OM9_JEWELRY_U", 48);
    let ring_v = env("OM9_JEWELRY_V", 16);
    let path = std::env::temp_dir().join(format!("om9-jewelry-{rings}-{stones}-{ring_u}x{ring_v}.3dm"));

    let started = Instant::now();
    write_jewelry_fixture(&path, rings, stones, ring_u, ring_v).expect("write jewelry archive");
    let file_bytes = std::fs::metadata(&path).expect("metadata").len();
    let write_seconds = started.elapsed().as_secs_f64();

    let started = Instant::now();
    let meshes = read_meshes(&path).expect("import jewelry archive");
    let import_seconds = started.elapsed().as_secs_f64();
    assert_eq!(meshes.len(), rings + stones, "every ring and stone comes back");

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
        eprintln!("no adapter; the manual viewport check covers this machine");
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("jewelry bench"),
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

    // Screen-space LOD + dynamic resolution: each row is (label, error budget in px, target size).
    // Idle frames pay for 1 px at full resolution; navigation frames trade 2 px and half resolution.
    // Whole scene through the GPU cluster culler: one meshlet set per piece, one indirect draw.
    if std::env::var("OM9_JEWELRY_CLUSTER").is_ok() {
        let camera = Camera {
            eye: [0., -260., 210.],
            target: [0., 0., 0.],
            up: [0., 0., 1.],
            projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
            near: 0.1,
            far: 5000.0,
        };
        let started = Instant::now();
        let mut clusters = Vec::new();
        for mesh in &meshes {
            let flat = flatten(mesh);
            clusters.extend(om9_render::meshlet::build_meshlets(&om9_render::MeshData::new(
                flat.positions,
                flat.indices,
            )));
        }
        let build_seconds = started.elapsed().as_secs_f64();
        let budget_mb = std::env::var("OM9_JEWELRY_STREAM_MB")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok());
        let prepared = match budget_mb {
            Some(megabytes) => om9_render::cluster::ClusterRenderer::with_budget(
                device.clone(),
                queue.clone(),
                &clusters,
                megabytes * 1024 * 1024,
            ),
            None => om9_render::cluster::ClusterRenderer::new(device.clone(), queue.clone(), &clusters),
        };
        let Some(mut renderer) = prepared else {
            eprintln!(
                "cluster scene: {} clusters from {} meshes could not be prepared",
                clusters.len(),
                meshes.len()
            );
            return;
        };
        // Optional occlusion: needs one frame to build the pyramid before it culls.
        let occlusion = std::env::var("OM9_JEWELRY_OCCLUSION").is_ok();
        if occlusion {
            renderer.set_occlusion(true);
        }
        renderer.render(&colour_view, camera, width, height).expect("warmup");
        if occlusion {
            // A second warm-up builds the pyramid from the first frame.
            renderer.render(&colour_view, camera, width, height).expect("warmup 2");
        }
        let mut frames = Vec::new();
        for _ in 0..8 {
            let started = Instant::now();
            renderer.render(&colour_view, camera, width, height).expect("frame");
            frames.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        frames.sort_by(f64::total_cmp);
        let visible = renderer.count_visible();
        eprintln!(
            "cluster scene: {} clusters from {} meshes (build {build_seconds:.1}s), resident {} of {} ({} bytes), visible {visible}, occlusion {occlusion}, median frame {:.2} ms",
            clusters.len(),
            meshes.len(),
            renderer.resident_count(),
            clusters.len(),
            renderer.total_bytes(),
            frames[frames.len() / 2]
        );
        return;
    }
    // QEM rows: a triangle budget per view, split evenly over the meshes, at navigation resolution.
    if std::env::var("OM9_JEWELRY_QEM").is_ok() {
        let camera = Camera {
            eye: [0., -260., 210.],
            target: [0., 0., 0.],
            up: [0., 0., 1.],
            projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
            near: 0.1,
            far: 5000.0,
        };
        let (target_w, target_h) = (width / 2, height / 2);
        let colour = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("qem target"),
            size: wgpu::Extent3d { width: target_w, height: target_h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("qem depth"),
            size: wgpu::Extent3d { width: target_w, height: target_h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        if std::env::var("OM9_JEWELRY_QEM_DIAG").is_ok() {
            // Diagnostic: the same code path with no decimation at all, plus a warm-up frame and
            // per-frame timings, separates "the scene handling is slow" from "the geometry is slow".
            for (label, per_mesh) in [("no simplify", usize::MAX), ("83 tris", 83usize)] {
                let started = Instant::now();
                let mut batch = SceneBatch::new(device.clone(), queue.clone());
                let mut triangles = 0usize;
                for mesh in &meshes {
                    let flat = flatten(mesh);
                    let data = om9_render::MeshData::new(flat.positions, flat.indices);
                    let simplified = om9_render::qem::simplify(&data, per_mesh);
                    triangles += simplified.triangle_count();
                    batch.add_static(
                        &simplified,
                        [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]],
                    );
                }
                let build_seconds = started.elapsed().as_secs_f64();
                batch.render_into(&colour_view, &depth_view, camera, target_w, target_h).expect("warmup");
                device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
                let mut frames = Vec::new();
                for _ in 0..6 {
                    let started = Instant::now();
                    batch.render_into(&colour_view, &depth_view, camera, target_w, target_h).expect("frame");
                    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
                    frames.push(started.elapsed().as_secs_f64() * 1000.0);
                }
                eprintln!(
                    "diag: {label:12} triangles={triangles:9} draws={:2} build={build_seconds:5.2}s frames={:?}",
                    batch.stats().draw_calls,
                    frames.iter().map(|value| (value * 100.0).round() / 100.0).collect::<Vec<_>>()
                );
            }
            return;
        }
        for budget in [2_000_000usize, 1_000_000, 400_000] {
            let per_mesh = (budget / meshes.len()).max(4);
            let started = Instant::now();
            let mut batch = SceneBatch::new(device.clone(), queue.clone());
            let mut triangles = 0usize;
            for mesh in &meshes {
                let flat = flatten(mesh);
                let data = om9_render::MeshData::new(flat.positions, flat.indices);
                let simplified = om9_render::qem::simplify(&data, per_mesh);
                triangles += simplified.triangle_count();
                batch.add_static(&simplified, [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
            }
            let simplify_seconds = started.elapsed().as_secs_f64();
            let started = Instant::now();
            let frames = 6;
            for _ in 0..frames {
                batch.render_into(&colour_view, &depth_view, camera, target_w, target_h).expect("frame");
            }
            device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
            let per_frame = started.elapsed().as_secs_f64() / frames as f64 * 1000.0;
            eprintln!(
                "qem: budget={budget:8} per-mesh={per_mesh:3} triangles={triangles:9} draws={:2} simplify={simplify_seconds:5.2}s {per_frame:6.2} ms/frame",
                batch.stats().draw_calls
            );
        }
        return;
    }
    if std::env::var("OM9_JEWELRY_LOD_SWEEP").is_ok() {
        let camera = Camera {
            eye: [0., -260., 210.],
            target: [0., 0., 0.],
            up: [0., 0., 1.],
            projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
            near: 0.1,
            far: 5000.0,
        };
        let distance = 334.0f32;
        let rows = [
            ("idle 1px full", 1.0f32, width, height),
            ("nav 2px half", 2.0f32, width / 2, height / 2),
            ("nav 2px full", 2.0f32, width, height),
        ];
        for (label, error_px, target_w, target_h) in rows {
            let colour = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("lod target"),
                size: wgpu::Extent3d { width: target_w, height: target_h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let depth = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("lod depth"),
                size: wgpu::Extent3d { width: target_w, height: target_h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
            let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
            let mut batch = SceneBatch::new(device.clone(), queue.clone());
            let mut triangles = 0usize;
            let mut cells = Vec::new();
            for mesh in &meshes {
                let flat = flatten(mesh);
                let data = om9_render::MeshData::new(flat.positions, flat.indices);
                let mut min = [f32::MAX; 3];
                let mut max = [f32::MIN; 3];
                for position in &data.positions {
                    for axis in 0..3 {
                        min[axis] = min[axis].min(position[axis]);
                        max[axis] = max[axis].max(position[axis]);
                    }
                }
                let radius = 0.5
                    * ((max[0] - min[0]).powi(2) + (max[1] - min[1]).powi(2) + (max[2] - min[2]).powi(2)).sqrt();
                let cell = om9_render::lod::cell_for_pixel_error(
                    radius,
                    distance,
                    target_h as f32,
                    camera.tan_half_fov().atan() * 2.0,
                    error_px,
                );
                cells.push(cell / radius.max(1e-6));
                let simplified = om9_render::lod::decimate(&data, cell);
                triangles += simplified.triangle_count();
                batch.add_static(&simplified, [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
            }
            let started = Instant::now();
            let frames = 6;
            for _ in 0..frames {
                batch.render_into(&colour_view, &depth_view, camera, target_w, target_h).expect("frame");
            }
            device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
            let per_frame = started.elapsed().as_secs_f64() / frames as f64 * 1000.0;
            let mean_cell = cells.iter().sum::<f32>() / cells.len() as f32;
            eprintln!(
                "lod: {label:14} target={target_w}x{target_h} cell={mean_cell:.3}r triangles={triangles:9} draws={:2} {per_frame:6.2} ms/frame",
                batch.stats().draw_calls
            );
        }
        return;
    }
    let started = Instant::now();
    let mut batch = SceneBatch::new(device.clone(), queue.clone());
    let identity = [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]];
    let static_merge = std::env::var("OM9_JEWELRY_STATIC").is_ok();
    for mesh in &meshes {
        let flat = flatten(mesh);
        let data = om9_render::MeshData::new(flat.positions, flat.indices);
        if static_merge {
            batch.add_static(&data, identity);
        } else {
            batch.add_mesh(&data, identity, [0.85, 0.7, 0.35, 1.0]);
        }
    }
    let upload_seconds = started.elapsed().as_secs_f64();
    let _ = std::fs::remove_file(&path);

    // Four views, each orbiting three degrees per frame and zooming in, like a person inspecting a piece.
    let scene_radius = meshes
        .iter()
        .flat_map(|mesh| mesh.vertices.iter())
        .fold(1.0f64, |radius, point| radius.max(point[0].abs()).max(point[1].abs()));
    let target = [scene_radius as f32 * 0.5, scene_radius as f32 * 0.5, 0.0];
    // Close enough that a ring is several pixels: the scene is metres wide, each ring is ~10 mm, so a
    // whole-scene camera would legitimately cull everything as sub-pixel.
    let distance = scene_radius as f32 * 0.06;
    let mut views = [
        Camera { eye: [target[0] + distance, target[1] - distance, distance], target, up: [0., 0., 1.], projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() }, near: 0.1, far: distance * 10.0 },
        Camera { eye: [target[0] - distance, target[1] - distance, distance * 0.6], target, up: [0., 0., 1.], projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() }, near: 0.1, far: distance * 10.0 },
        Camera { eye: [target[0], target[1], distance * 1.6], target, up: [0., 1., 0.], projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() }, near: 0.1, far: distance * 10.0 },
        Camera { eye: [target[0] + distance * 1.4, target[1], distance * 0.2], target, up: [0., 0., 1.], projection: Projection::Perspective { vertical_fov_rad: 40f32.to_radians() }, near: 0.1, far: distance * 10.0 },
    ];
    let frames = env("OM9_JEWELRY_FRAMES", 12);
    let mut totals = [0.0f64; 4];
    for (index, camera) in views.iter_mut().enumerate() {
        let started = Instant::now();
        for frame in 0..frames {
            camera.eye = orbit(camera.eye, camera.target, 3f32.to_radians());
            if frame % 3 == 0 {
                let zoom = if frame % 6 == 0 { 0.92 } else { 1.08 };
                camera.eye = [
                    camera.target[0] + (camera.eye[0] - camera.target[0]) * zoom,
                    camera.target[1] + (camera.eye[1] - camera.target[1]) * zoom,
                    camera.target[2] + (camera.eye[2] - camera.target[2]) * zoom,
                ];
            }
            batch.render_into(&colour_view, &depth_view, *camera, width, height).expect("frame");
        }
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        totals[index] = started.elapsed().as_secs_f64() / frames as f64 * 1000.0;
    }
    let stats = batch.stats();
    eprintln!(
        "jewelry: rings={rings} stones={stones} ring={ring_u}x{ring_v} file={:.1} MiB write={write_seconds:.2}s import={import_seconds:.2}s upload={upload_seconds:.2}s",
        file_bytes as f64 / (1024.0 * 1024.0)
    );
    eprintln!(
        "jewelry: objects={} visible={} culled={} small={} draws={} per-view ms={:?}",
        stats.objects, stats.visible, stats.culled, stats.culled_small, stats.draw_calls,
        totals.map(|value| (value * 100.0).round() / 100.0)
    );
    assert!(stats.objects == rings + stones);
    // A hard ceiling only to catch a hang; the measured number is the evidence.
    assert!(
        totals.iter().all(|value| *value < 2000.0),
        "a view should still finish, got {totals:?}"
    );
}
