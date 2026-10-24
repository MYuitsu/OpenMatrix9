//! Accuracy evidence on a real imported jewellery project.
//!
//! `OM9_ACCURACY_FIXTURE=<path.om9> cargo test --release -p om9-app --test accuracy_fixture -- --ignored --nocapture`
use om9_archive::Archive;
use om9_core::{budget::Limits, geometry::Geometry};
use om9_render::MeshData;
use std::collections::BTreeSet;

#[test]
#[ignore = "needs an imported jewellery fixture selected by OM9_ACCURACY_FIXTURE"]
fn jewellery_lod_keeps_surface_area() {
    let path = std::env::var_os("OM9_ACCURACY_FIXTURE").expect("OM9_ACCURACY_FIXTURE");
    let restored = Archive::open_restored(
        std::path::Path::new(&path),
        &Limits::default(),
        &BTreeSet::new(),
    )
    .expect("open fixture");
    let mut worst = 0.0f64;
    let mut pieces = 0usize;
    for entity in restored.document.entities.values() {
        let Geometry::Brep(brep) = &entity.geometry else {
            continue;
        };
        let display = &brep.display;
        if display.triangles.len() < 40 {
            continue;
        }
        let mesh = MeshData::new(
            display
                .points
                .iter()
                .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
                .collect(),
            display
                .triangles
                .iter()
                .flat_map(|t| [t[0], t[1], t[2]])
                .collect(),
        );
        let locked = om9_render::qem::feature_locks(&mesh, 30.0);
        let (lod, _) = om9_render::qem::simplify_with_error_locked(
            &mesh,
            mesh.triangle_count() / 2,
            Some(&locked),
        );
        let before = om9_render::qem::surface_area(&mesh);
        let after = om9_render::qem::surface_area(&lod);
        if before > 0.0 {
            let error = ((after - before) / before * 100.0).abs();
            eprintln!(
                "piece tris {} -> {} area error {error:.2}%",
                mesh.triangle_count(),
                lod.triangle_count()
            );
            worst = worst.max(error);
            pieces += 1;
        }
    }
    assert!(pieces > 0, "fixture has no usable BRep");
    assert!(
        worst < 10.0,
        "worst locked-LOD area error {worst:.2}% over {pieces} pieces"
    );
}
#[test]
#[ignore = "needs an imported jewellery fixture and a GPU adapter"]
fn jewellery_lod_matches_the_full_detail_image() {
    use om9_render::{Camera, Projection, Renderer};
    let path = std::env::var_os("OM9_ACCURACY_FIXTURE").expect("OM9_ACCURACY_FIXTURE");
    let restored = Archive::open_restored(
        std::path::Path::new(&path),
        &Limits::default(),
        &BTreeSet::new(),
    )
    .expect("open fixture");
    let Some(mut renderer) = Renderer::new() else {
        eprintln!("no GPU adapter; skip image diff");
        return;
    };
    let mut checked = 0usize;
    let mut worst = 0.0f64;
    for entity in restored.document.entities.values() {
        let Geometry::Brep(brep) = &entity.geometry else {
            continue;
        };
        let display = &brep.display;
        if display.triangles.len() < 400 {
            continue;
        }
        let mesh = MeshData::new(
            display
                .points
                .iter()
                .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
                .collect(),
            display
                .triangles
                .iter()
                .flat_map(|t| [t[0], t[1], t[2]])
                .collect(),
        );
        let locked = om9_render::qem::feature_locks(&mesh, 30.0);
        let (lod, _) = om9_render::qem::simplify_with_error_locked(
            &mesh,
            mesh.triangle_count() * 3 / 4,
            Some(&locked),
        );
        let centre = mesh.positions.iter().fold([0.0f32; 3], |mut c, p| {
            for a in 0..3 {
                c[a] += p[a] / mesh.positions.len() as f32;
            }
            c
        });
        let radius = mesh
            .positions
            .iter()
            .map(|p| {
                ((p[0] - centre[0]).powi(2)
                    + (p[1] - centre[1]).powi(2)
                    + (p[2] - centre[2]).powi(2))
                .sqrt()
            })
            .fold(0.0f32, f32::max)
            .max(1e-3);
        let camera = Camera {
            eye: [centre[0], centre[1], centre[2] + radius * 3.0],
            target: centre,
            up: [0.0, 1.0, 0.0],
            projection: Projection::Perspective {
                vertical_fov_rad: 35f32.to_radians(),
            },
            near: (radius * 0.1).max(0.001),
            far: radius * 10.0,
        };
        let full = renderer
            .render(1, &mesh, camera, 384, 384)
            .expect("full frame");
        let coarse = renderer
            .render(2, &lod, camera, 384, 384)
            .expect("lod frame");
        let mut different = 0usize;
        for (a, b) in full
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(coarse.pixels.as_chunks::<4>().0.iter())
        {
            if (0..3).any(|i| (a[i] as i16 - b[i] as i16).abs() > 8) {
                different += 1;
            }
        }
        let ratio = different as f64 / (384.0 * 384.0) * 100.0;
        eprintln!(
            "image diff {} triangles: {ratio:.2}%",
            mesh.triangle_count()
        );
        worst = worst.max(ratio);
        checked += 1;
    }
    assert!(checked > 0, "fixture has no usable BRep");
    assert!(
        worst < 2.0,
        "worst image difference {worst:.2}% over {checked} pieces"
    );
}
