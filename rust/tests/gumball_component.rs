// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::gumball::GumballError;
use openmatrix9_rust::gumball_component::{mesh_component_frame, Component};

fn close(actual: [f64; 3], expected: [f64; 3]) {
    for (a, b) in actual.into_iter().zip(expected) {
        assert!((a - b).abs() < 1e-10, "{actual:?} != {expected:?}");
    }
}

const POINTS: [[f64; 3]; 4] = [[4., 5., 6.], [6., 5., 6.], [4., 6., 6.], [4., 5., 10.]];
const TRIANGLES: [[u32; 3]; 2] = [[0, 1, 2], [0, 2, 3]];

// Catches using the whole owner's bounds or caller endpoint order.
#[test]
fn edge_uses_component_midpoint_and_canonical_endpoint_direction() {
    let frame = mesh_component_frame(&POINTS, &TRIANGLES, Component::Edge(2, 0)).unwrap();
    close(frame.origin, [4., 5.5, 6.]);
    close(frame.axes[0], [0., 1., 0.]);
    let s = 5_f64.sqrt();
    close(frame.axes[2], [2. / s, 0., 1. / s]);
    close(frame.axes[1], [-1. / s, 0., 2. / s]);
    assert_eq!(
        frame,
        mesh_component_frame(&POINTS, &TRIANGLES, Component::Edge(0, 2)).unwrap()
    );
}

// Catches averaging unit normals or passing an unprojected vertex edge as X.
#[test]
fn vertex_preserves_area_weighted_normal_and_selected_position() {
    let frame = mesh_component_frame(&POINTS, &TRIANGLES, Component::Vertex(0)).unwrap();
    let s = 5_f64.sqrt();
    close(frame.origin, [4., 5., 6.]);
    close(frame.axes[2], [2. / s, 0., 1. / s]);
    close(frame.axes[0], [1. / s, 0., -2. / s]);
    close(frame.axes[1], [0., 1., 0.]);
}

// Catches accidental world-axis fallback and placement applied twice.
#[test]
fn rotated_world_observations_rotate_vertex_frame_once() {
    let points = POINTS.map(|[x, y, z]| [10. - y, 20. + x, 30. + z]);
    let frame = mesh_component_frame(&points, &TRIANGLES, Component::Vertex(0)).unwrap();
    let s = 5_f64.sqrt();
    close(frame.origin, [5., 24., 36.]);
    close(frame.axes[0], [0., 1. / s, -2. / s]);
    close(frame.axes[1], [-1., 0., 0.]);
    close(frame.axes[2], [0., 2. / s, 1. / s]);
}

// Catches requiring every selected edge to have two facets.
#[test]
fn boundary_edge_has_one_oriented_facet_frame() {
    let frame = mesh_component_frame(&POINTS, &TRIANGLES, Component::Edge(0, 1)).unwrap();
    close(frame.origin, [5., 5., 6.]);
    close(frame.axes[0], [1., 0., 0.]);
    close(frame.axes[2], [0., 0., 1.]);
}

// Catches dependence on native facet order instead of persistent point IDs.
#[test]
fn vertex_tangent_is_stable_when_triangles_are_reordered() {
    assert_eq!(
        mesh_component_frame(&POINTS, &TRIANGLES, Component::Vertex(0)).unwrap(),
        mesh_component_frame(&POINTS, &[[0, 2, 3], [0, 1, 2]], Component::Vertex(0)).unwrap(),
    );
}

#[test]
fn missing_components_and_identical_edge_endpoints_reject() {
    let points = [POINTS[0], POINTS[1], POINTS[2], POINTS[3], [8., 9., 10.]];
    for component in [
        Component::Vertex(5),
        Component::Vertex(4),
        Component::Edge(1, 3),
        Component::Edge(0, 0),
    ] {
        assert_eq!(
            mesh_component_frame(&points, &TRIANGLES, component),
            Err(GumballError::InvalidSources)
        );
    }
}

#[test]
fn invalid_indices_and_repeated_triangle_vertices_reject() {
    for triangle in [[0, 1, 4], [0, 0, 2]] {
        assert_eq!(
            mesh_component_frame(&POINTS, &[triangle], Component::Vertex(0)),
            Err(GumballError::InvalidSources)
        );
    }
}

#[test]
fn all_snapshot_coordinates_require_finite_bounded_values() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e12 + 1.] {
        let mut points = POINTS;
        points[3][0] = invalid; // Unrelated to the selected boundary edge.
        assert_eq!(
            mesh_component_frame(&points, &TRIANGLES[..1], Component::Edge(0, 1)),
            Err(GumballError::InvalidInput)
        );
    }
}

#[test]
fn empty_or_geometrically_degenerate_observations_reject() {
    assert_eq!(
        mesh_component_frame(&[], &[], Component::Vertex(0)),
        Err(GumballError::InvalidSources)
    );
    assert_eq!(
        mesh_component_frame(&POINTS, &[], Component::Vertex(0)),
        Err(GumballError::InvalidSources)
    );
    let points = [[0.; 3], [1., 0., 0.], [2., 0., 0.]];
    assert_eq!(
        mesh_component_frame(&points, &[[0, 1, 2]], Component::Vertex(0)),
        Err(GumballError::InvalidFrame)
    );
}

#[test]
fn cancelling_oriented_normals_reject_without_fabricated_axes() {
    let points = [[0.; 3], [1., 0., 0.], [0., 1., 0.], [1., 0., 0.]];
    for component in [Component::Vertex(0), Component::Edge(0, 2)] {
        assert_eq!(
            mesh_component_frame(&points, &TRIANGLES, component),
            Err(GumballError::InvalidFrame)
        );
    }
}

#[test]
fn shared_edges_require_opposite_triangle_winding() {
    for component in [Component::Vertex(0), Component::Edge(0, 2)] {
        assert_eq!(
            mesh_component_frame(&POINTS, &[[0, 1, 2], [0, 3, 2]], component),
            Err(GumballError::InvalidFrame)
        );
    }
}

#[test]
fn three_facets_on_an_incident_edge_are_nonmanifold() {
    let points = [POINTS[0], POINTS[1], POINTS[2], POINTS[3], [4., 4., 6.]];
    let triangles = [[0, 1, 2], [0, 3, 1], [0, 1, 4]];
    for component in [Component::Vertex(0), Component::Edge(0, 1)] {
        assert_eq!(
            mesh_component_frame(&points, &triangles, component),
            Err(GumballError::InvalidFrame)
        );
    }
}

#[test]
fn disconnected_vertex_fans_are_ambiguous() {
    let points = [
        [0.; 3],
        [1., 0., 0.],
        [0., 1., 0.],
        [-1., 0., 0.],
        [0., -1., 0.],
    ];
    assert_eq!(
        mesh_component_frame(&points, &[[0, 1, 2], [0, 3, 4]], Component::Vertex(0)),
        Err(GumballError::InvalidFrame)
    );
}

// Catches validation/allocation beginning before the complete snapshot budget check.
#[test]
fn full_mesh_budget_precedes_geometry_validation() {
    let points = vec![[f64::NAN; 3]; 131073];
    assert_eq!(
        mesh_component_frame(&points, &[[0, 1, 2]], Component::Vertex(0)),
        Err(GumballError::Limit)
    );
    let triangles = vec![[u32::MAX; 3]; 131073];
    assert_eq!(
        mesh_component_frame(&POINTS, &triangles, Component::Vertex(0)),
        Err(GumballError::Limit)
    );
}

#[test]
fn point_budget_boundary_still_admits_a_valid_selected_component() {
    let mut points = vec![[0.; 3]; 131072];
    points[..4].copy_from_slice(&POINTS);
    let frame = mesh_component_frame(&points, &TRIANGLES, Component::Vertex(0)).unwrap();
    close(frame.origin, [4., 5., 6.]);
}
