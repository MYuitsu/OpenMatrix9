//! M00-L2-02 / L3-03 — numeric invariants asserted from Rust.
//!
//! The expected values are analytic, so a wrong kernel measurement cannot pass. The tessellation check
//! separates the two properties a mesh can have: closed (no boundary edge) and manifold (no edge used by
//! more than two triangles). OCCT's sphere mesh is closed but leaves one non-manifold edge at each pole.

use om9_kernel::{
    box_measures, cylinder_measures, degenerate, fillet_volume, offset_volume, sphere_measures,
    sphere_mesh,
};
use std::f64::consts::PI;

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

#[test]
fn unit_pinned_primitives_match_the_analytic_volume_and_area() {
    let (volume, area) = box_measures(10.0, 10.0, 4.0).expect("box");
    assert!(close(volume, 400.0, 1e-9), "box volume {volume}");
    assert!(close(area, 360.0, 1e-9), "box area {area}");

    let (volume, area) = sphere_measures(5.0).expect("sphere");
    assert!(close(volume, 4.0 / 3.0 * PI * 125.0, 1e-7), "sphere volume {volume}");
    assert!(close(area, 4.0 * PI * 25.0, 1e-7), "sphere area {area}");

    let (volume, area) = cylinder_measures(2.0, 4.0).expect("cylinder");
    assert!(close(volume, PI * 4.0 * 4.0, 1e-9), "cylinder volume {volume}");
    assert!(close(area, 24.0 * PI, 1e-7), "cylinder area {area}");
}

#[test]
fn fillet_and_offset_volumes_match_the_analytic_values() {
    let filleted = fillet_volume(10.0, 10.0, 4.0, 1.0).expect("fillet");
    let expected = 400.0 - (1.0 - PI / 4.0) * 4.0;
    assert!(close(filleted, expected, 1e-6), "fillet volume {filleted} != {expected}");

    let offset = offset_volume(10.0, 10.0, 4.0, -1.0).expect("offset");
    assert!(close(offset, 8.0 * 8.0 * 2.0, 1e-6), "offset volume {offset}");
}

#[test]
fn degenerate_inputs_are_reported_instead_of_crashing() {
    // Cutting a box with a box that only touches it leaves the original body.
    let touching = degenerate(0).expect("touching cut");
    assert!(touching.valid, "a touching boolean must stay valid");
    assert!(close(touching.volume, 400.0, 1e-9), "touching cut volume {}", touching.volume);

    // Offsetting inward past the wall cannot produce a solid, and must say so.
    let past_the_wall = degenerate(1).expect("offset past the wall");
    assert!(!past_the_wall.valid, "an impossible offset must not claim validity");

    // A body at the kernel's resolution is refused with a typed failure.
    let error = degenerate(2).expect_err("a body at the kernel resolution must be refused");
    assert!(
        error.to_string().contains("DomainError") || error.to_string().contains("ConstructionError"),
        "unexpected failure for the tiny body: {error}"
    );
}

#[test]
fn the_mesh_stays_on_the_exact_sphere_and_is_closed() {
    for tolerance in [0.01, 0.001] {
        let report = sphere_mesh(5.0, tolerance).expect("sphere mesh");
        assert!(
            report.max_deviation <= tolerance,
            "tolerance {tolerance}: a node sits {} from the surface",
            report.max_deviation
        );
        assert_eq!(report.boundary_edges, 0, "tolerance {tolerance}: the mesh has a hole");
        assert!(report.closed(), "closed() must follow boundary_edges");
        // Recorded, not asserted away: OCCT closes the pole fan over a duplicated pole node.
        assert_eq!(
            report.non_manifold_edges, 2,
            "tolerance {tolerance}: the pole edges changed, re-check the mesher"
        );
        assert!(report.triangles > 0);
    }
    let coarse = sphere_mesh(5.0, 0.01).expect("coarse");
    let fine = sphere_mesh(5.0, 0.001).expect("fine");
    assert!(
        fine.triangles > coarse.triangles,
        "a tighter deflection must refine the mesh ({} vs {})",
        fine.triangles,
        coarse.triangles
    );
}
