//! M01-L2-01-05 — the seven B-Rep kernel capabilities, asserted from Rust.
//!
//! Every expected value is analytic or a geometric invariant, so a wrong kernel result cannot pass:
//! topology counts of the three primitives, a transform that must move the box without changing its
//! volume, a rational NURBS quarter circle evaluated at a known parameter, a sphere normal that must be
//! the unit radial direction, the two face hits of a line through a box, the tessellation deviation and
//! the reference fixtures.

use om9_kernel::{
    box_measures, cylinder_measures, curve_point_and_derivative, line_face_hits, sphere_measures,
    sphere_mesh, surface_point_and_normal, topology, transform_box,
};
use std::f64::consts::PI;

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

#[test]
fn face_wire_edge_topology_is_exact() {
    let box_topology = topology(0).expect("box topology");
    assert_eq!(
        (box_topology.faces, box_topology.edges, box_topology.vertices, box_topology.wires),
        (6, 12, 8, 6),
        "a box has 6 faces, 12 edges, 8 vertices and 6 wires (unique, not per-face occurrences)"
    );
    assert!(box_topology.valid);

    let sphere = topology(1).expect("sphere topology");
    assert_eq!(sphere.faces, 1, "a sphere is one face");
    assert_eq!(sphere.wires, 1, "a sphere has one wire");
    assert!(sphere.valid);

    let cylinder = topology(2).expect("cylinder topology");
    assert_eq!(cylinder.faces, 3, "a cylinder has two caps and one lateral face");
    assert_eq!(cylinder.wires, 3);
    assert!(cylinder.valid);
}

#[test]
fn a_transform_moves_the_box_and_keeps_its_volume() {
    let translated = transform_box(0.0, 0.0, 5.0, 0.0, 1.0).expect("translation");
    assert!(translated.valid);
    assert!(close(translated.min_z, 5.0, 1e-6));
    assert!(close(translated.max_z, 9.0, 1e-6));
    assert!(close(translated.volume, 400.0, 1e-6));

    let rotated = transform_box(0.0, 0.0, 0.0, 90.0, 1.0).expect("rotation");
    // A 10x10 box is symmetric under a 90 degree turn, so the bounds must stay the same.
    assert!(close(rotated.min_x, 0.0, 1e-6) || close(rotated.min_x, -10.0, 1e-6));
    assert!(close(rotated.volume, 400.0, 1e-6));

    let scaled = transform_box(0.0, 0.0, 0.0, 0.0, 2.0).expect("scale");
    assert!(close(scaled.volume, 3200.0, 1e-5), "a 2x scale cubes the volume");
}

#[test]
fn the_rational_nurbs_quarter_circle_evaluates_exactly() {
    let (point, derivative) = curve_point_and_derivative(0.5).expect("curve");
    let expected = 5.0 / 2.0_f64.sqrt();
    assert!(close(point[0], expected, 1e-9), "x {} != {expected}", point[0]);
    assert!(close(point[1], expected, 1e-9), "y {} != {expected}", point[1]);
    assert!(close(point[2], 0.0, 1e-12));
    let speed = derivative[0].hypot(derivative[1]);
    assert!(speed > 0.0, "the derivative must not vanish");
    // The quarter circle has radius 5, so the point is 5 from the origin.
    assert!(close(point[0].hypot(point[1]), 5.0, 1e-9));
}

#[test]
fn the_sphere_normal_is_the_unit_radial_direction() {
    for (u, v) in [(0.0, 0.0), (1.0, 0.5), (2.5, 1.2)] {
        let (point, normal) = surface_point_and_normal(u, v).expect("surface");
        let radius = point[0].hypot(point[1]).hypot(point[2]);
        assert!(close(radius, 5.0, 1e-9), "u={u} v={v}: radius {radius}");
        let length = normal[0].hypot(normal[1]).hypot(normal[2]);
        assert!(close(length, 1.0, 1e-9), "the normal must be a unit vector");
        // The normal of a sphere is parallel to the position vector.
        let dot = (point[0] * normal[0] + point[1] * normal[1] + point[2] * normal[2]) / radius;
        assert!(close(dot.abs(), 1.0, 1e-9), "u={u} v={v}: normal/point alignment {dot}");
    }
}

#[test]
fn a_line_through_the_box_hits_two_faces() {
    let (hits, first) = line_face_hits().expect("intersection");
    assert_eq!(hits, 2, "a line through a box enters and leaves it");
    let first = first.expect("first hit");
    assert!(close(first[0], 5.0, 1e-9) && close(first[1], 5.0, 1e-9));
    assert!(close(first[2], 0.0, 1e-7), "the first hit is the bottom face at z=0, got {}", first[2]);
}

#[test]
fn the_tessellation_stays_on_the_exact_sphere() {
    for tolerance in [0.01, 0.001] {
        let report = sphere_mesh(5.0, tolerance).expect("mesh");
        assert!(report.max_deviation <= tolerance, "tolerance {tolerance}: deviation {}", report.max_deviation);
        assert_eq!(report.boundary_edges, 0, "the mesh must be closed");
        assert!(report.triangles > 0);
    }
}

#[test]
fn the_reference_fixtures_match_their_analytic_values() {
    let (volume, area) = box_measures(10.0, 10.0, 4.0).expect("box");
    assert!(close(volume, 400.0, 1e-9));
    assert!(close(area, 360.0, 1e-9));

    let (volume, area) = sphere_measures(5.0).expect("sphere");
    assert!(close(volume, 4.0 / 3.0 * PI * 125.0, 1e-7));
    assert!(close(area, 4.0 * PI * 25.0, 1e-7));

    let (volume, area) = cylinder_measures(2.0, 4.0).expect("cylinder");
    assert!(close(volume, PI * 4.0 * 4.0, 1e-9));
    assert!(close(area, 24.0 * PI, 1e-7));
}
