//! 3DM mesh import: a written archive comes back as OM9 meshes, and many of them reach the GPU.
//!
//! The end-to-end path is the point: openNURBS writes the archive, `om9-3dm` reads it into
//! `om9_core::geometry::Mesh`, `flatten` produces GPU buffers, and `SceneBatch` draws the lot. A mesh
//! round-trip that loses triangles, or a renderer that cannot take 500 imported meshes, would both show
//! up here.
use om9_render::Projection;

use om9_3dm::{ImportBudget, flatten, read_meshes, read_meshes_with, write_fixture};
use om9_render::Camera;
use om9_render::scene::SceneBatch;
use std::time::Instant;

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("om9-3dm-{name}.3dm"))
}

#[test]
fn an_archive_round_trips_into_om9_meshes() {
    let path = temp_path("roundtrip");
    write_fixture(&path, 4).expect("write fixture");
    let meshes = read_meshes(&path).expect("read meshes");
    assert_eq!(meshes.len(), 4, "one OM9 mesh per 3DM mesh object");
    let mut total = 0;
    for mesh in &meshes {
        assert_eq!(mesh.vertices.len(), 8, "a cube keeps its eight vertices");
        assert_eq!(
            mesh.faces.len(),
            12,
            "the twelve triangles survive the round trip"
        );
        assert!(mesh.faces.iter().all(|face| face.len() == 3));
        total += mesh.faces.len();
    }
    assert_eq!(total, 48);
    // Indices are local to their own mesh: a shared global index would read past the vertex array.
    for (index, mesh) in meshes.iter().enumerate() {
        for face in &mesh.faces {
            for corner in face {
                assert!(
                    (*corner as usize) < mesh.vertices.len(),
                    "mesh {index} has corner {corner} outside its {} vertices",
                    mesh.vertices.len()
                );
            }
        }
    }
    // The cubes are translated along X, so the archive's extent must reflect that.
    let max_x = meshes
        .iter()
        .flat_map(|mesh| mesh.vertices.iter())
        .map(|point| point[0])
        .fold(f64::MIN, f64::max);
    assert!(max_x > 3.0, "the fourth cube sits past x = 3, got {max_x}");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_import_rejects_a_missing_archive() {
    let meshes = read_meshes(&temp_path("does-not-exist"));
    assert!(
        meshes.is_err(),
        "a missing file must be an error, not an empty import"
    );
}

#[test]
fn five_hundred_imported_meshes_render_in_one_pass() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let Some(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
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
        label: Some("3dm bench"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");

    let path = temp_path("many");
    let count = 500usize;
    write_fixture(&path, count).expect("write fixture");
    let started = Instant::now();
    let meshes = read_meshes(&path).expect("read meshes");
    let import_seconds = started.elapsed().as_secs_f64();
    assert_eq!(meshes.len(), count);
    let _ = std::fs::remove_file(&path);

    let width = 960u32;
    let height = 540u32;
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("colour"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
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
    for (index, mesh) in meshes.iter().enumerate() {
        let flat = flatten(mesh);
        let x = (index % 25) as f32 - 12.0;
        let y = (index / 25) as f32 - 10.0;
        batch.add_mesh(
            &om9_render::MeshData::new(flat.positions, flat.indices),
            [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [x * 3.2, y * 3.2, 0., 1.],
            ],
            [0.8, 0.55, 0.25, 1.0],
        );
    }
    // Throughput: draw every imported mesh, no culling, so the number is the real cost of the scene.
    batch.set_culling(false);
    let camera = Camera {
        // Far enough back to hold the whole imported grid in view: the earlier camera culled most of it,
        // which is correct behaviour but hides the throughput this test exists to show.
        eye: [0., -260., 210.],
        target: [0., 0., 0.],
        up: [0., 0., 1.],
        projection: Projection::Perspective {
            vertical_fov_rad: 40f32.to_radians(),
        },
        near: 0.1,
        far: 2000.0,
    };
    batch
        .render_into(&colour_view, &depth_view, camera, width, height)
        .expect("warmup");
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .expect("poll");
    let started = Instant::now();
    let frames = 20;
    for _ in 0..frames {
        batch
            .render_into(&colour_view, &depth_view, camera, width, height)
            .expect("frame");
    }
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .expect("poll");
    let per_frame = started.elapsed().as_secs_f64() / frames as f64 * 1000.0;
    let stats = batch.stats();
    eprintln!(
        "imported {count} meshes in {:.1} ms; render {per_frame:.2} ms/frame ({} visible, {} culled, {} draws)",
        import_seconds * 1000.0,
        stats.visible,
        stats.culled,
        stats.draw_calls
    );
    assert!(stats.visible > 0);
    assert!(
        per_frame < 16.0,
        "the imported scene must fit a frame, got {per_frame} ms"
    );
}

#[test]
fn the_budget_is_derived_from_memory_and_rejects_an_oversized_import() {
    // 108 bytes per vertex by construction; 6 corners per vertex for a closed triangle surface.
    let small = ImportBudget::from_memory(108 * 100);
    assert_eq!(small.max_vertices, 100);
    assert_eq!(small.max_corners, 600);
    assert_eq!(small.max_bytes, 108 * 100);

    let document = ImportBudget::document();
    assert_eq!(
        document.max_bytes,
        256 * 1024 * 1024,
        "the document budget is the repository limit"
    );
    assert!(
        document.max_vertices > 2_000_000,
        "256 MiB allows more than two million vertices"
    );
    assert_eq!(document.max_corners, document.max_vertices * 6);
    let sculpt = ImportBudget::sculpt();
    assert!(
        sculpt.max_vertices > document.max_vertices,
        "sculpt asks for a larger budget explicitly"
    );

    // The four-mesh fixture fits the document budget but not a budget of a hundred vertices.
    let path = temp_path("budget");
    write_fixture(&path, 4).expect("write fixture");
    assert!(
        read_meshes(&path).is_ok(),
        "the default budget accepts the fixture"
    );
    let tight = read_meshes_with(&path, ImportBudget::from_memory(108 * 16));
    assert!(tight.is_err(), "a 16-vertex budget must reject 32 vertices");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_file_larger_than_the_budget_is_refused_before_reading() {
    let path = temp_path("oversized");
    write_fixture(&path, 1).expect("write fixture");
    let size = std::fs::metadata(&path).expect("metadata").len();
    // A budget one byte below the file size must be refused outright.
    let budget = ImportBudget {
        max_bytes: size - 1,
        max_vertices: 1_000_000,
        max_corners: 6_000_000,
    };
    assert!(
        read_meshes_with(&path, budget).is_err(),
        "the file byte cap must apply"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_default_budget_takes_three_quarters_of_the_free_physical_memory() {
    let budget = ImportBudget::detect();
    assert!(
        budget.max_bytes >= 64 * 1024 * 1024,
        "the floor keeps small imports possible"
    );
    if let Some(available) = ImportBudget::available_bytes() {
        // Free memory moves between the two reads, so the check is a bracket rather than an equality.
        let ceiling = (available / 4 * 3).max(64 * 1024 * 1024);
        assert!(
            budget.max_bytes <= ceiling + 256 * 1024 * 1024,
            "the budget {} must stay near 75% of the free memory {}",
            budget.max_bytes,
            ceiling
        );
        assert!(budget.max_bytes >= 64 * 1024 * 1024);
    }
    // The vertex count must be exactly what the byte budget allows at 108 bytes per vertex.
    assert_eq!(budget.max_vertices as u64, budget.max_bytes / 108);
    assert_eq!(budget.max_corners, budget.max_vertices * 6);
    // The default is the detected budget, not a fixed number. Free memory moves between two reads, so
    // this is checked by shape (same derivation), not by equality.
    let default = ImportBudget::default();
    assert_eq!(default.max_vertices as u64, default.max_bytes / 108);
    assert!(default.max_bytes >= 64 * 1024 * 1024);
}

#[test]
fn a_refused_import_names_both_limits_for_the_user() {
    let path = temp_path("refused");
    write_fixture(&path, 4).expect("write fixture");
    // Keep the byte cap out of the way so the vertex/corner numbers are what the user sees.
    let geometry_budget = ImportBudget {
        max_bytes: 64 * 1024 * 1024,
        max_vertices: 16,
        max_corners: 96,
    };
    let error = read_meshes_with(&path, geometry_budget).expect_err("must refuse");
    let text = error.to_string();
    assert!(
        text.contains("3DM import refused"),
        "the message must say what happened: {text}"
    );
    assert!(
        text.contains("vertices"),
        "the message must name the vertex limit: {text}"
    );
    assert!(
        text.contains("corners"),
        "the message must name the corner limit: {text}"
    );
    assert!(
        text.contains("32") && text.contains("16"),
        "the message must carry both numbers: {text}"
    );
    let _ = std::fs::remove_file(&path);

    // A file over the byte cap reports bytes rather than geometry.
    write_fixture(&path, 1).expect("write fixture");
    let size = std::fs::metadata(&path).expect("metadata").len();
    let error = read_meshes_with(
        &path,
        ImportBudget {
            max_bytes: size - 1,
            max_vertices: 1_000_000,
            max_corners: 6_000_000,
        },
    )
    .expect_err("must refuse");
    let text = error.to_string();
    assert!(
        text.contains("import budget"),
        "the byte refusal names the budget: {text}"
    );
    let _ = std::fs::remove_file(&path);
}
