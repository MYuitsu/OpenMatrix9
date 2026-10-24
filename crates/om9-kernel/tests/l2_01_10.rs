// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused acceptance tests for M03 `OM9-L2-01-10` boolean / split / cap / fillet / offset /
//! extract / health operations.
//!
//! Every assertion measures the exact OCCT B-Rep (volume/area/topology/naked edges), never a
//! display mesh. Run one owner at a time, for example:
//!
//! `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 m03_01_10_001 -- --exact`
use om9_core::{brep::Brep, budget::Limits, geometry::Point3};
use om9_kernel::{
    creation::{self, DEFAULT_DEFLECTION_MM, Primitive, Profile},
    editing::{self, BooleanOp, BooleanOutcome, EditingError},
};
use std::f64::consts::PI;

fn limits() -> Limits {
    Limits::default()
}

fn deflection() -> f64 {
    DEFAULT_DEFLECTION_MM
}

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

fn box_shape(dx: f64, dy: f64, dz: f64) -> Brep {
    creation::primitive(
        Primitive::Box {
            x_mm: dx,
            y_mm: dy,
            z_mm: dz,
        },
        deflection(),
        &limits(),
    )
    .expect("box")
}

fn box_at(origin: Point3, dx: f64, dy: f64, dz: f64) -> Brep {
    let profile = Profile::new(vec![
        [origin[0], origin[1], origin[2]],
        [origin[0] + dx, origin[1], origin[2]],
        [origin[0] + dx, origin[1] + dy, origin[2]],
        [origin[0], origin[1] + dy, origin[2]],
    ])
    .expect("box profile");
    creation::extrude(&profile, [0.0, 0.0, dz], true, deflection(), &limits()).expect("box prism")
}

fn rect_face(width: f64, height: f64) -> Brep {
    let profile = Profile::new(vec![
        [0.0, 0.0, 0.0],
        [width, 0.0, 0.0],
        [width, height, 0.0],
        [0.0, height, 0.0],
    ])
    .expect("rectangle profile");
    creation::planar(&profile, deflection(), &limits()).expect("planar face")
}

fn measured(brep: &Brep) -> creation::Measures {
    creation::measure(brep, &limits()).expect("measure")
}

fn vertical_edge(brep: &Brep) -> usize {
    for index in 0..brep.display.edges.len() {
        if let Ok((a, b)) = editing::edge_endpoints(brep, index) {
            let dx = (a[0] - b[0]).abs();
            let dy = (a[1] - b[1]).abs();
            let dz = (a[2] - b[2]).abs();
            if dx < 1.0e-6 && dy < 1.0e-6 && dz > 1.0 {
                return index;
            }
        }
    }
    panic!("no vertical edge in the box");
}

fn face_area(brep: &Brep, index: usize) -> f64 {
    let face = editing::extract_face(brep, index, deflection(), &limits()).expect("extract face");
    measured(&face).area_mm2
}

fn face_min_z(brep: &Brep, index: usize) -> f64 {
    let mut min_z = f64::INFINITY;
    for [a, b, c, face] in &brep.display.triangles {
        if *face as usize != index {
            continue;
        }
        for point in [a, b, c] {
            min_z = min_z.min(brep.display.points[*point as usize][2]);
        }
    }
    min_z
}

fn face_max_z(brep: &Brep, index: usize) -> f64 {
    let mut max_z = f64::NEG_INFINITY;
    for [a, b, c, face] in &brep.display.triangles {
        if *face as usize != index {
            continue;
        }
        for point in [a, b, c] {
            max_z = max_z.max(brep.display.points[*point as usize][2]);
        }
    }
    max_z
}

fn top_face_index(brep: &Brep) -> usize {
    let mut global_max = f64::NEG_INFINITY;
    for index in 0..brep.display.faces.len() {
        global_max = global_max.max(face_max_z(brep, index));
    }
    // The top face is the horizontal face whose every vertex is at the global maximum z; a side
    // face also reaches that z but its minimum z is lower.
    for index in 0..brep.display.faces.len() {
        if close(face_max_z(brep, index), global_max, 1e-9)
            && close(face_min_z(brep, index), global_max, 1e-9)
        {
            return index;
        }
    }
    panic!("no horizontal top face");
}

/* ------------------------- 001 union ------------------------- */

#[test]
fn m03_01_10_001() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let union = editing::boolean(&a, &b, BooleanOp::Union, deflection(), &limits).expect("union");
    let body = union.body().expect("union body");
    let union_measures = measured(body);
    assert!(
        close(union_measures.volume_mm3, 1500.0, 1e-6),
        "union volume {}",
        union_measures.volume_mm3
    );
    assert_eq!(union_measures.solids, 1);
    assert!(union_measures.closed);
    assert_eq!(union_measures.naked_edges, 0);

    // Disjoint union is a compound of two solids; its volume is the sum and no phantom body exists.
    let far = box_at([50.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let disjoint = editing::boolean(&a, &far, BooleanOp::Union, deflection(), &limits)
        .expect("disjoint union")
        .body()
        .cloned()
        .expect("disjoint body");
    let disjoint_measures = measured(&disjoint);
    assert!(close(disjoint_measures.volume_mm3, 2000.0, 1e-6));
    assert_eq!(disjoint_measures.solids, 2);

    // Negative: an open shell is not a boolean operand; a tampered source is refused.
    let open = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
        ],
        [0.0, 0.0, 10.0],
        deflection(),
        &limits,
    )
    .expect("open shell");
    assert!(matches!(
        editing::boolean(&open, &a, BooleanOp::Union, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let mut json = serde_json::to_value(&a).expect("serialize");
    json["source"] = serde_json::Value::String("not a brep".into());
    let tampered: Brep = serde_json::from_value(json).expect("deserialize");
    assert!(matches!(
        editing::boolean(&tampered, &a, BooleanOp::Union, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 002 difference ------------------------- */

#[test]
fn m03_01_10_002() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let cut = editing::boolean(&a, &b, BooleanOp::Difference, deflection(), &limits)
        .expect("difference")
        .body()
        .cloned()
        .expect("difference body");
    let cut_measures = measured(&cut);
    assert!(
        close(cut_measures.volume_mm3, 500.0, 1e-6),
        "difference volume {}",
        cut_measures.volume_mm3
    );
    assert_eq!(cut_measures.solids, 1);
    assert!(cut_measures.closed);
    // The cut face at x = 5 is part of the result: the box's extent must stop there.
    assert!(close(cut_measures.bounds_mm[3], 5.0, 1e-6));

    // A tool outside the argument leaves the argument unchanged (explicit, not a phantom body).
    let outside = box_at([50.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let unchanged = editing::boolean(&a, &outside, BooleanOp::Difference, deflection(), &limits)
        .expect("outside difference")
        .body()
        .cloned()
        .expect("unchanged body");
    assert!(close(measured(&unchanged).volume_mm3, 1000.0, 1e-6));

    let mut json = serde_json::to_value(&a).expect("serialize");
    json["source"] = serde_json::Value::String("not a brep".into());
    let tampered: Brep = serde_json::from_value(json).expect("deserialize");
    assert!(matches!(
        editing::boolean(&a, &tampered, BooleanOp::Difference, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 003 intersection ------------------------- */

#[test]
fn m03_01_10_003() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let common = editing::boolean(&a, &b, BooleanOp::Intersection, deflection(), &limits)
        .expect("intersection")
        .body()
        .cloned()
        .expect("intersection body");
    let common_measures = measured(&common);
    assert!(
        close(common_measures.volume_mm3, 500.0, 1e-6),
        "intersection volume {}",
        common_measures.volume_mm3
    );
    assert_eq!(common_measures.solids, 1);

    // Disjoint solids have an empty intersection: a typed empty outcome, never a phantom body.
    let far = box_at([50.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    assert_eq!(
        editing::boolean(&a, &far, BooleanOp::Intersection, deflection(), &limits)
            .expect("empty intersection"),
        BooleanOutcome::Empty
    );

    // Borderline coplanar contact must not silently produce a wrong solid body.
    let touching = box_at([10.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let contact = editing::boolean(
        &a,
        &touching,
        BooleanOp::Intersection,
        deflection(),
        &limits,
    )
    .expect("contact intersection");
    match contact {
        BooleanOutcome::Empty => {}
        BooleanOutcome::Body(body) => {
            let measures = measured(&body);
            assert!(
                measures.volume_mm3 <= 1e-9,
                "coplanar contact produced volume {}",
                measures.volume_mm3
            );
            assert_eq!(measures.solids, 0);
        }
    }
}

/* ------------------------- 004 split ------------------------- */

#[test]
fn m03_01_10_004() {
    let limits = limits();
    let face = rect_face(10.0, 20.0);
    let original = measured(&face);
    let split = editing::split_face(
        &face,
        &[[5.0, -5.0, 0.0], [5.0, 25.0, 0.0]],
        deflection(),
        &limits,
    )
    .expect("split face");
    assert_eq!(split.pieces, 2);
    let split_measures = measured(&split.brep);
    assert_eq!(split_measures.faces, 2);
    assert!(
        close(split_measures.area_mm2, original.area_mm2, 1e-6),
        "split area {} vs {}",
        split_measures.area_mm2,
        original.area_mm2
    );
    // Each piece is exactly half the original face.
    for index in 0..split_measures.faces {
        assert!(close(face_area(&split.brep, index), 100.0, 1e-6));
    }

    // Negative: a trim that misses the face, a repeated point, and a non-face input.
    assert!(matches!(
        editing::split_face(
            &face,
            &[[50.0, -5.0, 0.0], [50.0, 25.0, 0.0]],
            deflection(),
            &limits
        ),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::split_face(
            &face,
            &[[1.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
            deflection(),
            &limits
        ),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::split_face(
            &box_shape(10.0, 10.0, 10.0),
            &[[5.0, -5.0, 0.0], [5.0, 25.0, 0.0]],
            deflection(),
            &limits
        ),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 005 cap ------------------------- */

#[test]
fn m03_01_10_005() {
    let limits = limits();
    let boxed = box_shape(10.0, 10.0, 10.0);
    let top = top_face_index(&boxed);
    let open = editing::remove_face(&boxed, top, deflection(), &limits).expect("open shell");
    let open_measures = measured(&open);
    assert_eq!(open_measures.naked_edges, 4);
    assert!(!open_measures.closed);

    let capped = editing::cap_planar(&open, 1.0e-6, deflection(), &limits).expect("cap");
    let capped_measures = measured(&capped);
    assert!(capped_measures.closed);
    assert_eq!(capped_measures.solids, 1);
    assert_eq!(capped_measures.naked_edges, 0);
    assert!(
        close(capped_measures.volume_mm3, 1000.0, 1e-6),
        "capped volume {}",
        capped_measures.volume_mm3
    );

    // Negative: capping an already closed body and capping a non-planar naked loop.
    assert!(matches!(
        editing::cap_planar(&boxed, 1.0e-6, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let skew = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 3.0],
            [0.0, 10.0, 0.0],
        ],
        [0.0, 0.0, 10.0],
        deflection(),
        &limits,
    )
    .expect("skewed open shell");
    assert!(editing::cap_planar(&skew, 1.0e-6, deflection(), &limits).is_err());
}

/* ------------------------- 006 fillet and chamfer ------------------------- */

#[test]
fn m03_01_10_006() {
    let limits = limits();
    let boxed = box_shape(10.0, 10.0, 10.0);
    let edge = vertical_edge(&boxed);

    let filleted =
        editing::fillet(&boxed, &[edge], 1.0, deflection(), &limits).expect("fillet edge");
    let fillet_measures = measured(&filleted);
    let expected = 1000.0 - (1.0 - PI / 4.0) * 10.0;
    assert!(
        close(fillet_measures.volume_mm3, expected, 1e-5),
        "fillet volume {} != {expected}",
        fillet_measures.volume_mm3
    );
    assert!(fillet_measures.closed);
    assert_eq!(fillet_measures.naked_edges, 0);
    assert!(fillet_measures.faces > 6, "the fillet adds a face");
    // The unaffected extent is preserved.
    assert!(close(fillet_measures.bounds_mm[3], 10.0, 1e-6));

    let chamfered =
        editing::chamfer(&boxed, &[edge], 1.0, deflection(), &limits).expect("chamfer edge");
    let chamfer_measures = measured(&chamfered);
    assert!(
        close(chamfer_measures.volume_mm3, 995.0, 1e-5),
        "chamfer volume {}",
        chamfer_measures.volume_mm3
    );
    assert!(chamfer_measures.closed);

    // Negative: oversized radius fails in the kernel and leaves the input untouched; zero radius
    // and an unknown edge index are refused before the kernel.
    assert!(
        editing::fillet(&boxed, &[edge], 1000.0, deflection(), &limits).is_err(),
        "an oversized fillet radius must fail"
    );
    assert!(matches!(
        editing::fillet(&boxed, &[edge], 0.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::fillet(&boxed, &[9999], 1.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let open = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
        ],
        [0.0, 0.0, 10.0],
        deflection(),
        &limits,
    )
    .expect("open shell");
    assert!(matches!(
        editing::fillet(&open, &[0], 1.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    // The original body is unchanged by a refused edit.
    assert!(close(measured(&boxed).volume_mm3, 1000.0, 1e-9));
}

/* ------------------------- 007 offset and thicken ------------------------- */

#[test]
fn m03_01_10_007() {
    let limits = limits();
    let face = rect_face(10.0, 20.0);
    let thick = editing::thicken(&face, 2.0, deflection(), &limits).expect("thicken");
    let thick_measures = measured(&thick);
    assert!(
        close(thick_measures.volume_mm3, 400.0, 1e-5),
        "thicken volume {}",
        thick_measures.volume_mm3
    );
    assert!(thick_measures.closed);
    assert_eq!(thick_measures.solids, 1);
    assert!(
        close(thick_measures.size_mm()[2], 2.0, 1e-6),
        "thicken size {}",
        thick_measures.size_mm()[2]
    );

    let offset = editing::offset_shape(&face, 2.0, deflection(), &limits).expect("offset face");
    let offset_measures = measured(&offset);
    assert_eq!(offset_measures.faces, 1);
    assert!(close(offset_measures.area_mm2, 200.0, 1e-6));
    assert!(
        close(offset_measures.bounds_mm[2].abs(), 2.0, 1e-6),
        "offset plane z {}",
        offset_measures.bounds_mm[2]
    );

    // Negative: zero thickness/offset and thickening a solid.
    assert!(matches!(
        editing::thicken(&face, 0.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::offset_shape(&face, 0.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::thicken(&box_shape(10.0, 10.0, 10.0), 2.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 008 extract, unjoin, rejoin ------------------------- */

#[test]
fn m03_01_10_008() {
    let limits = limits();
    let boxed = box_shape(10.0, 20.0, 30.0);
    let before = measured(&boxed);

    // Pick a 10×20 face (area 600 mm²) by exact extraction, then unjoin and rejoin it.
    let mut selected = None;
    for index in 0..before.faces {
        if close(face_area(&boxed, index), 600.0, 1e-6) {
            selected = Some(index);
            break;
        }
    }
    let index = selected.expect("10×20 face");
    let face = editing::extract_face(&boxed, index, deflection(), &limits).expect("extract");
    let face_measures = measured(&face);
    assert_eq!(face_measures.faces, 1);
    assert!(close(face_measures.area_mm2, 600.0, 1e-6));
    // Extract never mutates the source body.
    assert!(close(measured(&boxed).volume_mm3, before.volume_mm3, 1e-9));

    let open = editing::remove_face(&boxed, index, deflection(), &limits).expect("unjoin");
    let open_measures = measured(&open);
    assert_eq!(open_measures.faces, 5);
    assert_eq!(open_measures.naked_edges, 4);
    assert!(!open_measures.closed);

    let rejoined = editing::sew(&open, &face, 1.0e-6, deflection(), &limits).expect("rejoin");
    let joined_measures = measured(&rejoined);
    assert!(joined_measures.closed);
    assert_eq!(joined_measures.faces, 6);
    assert!(close(joined_measures.volume_mm3, 6000.0, 1e-6));

    // Negative: unknown index; two faces from unrelated places stay an open join, not a solid.
    assert!(matches!(
        editing::extract_face(&boxed, 9999, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::remove_face(&boxed, 9999, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let other = rect_face(1.0, 1.0);
    let distant = editing::sew(&face, &other, 1.0e-6, deflection(), &limits).expect("open join");
    let distant_measures = measured(&distant);
    assert!(!distant_measures.closed);
    assert_eq!(distant_measures.solids, 0);
}

/* ------------------------- 009 health ------------------------- */

#[test]
fn m03_01_10_009() {
    let limits = limits();
    let boxed = box_shape(10.0, 10.0, 10.0);
    let closed = editing::geometry_health(&boxed, &limits).expect("closed health");
    assert!(closed.valid);
    assert!(closed.closed);
    assert_eq!(closed.naked_edges, 0);
    assert_eq!(closed.solids, 1);
    assert_eq!(closed.faces, 6);
    assert_eq!(closed.source_sha256, boxed.display.source_sha256);
    assert!(closed.is_solid());

    let top = top_face_index(&boxed);
    let open = editing::remove_face(&boxed, top, deflection(), &limits).expect("open shell");
    let open_health = editing::geometry_health(&open, &limits).expect("open health");
    assert!(open_health.valid);
    assert!(!open_health.closed);
    assert_eq!(open_health.naked_edges, 4);
    assert_eq!(open_health.faces, 5);
    assert_eq!(open_health.solids, 0);
    assert!(!open_health.is_solid());

    // The count is exact topology: removing one face adds exactly its four boundary edges, no
    // matter how the face is tessellated (a mesh boundary is never consulted).
    assert_eq!(open_health.naked_edges, closed.naked_edges + 4);
    let tessellated = creation::measure(&boxed, &limits).expect("tessellated measure");
    assert_eq!(tessellated.naked_edges, 0);
}

/* ------------------------- regression and stress ------------------------- */

#[test]
fn m03_01_10_regression_boolean_identity_and_round_trip() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let union = editing::boolean(&a, &b, BooleanOp::Union, deflection(), &limits)
        .expect("union")
        .body()
        .cloned()
        .expect("body");
    let common = editing::boolean(&a, &b, BooleanOp::Intersection, deflection(), &limits)
        .expect("common")
        .body()
        .cloned()
        .expect("body");
    let union_volume = measured(&union).volume_mm3;
    let common_volume = measured(&common).volume_mm3;
    // Inclusion–exclusion: |A∪B| + |A∩B| = |A| + |B|.
    assert!(close(union_volume + common_volume, 2000.0, 1e-6));
    // Every result reopens from its canonical source to the same measured solid.
    for brep in [&union, &common] {
        let reopened =
            om9_kernel::brep::read_brep(brep.source(), deflection(), &limits).expect("reopen");
        assert!(close(
            measured(&reopened).volume_mm3,
            measured(brep).volume_mm3,
            1e-6
        ));
    }
}

#[test]
fn m03_01_10_stress_union_chain_and_fillet_family() {
    let limits = limits();
    // A chain of 20 unit cubes overlapping by 0.5 mm: the union volume is 20 − 19×0.5 = 10.5.
    let mut chain = box_at([0.0, 0.0, 0.0], 1.0, 1.0, 1.0);
    for step in 1..20 {
        let next = box_at([step as f64 * 0.5, 0.0, 0.0], 1.0, 1.0, 1.0);
        chain = editing::boolean(&chain, &next, BooleanOp::Union, deflection(), &limits)
            .expect("chain union")
            .body()
            .cloned()
            .expect("chain body");
    }
    let chain_measures = measured(&chain);
    assert!(
        close(chain_measures.volume_mm3, 10.5, 1e-6),
        "chain volume {}",
        chain_measures.volume_mm3
    );
    assert_eq!(chain_measures.solids, 1);

    // Fillet radius family: volume decreases monotonically and every result stays valid.
    let boxed = box_shape(10.0, 10.0, 10.0);
    let edge = vertical_edge(&boxed);
    let mut previous = measured(&boxed).volume_mm3;
    for radius in [0.5, 1.0, 1.5, 2.0] {
        let filleted =
            editing::fillet(&boxed, &[edge], radius, deflection(), &limits).expect("fillet step");
        let volume = measured(&filleted).volume_mm3;
        assert!(volume < previous, "radius {radius}: {volume} !< {previous}");
        assert_eq!(measured(&filleted).naked_edges, 0);
        previous = volume;
    }
}

/* ------------------------- measured receipt ------------------------- */

/// Prints the measured oracle for every L3 fixture as one JSON object. The same test asserts the
/// values, so a printed line is also a pass.
#[test]
fn m03_01_10_receipt() {
    let limits = limits();
    let mut report = String::from("{\"measured\":{");
    let mut push = |name: &str, value: f64| {
        report.push_str(&format!("\"{name}\":{value:.12},"));
    };

    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let union = editing::boolean(&a, &b, BooleanOp::Union, deflection(), &limits)
        .expect("union")
        .body()
        .cloned()
        .expect("body");
    let union_volume = measured(&union).volume_mm3;
    push("union_volume_mm3", union_volume);
    assert!(close(union_volume, 1500.0, 1e-6));
    let difference = editing::boolean(&a, &b, BooleanOp::Difference, deflection(), &limits)
        .expect("difference")
        .body()
        .cloned()
        .expect("body");
    let difference_volume = measured(&difference).volume_mm3;
    push("difference_volume_mm3", difference_volume);
    assert!(close(difference_volume, 500.0, 1e-6));
    let intersection = editing::boolean(&a, &b, BooleanOp::Intersection, deflection(), &limits)
        .expect("intersection")
        .body()
        .cloned()
        .expect("body");
    let intersection_volume = measured(&intersection).volume_mm3;
    push("intersection_volume_mm3", intersection_volume);
    assert!(close(intersection_volume, 500.0, 1e-6));

    let face = rect_face(10.0, 20.0);
    let split = editing::split_face(
        &face,
        &[[5.0, -5.0, 0.0], [5.0, 25.0, 0.0]],
        deflection(),
        &limits,
    )
    .expect("split");
    let split_area = measured(&split.brep).area_mm2;
    push("split_area_mm2", split_area);
    assert!(close(split_area, 200.0, 1e-6));

    let boxed = box_shape(10.0, 10.0, 10.0);
    let open = editing::remove_face(&boxed, top_face_index(&boxed), deflection(), &limits)
        .expect("open shell");
    let capped = editing::cap_planar(&open, 1.0e-6, deflection(), &limits).expect("cap");
    let capped_volume = measured(&capped).volume_mm3;
    push("capped_volume_mm3", capped_volume);
    assert!(close(capped_volume, 1000.0, 1e-6));

    let edge = vertical_edge(&boxed);
    let filleted = editing::fillet(&boxed, &[edge], 1.0, deflection(), &limits).expect("fillet");
    let fillet_volume = measured(&filleted).volume_mm3;
    push("fillet_one_edge_volume_mm3", fillet_volume);
    assert!(close(fillet_volume, 1000.0 - (1.0 - PI / 4.0) * 10.0, 1e-5));
    let chamfered = editing::chamfer(&boxed, &[edge], 1.0, deflection(), &limits).expect("chamfer");
    let chamfer_volume = measured(&chamfered).volume_mm3;
    push("chamfer_one_edge_volume_mm3", chamfer_volume);
    assert!(close(chamfer_volume, 995.0, 1e-5));

    let thick = editing::thicken(&face, 2.0, deflection(), &limits).expect("thicken");
    let thick_volume = measured(&thick).volume_mm3;
    push("thicken_volume_mm3", thick_volume);
    assert!(close(thick_volume, 400.0, 1e-5));
    let offset = editing::offset_shape(&face, 2.0, deflection(), &limits).expect("offset");
    let offset_area = measured(&offset).area_mm2;
    push("offset_area_mm2", offset_area);
    assert!(close(offset_area, 200.0, 1e-6));

    let boxed_3d = box_shape(10.0, 20.0, 30.0);
    let mut selected = 0;
    for index in 0..measured(&boxed_3d).faces {
        if close(face_area(&boxed_3d, index), 600.0, 1e-6) {
            selected = index;
            break;
        }
    }
    let extracted =
        editing::extract_face(&boxed_3d, selected, deflection(), &limits).expect("extract");
    let extracted_area = measured(&extracted).area_mm2;
    push("extracted_face_area_mm2", extracted_area);
    assert!(close(extracted_area, 600.0, 1e-6));

    let health = editing::geometry_health(&boxed, &limits).expect("health");
    push("closed_naked_edges", health.naked_edges as f64);
    assert_eq!(health.naked_edges, 0);
    let open_health = editing::geometry_health(&open, &limits).expect("open health");
    push("open_naked_edges", open_health.naked_edges as f64);
    assert_eq!(open_health.naked_edges, 4);

    report.pop();
    report.push_str("}}");
    println!("{report}");
}
