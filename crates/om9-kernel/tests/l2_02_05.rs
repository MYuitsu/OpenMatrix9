// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused OCCT acceptance for `OM9-L2-02-05`: patch, join, offset, fillet/chamfer and
//! normal extrusion through the shared kernel. The portable surface services are covered by
//! `crates/om9-core/tests/surface_ops.rs`; this file proves the exact B-Rep side.
#![cfg(feature = "3dm-brep")]

use om9_core::budget::Limits;
use om9_kernel::{
    creation::{self, DEFAULT_DEFLECTION_MM, Primitive},
    editing,
};

fn limits() -> Limits {
    Limits::default()
}

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

fn rectangle_patch(x0: f64, x1: f64) -> creation::SurfacePatch {
    let boundary = [
        vec![[x0, 0.0, 0.0], [(x0 + x1) / 2.0, 0.0, 0.0], [x1, 0.0, 0.0]],
        vec![[x1, 0.0, 0.0], [x1, 1.0, 0.0], [x1, 2.0, 0.0]],
        vec![[x1, 2.0, 0.0], [(x0 + x1) / 2.0, 2.0, 0.0], [x0, 2.0, 0.0]],
        vec![[x0, 2.0, 0.0], [x0, 1.0, 0.0], [x0, 0.0, 0.0]],
    ];
    creation::surface_from_edges(&boundary, 1e-6, DEFAULT_DEFLECTION_MM, &limits())
        .expect("planar boundary patch")
}

#[test]
fn m03_02_05_patch_and_normal_extrude_are_exact_breps() {
    let patch = rectangle_patch(0.0, 4.0);
    assert!(patch.max_deviation_mm <= 1e-6);
    let measured = creation::measure(&patch.brep, &limits()).expect("patch measure");
    assert!(
        close(measured.area_mm2, 8.0, 1e-5),
        "patch area {}",
        measured.area_mm2
    );
    assert!(!measured.closed, "a single patch is an open sheet");

    // Normal extrusion / thickness through the shared OCCT kernel.
    let solid = editing::thicken(&patch.brep, 0.5, DEFAULT_DEFLECTION_MM, &limits())
        .expect("thicken patch");
    let measured = creation::measure(&solid, &limits()).expect("solid measure");
    assert!(measured.closed, "thickened patch must be closed");
    assert!(
        close(measured.volume_mm3, 8.0 * 0.5, 0.05),
        "thickened volume {}",
        measured.volume_mm3
    );
    assert_eq!(measured.solids, 1);
}

#[test]
fn m03_02_05_offset_fillet_and_chamfer_use_the_shared_kernel() {
    let boxed = creation::primitive(
        Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("box");
    let boxed_volume = creation::measure(&boxed, &limits()).unwrap().volume_mm3;
    let offset =
        editing::offset_shape(&boxed, 1.0, DEFAULT_DEFLECTION_MM, &limits()).expect("offset box");
    let offset_volume = creation::measure(&offset, &limits()).unwrap().volume_mm3;
    assert!(
        offset_volume > boxed_volume,
        "outward offset {offset_volume} <= {boxed_volume}"
    );
    assert!(creation::measure(&offset, &limits()).unwrap().closed);

    let filleted = editing::fillet(&boxed, &[0], 1.0, DEFAULT_DEFLECTION_MM, &limits())
        .expect("fillet box edge");
    let fillet_measures = creation::measure(&filleted, &limits()).unwrap();
    assert!(fillet_measures.closed);
    assert_eq!(fillet_measures.naked_edges, 0);
    assert!(fillet_measures.faces > 6, "fillet adds a face");

    let chamfered = editing::chamfer(&boxed, &[0], 1.0, DEFAULT_DEFLECTION_MM, &limits())
        .expect("chamfer box edge");
    let chamfer_measures = creation::measure(&chamfered, &limits()).unwrap();
    assert!(chamfer_measures.closed);
    assert_eq!(chamfer_measures.naked_edges, 0);
    assert!(chamfer_measures.faces > 6, "chamfer adds a face");
    assert!(
        chamfer_measures.volume_mm3 < boxed_volume,
        "chamfer removes material"
    );
}

#[test]
fn m03_02_05_join_surfaces_sews_two_sheets() {
    let left = rectangle_patch(0.0, 2.0);
    let right = rectangle_patch(2.0, 4.0);
    let joined = editing::sew_all(
        &[left.brep, right.brep],
        1e-6,
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("sew two surface sheets");
    let measured = creation::measure(&joined, &limits()).expect("joined measure");
    assert!(measured.faces >= 2, "joined faces {}", measured.faces);
    assert!(measured.shells >= 1, "joined shells {}", measured.shells);
    assert!(!measured.closed, "two open sheets form an open shell");
    // Both authored boundaries remain in the exact B-Rep.
    assert!(measured.area_mm2 > 8.0 - 1e-6);
}

#[test]
fn m03_02_05_surface_services_refuse_degenerate_bodies() {
    let patch = rectangle_patch(0.0, 4.0);
    assert!(editing::thicken(&patch.brep, 0.0, DEFAULT_DEFLECTION_MM, &limits()).is_err());
    assert!(editing::offset_shape(&patch.brep, 0.0, DEFAULT_DEFLECTION_MM, &limits()).is_err());
    // A gap in the boundary loop must be refused, never bridged silently.
    let open = [
        vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
        vec![[4.0, 0.0, 0.0], [4.0, 2.0, 0.0]],
        vec![[4.0, 2.0, 0.0], [0.0, 2.0, 0.0]],
        vec![[0.0, 2.0, 0.0], [0.0, 0.5, 0.0]],
    ];
    assert!(creation::surface_from_edges(&open, 1e-6, DEFAULT_DEFLECTION_MM, &limits()).is_err());
}

#[test]
fn m03_02_05_kernel_receipt() {
    let patch = rectangle_patch(0.0, 4.0);
    let patch_measure = creation::measure(&patch.brep, &limits()).expect("patch measure");
    let solid = editing::thicken(&patch.brep, 0.5, DEFAULT_DEFLECTION_MM, &limits())
        .expect("thicken patch");
    let solid_measure = creation::measure(&solid, &limits()).expect("solid measure");
    let boxed = creation::primitive(
        Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("box");
    let offset =
        editing::offset_shape(&boxed, 1.0, DEFAULT_DEFLECTION_MM, &limits()).expect("offset box");
    let offset_measure = creation::measure(&offset, &limits()).expect("offset measure");
    let filleted =
        editing::fillet(&boxed, &[0], 1.0, DEFAULT_DEFLECTION_MM, &limits()).expect("fillet");
    let fillet_measure = creation::measure(&filleted, &limits()).expect("fillet measure");
    let joined = editing::sew_all(
        &[
            rectangle_patch(0.0, 2.0).brep,
            rectangle_patch(2.0, 4.0).brep,
        ],
        1e-6,
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("sew sheets");
    let join_measure = creation::measure(&joined, &limits()).expect("join measure");
    let receipt = serde_json::json!({
        "status": "measured",
        "patch_area_mm2": patch_measure.area_mm2,
        "patch_deviation_mm": patch.max_deviation_mm,
        "thicken_volume_mm3": solid_measure.volume_mm3,
        "thicken_closed": solid_measure.closed,
        "offset_volume_mm3": offset_measure.volume_mm3,
        "offset_closed": offset_measure.closed,
        "fillet_faces": fillet_measure.faces,
        "fillet_naked_edges": fillet_measure.naked_edges,
        "join_faces": join_measure.faces,
        "join_shells": join_measure.shells,
    });
    println!("{receipt}");
}

#[test]
fn m03_02_05_kernel_stress_joins_and_offsets_repeatedly() {
    let mut parts = Vec::new();
    for index in 0..10 {
        parts.push(rectangle_patch(index as f64 * 2.0, index as f64 * 2.0 + 2.0).brep);
    }
    let joined =
        editing::sew_all(&parts, 1e-6, DEFAULT_DEFLECTION_MM, &limits()).expect("stress sew");
    let measured = creation::measure(&joined, &limits()).expect("stress join measure");
    assert!(measured.faces >= 10);
    assert!(measured.shells >= 1);
    let mut shape = joined;
    for _ in 0..3 {
        shape = editing::offset_shape(&shape, 0.1, DEFAULT_DEFLECTION_MM, &limits())
            .expect("stress offset");
        let measured = creation::measure(&shape, &limits()).expect("stress offset measure");
        assert!(measured.faces >= 10);
        assert!(measured.volume_mm3.is_finite());
    }
}
