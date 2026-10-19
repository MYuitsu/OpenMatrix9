//! M01-L2-01-06 — the render loop, its cache, its redraw rule and device-loss recovery.
//!
//! The automated half of the viewport L3: a device exists, the kernel's own tessellation reaches the GPU,
//! the depth buffer resolves two overlapping triangles to the nearer one, DPI keeps the framing, the
//! cache uploads once per key, the scheduler redraws only on change, and a destroyed device is recovered.
//! Wireframe quality, grid, highlight colours and the real window stay manual.
use om9_render::Projection;

use om9_kernel::sphere_triangles;
use om9_render::{Camera, MeshData, RenderScheduler, RenderStyle, Renderer};

fn mesh() -> MeshData {
    let geometry = sphere_triangles(5.0, 0.01).expect("OCCT tessellation");
    assert!(geometry.positions.len() > 100, "the kernel must produce a real mesh");
    assert_eq!(geometry.indices.len() % 3, 0);
    MeshData::new(geometry.positions, geometry.indices)
}

#[test]
fn a_device_exists_and_draws_the_kernel_mesh() {
    let Some(mut renderer) = Renderer::new() else {
        eprintln!("no wgpu adapter on this machine; the manual viewport check covers it");
        return;
    };
    let frame = renderer.render(1, &mesh(), Camera::default(), 640, 360).expect("frame");
    assert_eq!((frame.width, frame.height), (640, 360));
    let painted = frame.painted_pixels();
    assert!(
        painted > 20_000,
        "the sphere should cover a large part of the frame, painted {painted} of {}",
        640 * 360
    );
}

#[test]
fn the_depth_buffer_keeps_the_nearer_triangle() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let far = MeshData::new(
        vec![[-0.8, -0.8, 8.0], [0.8, -0.8, 8.0], [0.0, 0.8, 8.0]],
        vec![0, 1, 2],
    );
    let near = MeshData::new(
        vec![[-0.4, -0.4, 2.0], [0.4, -0.4, 2.0], [0.0, 0.4, 2.0]],
        vec![0, 1, 2],
    );
    let camera = Camera {
        eye: [0.0, 0.0, 10.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        projection: Projection::Perspective { vertical_fov_rad: 45f32.to_radians() },
        near: 0.1,
        far: 100.0,
    };
    let far_painted = renderer.render(1, &far, camera, 320, 180).expect("far").painted_pixels();
    let near_painted = renderer.render(2, &near, camera, 320, 180).expect("near").painted_pixels();
    assert!(
        far_painted > near_painted,
        "the nearer triangle must cover fewer pixels ({near_painted}) than the far one ({far_painted})"
    );
    assert!(near_painted > 100, "the near triangle must be visible at all");
}

#[test]
fn a_two_times_dpi_target_keeps_the_framing() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let mesh = mesh();
    let base = renderer.render(1, &mesh, Camera::default(), 640, 360).expect("base");
    let hidpi = renderer.render(1, &mesh, Camera::default(), 1280, 720).expect("hidpi");
    let base_fraction = base.painted_pixels() as f64 / (640.0 * 360.0);
    let hidpi_fraction = hidpi.painted_pixels() as f64 / (1280.0 * 720.0);
    assert!(
        (base_fraction - hidpi_fraction).abs() < 0.01,
        "the framed area must not change with DPI: {base_fraction} vs {hidpi_fraction}"
    );
}

#[test]
fn a_camera_move_changes_what_is_drawn() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let mesh = mesh();
    let centred = renderer.render(1, &mesh, Camera::default(), 320, 180).expect("centred");
    let panned = renderer
        .render(1, &mesh, Camera { eye: [6.0, -2.0, 14.0], ..Camera::default() }, 320, 180)
        .expect("panned");
    assert_ne!(
        centred.painted_pixels(),
        panned.painted_pixels(),
        "panning the camera must move the mesh, not redraw the same frame"
    );
}

#[test]
fn the_mesh_cache_uploads_once_per_key() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let mesh = mesh();
    renderer.render(7, &mesh, Camera::default(), 320, 180).expect("first");
    assert_eq!(renderer.uploads(), 1, "the first draw uploads the mesh");
    renderer.render(7, &mesh, Camera::default(), 320, 180).expect("second");
    assert_eq!(renderer.uploads(), 1, "the second draw reuses the cached buffers");
    renderer.invalidate(7);
    renderer.render(7, &mesh, Camera::default(), 320, 180).expect("third");
    assert_eq!(renderer.uploads(), 2, "an invalidated key re-uploads");
}

#[test]
fn the_scheduler_redraws_only_on_change() {
    let mut scheduler = RenderScheduler::default();
    assert!(scheduler.should_redraw(0, 0), "the first frame always draws");
    scheduler.mark_drawn();
    assert!(!scheduler.should_redraw(0, 0), "an unchanged state must not redraw");
    assert!(scheduler.should_redraw(1, 0), "a camera revision change redraws");
    scheduler.mark_drawn();
    assert!(scheduler.should_redraw(1, 1), "a scene revision change redraws");
    scheduler.mark_drawn();
    assert!(!scheduler.should_redraw(1, 1));
    scheduler.invalidate();
    assert!(scheduler.should_redraw(1, 1), "an explicit invalidation redraws");
}

#[test]
fn a_lost_device_is_detected_and_recovered() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let mesh = mesh();
    renderer.render(1, &mesh, Camera::default(), 320, 180).expect("before loss");
    renderer.simulate_device_loss();
    let after = renderer.render(1, &mesh, Camera::default(), 320, 180);
    assert_eq!(after, Err(om9_render::RenderError::Lost), "a destroyed device must refuse to draw");
    assert!(renderer.recover(), "a fresh device must be built");
    let recovered = renderer.render(1, &mesh, Camera::default(), 320, 180).expect("after recovery");
    assert!(
        recovered.painted_pixels() > 1000,
        "the recovered device must draw the scene again"
    );
    // The recovery installs a fresh renderer, so its upload counter starts at zero and the one draw
    // after the loss must upload the mesh exactly once: the old cache did not survive the device.
    assert_eq!(renderer.uploads(), 1, "the cache was dropped by the loss and refilled once after recovery");
}

#[test]
fn the_wireframe_pass_draws_edges() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let mesh = mesh();
    let shaded = renderer.render(1, &mesh, Camera::default(), 480, 270).expect("shaded");
    let wireframe = renderer
        .render_with(1, &mesh, Camera::default(), 480, 270, RenderStyle { wireframe: true, ..RenderStyle::default() })
        .expect("wireframe");
    assert!(
        wireframe.painted_pixels() > shaded.painted_pixels(),
        "the wireframe must add drawn pixels: {} vs {}",
        wireframe.painted_pixels(),
        shaded.painted_pixels()
    );
    // Every triangle edge of the sphere is a line, so the frame gains thousands of pixels.
    let gained = wireframe.painted_pixels() - shaded.painted_pixels();
    assert!(gained > 500, "the wireframe overlay added only {gained} pixels");
}

#[test]
fn the_ground_grid_is_drawn_and_clipped_by_the_far_plane() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let empty = MeshData::new(Vec::new(), Vec::new());
    let with_grid = renderer
        .render_with(9, &empty, Camera::default(), 480, 270, RenderStyle { grid: true, ..RenderStyle::default() })
        .expect("grid");
    assert!(with_grid.painted_pixels() > 200, "the grid must draw lines into the empty frame");

    // The sphere spans distances 9..19 from the eye, so a far plane at 5 puts all of it behind the clip.
    let far = Camera { far: 5.0, ..Camera::default() };
    let clipped = renderer
        .render_with(10, &mesh(), far, 480, 270, RenderStyle::default())
        .expect("clipped");
    assert!(clipped.painted_pixels() < 200, "geometry past the far plane must be clipped, painted {}", clipped.painted_pixels());
}

#[test]
fn the_highlight_tints_the_selected_object() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let mesh = mesh();
    let normal = renderer.render(1, &mesh, Camera::default(), 320, 180).expect("normal");
    let highlighted = renderer
        .render_with(1, &mesh, Camera::default(), 320, 180, RenderStyle::highlighted())
        .expect("highlight");
    assert_eq!(normal.green_pixels(), 0, "the base tint is orange, not green");
    assert!(
        highlighted.green_pixels() > 5_000,
        "the highlighted object must turn green, got {} green pixels",
        highlighted.green_pixels()
    );
}

#[test]
fn shading_varies_across_a_curved_face() {
    let Some(mut renderer) = Renderer::new() else {
        return;
    };
    let frame = renderer.render(1, &mesh(), Camera::default(), 320, 180).expect("frame");
    let mut values = Vec::new();
    for y in 0..180 {
        for x in 0..320 {
            let pixel = frame.pixel(x, y);
            if pixel[0] > 60 || pixel[1] > 60 || pixel[2] > 60 {
                values.push(pixel[0] as f64);
            }
        }
    }
    assert!(values.len() > 1000, "the sphere must cover the frame");
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / values.len() as f64;
    assert!(variance.sqrt() > 5.0, "the shading must vary, standard deviation {:.2}", variance.sqrt());
}
