// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    brep::{Brep, BrepDisplay},
    budget::Limits,
    curve_build::circle_through,
    curve_ops::point_at,
    curve_surface_ops::{
        boundary_edges, closest_point_on_mesh, closest_point_on_surface, curve_curve_intersections,
        curve_surface_intersections, duplicate_brep_edge, duplicate_isocurve, point_from_two_views,
        pullback_curve, sketch_on_mesh, sketch_on_surface,
    },
    geometry::{Curve, Mesh, Point3},
    surface_kernel::{self, SurfaceIso},
};
use uuid::Uuid;

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= tolerance)
}

fn line(a: Point3, b: Point3) -> Curve {
    let length = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, length],
    }
}

fn plane() -> om9_core::geometry::NurbsSurface {
    surface_kernel::plane(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap()
}

#[test]
fn surface_projection_and_pullback_measure_the_deviation() {
    let limits = Limits::default();
    let surface = plane();
    let projection = closest_point_on_surface(&surface, [2.0, 3.0, 5.0], &limits).unwrap();
    assert!(close(projection.point, [2.0, 3.0, 0.0], 1e-9));
    assert!((projection.distance - 5.0).abs() < 1e-9);
    assert!((projection.parameters[0] - 0.2).abs() < 1e-9);
    assert!((projection.parameters[1] - 0.3).abs() < 1e-9);
    let source = line([1.0, 1.0, 1.0], [9.0, 8.0, 1.0]);
    let pullback = pullback_curve(&source, &surface, 17, 1.0 + 1e-6, &limits).unwrap();
    assert_eq!(pullback.parameters.len(), 17);
    assert!((pullback.max_deviation - 1.0).abs() < 1e-6);
    for point in &pullback.points {
        assert!(point[2].abs() < 1e-9);
    }
    assert!(pullback_curve(&source, &surface, 17, 0.5, &limits).is_err());
}

#[test]
fn line_line_and_circle_intersections_are_analytic() {
    let limits = Limits::default();
    let first = line([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    let second = line([5.0, 0.0, 0.0], [5.0, 10.0, 0.0]);
    let hits = curve_curve_intersections(&first, &second, 1e-8, &limits).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(close(hits[0].point, [5.0, 5.0, 0.0], 1e-9));
    assert!((hits[0].first_fraction - 0.5).abs() < 1e-9);
    let circle = circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    let chord = line([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let hits = curve_curve_intersections(&chord, &circle, 1e-6, &limits).unwrap();
    assert_eq!(hits.len(), 2);
    let mut xs: Vec<f64> = hits.iter().map(|hit| hit.point[0]).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!((xs[0] + 5.0).abs() < 1e-6);
    assert!((xs[1] - 5.0).abs() < 1e-6);
}

#[test]
fn coplanar_circle_circle_intersections_match_the_closed_form() {
    let limits = Limits::default();
    let first = circle_through([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]).unwrap();
    let second = circle_through([2.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 0.0, 0.0]).unwrap();
    let hits = curve_curve_intersections(&first, &second, 1e-7, &limits).unwrap();
    assert_eq!(hits.len(), 2);
    for hit in &hits {
        assert!((hit.point[0] - 0.5).abs() < 1e-7);
        assert!((hit.point[1].abs() - 3.0f64.sqrt() / 2.0).abs() < 1e-7);
    }
}

#[test]
fn curve_surface_intersection_finds_a_piercing_line() {
    let limits = Limits::default();
    let surface = plane();
    let pierce = line([2.0, 2.0, -3.0], [2.0, 2.0, 3.0]);
    let hits = curve_surface_intersections(&pierce, &surface, 1e-4, &limits).unwrap();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(close(hits[0].point, [2.0, 2.0, 0.0], 1e-4));
}

#[test]
fn mesh_projection_and_sketch_land_on_the_triangle() {
    let limits = Limits::default();
    let mesh = Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [10.0, 10.0, 0.0],
        ],
        faces: vec![vec![0, 1, 2], vec![1, 3, 2]],
        normals: Vec::new(),
        colors: Vec::new(),

        corner_uvs: Vec::new(),
    };
    let (point, distance) = closest_point_on_mesh(&mesh, [2.0, 3.0, 4.0], &limits).unwrap();
    assert!(close(point, [2.0, 3.0, 0.0], 1e-9));
    assert!((distance - 4.0).abs() < 1e-9);
    let curve = sketch_on_mesh(
        &mesh,
        &[[1.0, 1.0, 2.0], [4.0, 1.0, 2.0], [4.0, 4.0, 2.0]],
        2.0 + 1e-6,
        &limits,
    )
    .unwrap();
    for step in 0..=8 {
        let on_curve = point_at(&curve, step as f64 / 8.0, &limits).unwrap();
        assert!(on_curve[2].abs() < 1e-9);
    }
}

#[test]
fn two_view_reconstruction_accepts_meeting_rays_and_refuses_skew() {
    let point = point_from_two_views(
        [0.0, 0.0, 5.0],
        [0.0, 0.0, -1.0],
        [5.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        1e-9,
    )
    .unwrap();
    assert!(close(point, [0.0, 0.0, 0.0], 1e-9));
    assert!(
        point_from_two_views(
            [0.0, 0.0, 5.0],
            [0.0, 0.0, -1.0],
            [5.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            1e-6,
        )
        .is_err()
    );
    assert!(
        point_from_two_views([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [1.0, 0.0, 0.0], 1e-6).is_err()
    );
}

#[test]
fn surface_sketch_and_isocurve_use_the_surface_evaluator() {
    let limits = Limits::default();
    let surface = plane();
    let curve =
        sketch_on_surface(&surface, &[[0.1, 0.2], [0.5, 0.5], [0.9, 0.8]], &limits).unwrap();
    let start = point_at(&curve, 0.0, &limits).unwrap();
    assert!(close(start, [1.0, 2.0, 0.0], 1e-9));
    let iso = duplicate_isocurve(&surface, SurfaceIso::U, 0.5, &limits).unwrap();
    let middle = point_at(&iso, 0.5, &limits).unwrap();
    assert!((middle[0] - 5.0).abs() < 1e-9);
    assert!(middle[2].abs() < 1e-9);
}

fn square_brep() -> Brep {
    let face = Uuid::from_u128(1);
    let edges: Vec<Uuid> = (2..6).map(Uuid::from_u128).collect();
    let vertices: Vec<Uuid> = (6..10).map(Uuid::from_u128).collect();
    let display = BrepDisplay {
        source_sha256: [0; 32],
        deflection_mm: 0.01,
        points: vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
        ],
        triangles: vec![[0, 1, 2, 0], [0, 2, 3, 0]],
        segments: vec![[0, 1, 0], [1, 2, 1], [2, 3, 2], [3, 0, 3]],
        vertices: vec![0, 1, 2, 3],
        faces: vec![face],
        edges,
        vertex_ids: vertices,
        face_edges: vec![[0, 0], [0, 1], [0, 2], [0, 3]],
    };
    Brep::from_native(
        "CASCADE Topology V3, (c) Open Cascade".to_string(),
        display,
        &Limits::default(),
    )
    .unwrap()
}

#[test]
fn brep_boundary_and_edge_duplication_follow_the_display_snapshot() {
    let limits = Limits::default();
    let brep = square_brep();
    let boundary = boundary_edges(&brep, &limits).unwrap();
    assert_eq!(boundary.len(), 4);
    // Segment 3 is the vertical edge (0,10) -> (0,0); its display chain is ordered from
    // the degree-one endpoint at the origin.
    let edge = brep.display.edges[3];
    let curve = duplicate_brep_edge(&brep, edge, &limits).unwrap();
    let [start, end] = om9_core::curve_ops::curve_endpoints(&curve, &limits).unwrap();
    let length = om9_core::curve_ops::arc_length(&curve, &limits).unwrap();
    assert!((length - 10.0).abs() < 1e-9);
    assert!(close(start, [0.0, 0.0, 0.0], 1e-9));
    assert!(close(end, [0.0, 10.0, 0.0], 1e-9));
}
