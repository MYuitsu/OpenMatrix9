// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused acceptance suite for M03 `OM9-L2-02-04` (04-febe-freeform-curve-modeling).
//!
//! One test per L3 owner (`m03_02_04_001` … `m03_02_04_012`). Each test asserts a
//! measured positive oracle and at least one negative refusal through the same safe-Rust
//! service the host commands dispatch to. Run one owner with:
//!
//! ```text
//! cargo test -p om9-core --test l2_02_04 m03_02_04_001 -- --exact --nocapture
//! ```
use om9_core::{
    brep::{Brep, BrepDisplay},
    budget::Limits,
    curve_build::{
        KnotMode, arc_by_center, arc_through, circle_on_frame, ellipse, fit_points, insert_knot,
        interpolate, polygon, polyline, rectangle,
    },
    curve_modify::{
        Continuity, arc_blend_lines, blend_curves, chamfer_lines, fillet_lines, match_curve,
        offset_curve, rebuild_curve,
    },
    curve_ops::{
        arc_length, close_curve, curve_endpoints, curve_grips, divide_by_count, divide_by_length,
        explode_curve, fraction_derivatives, insert_control_point, is_closed, join_curves,
        move_grip, point_at, reverse_curve, sample_curve, set_seam, split_curve, tangent_at,
        trim_curve,
    },
    curve_surface_ops::{
        boundary_edges, closest_point_on_surface, curve_curve_intersections,
        curve_surface_intersections, duplicate_brep_edge, duplicate_isocurve, point_from_two_views,
        pullback_curve, sketch_on_mesh, sketch_on_surface,
    },
    geometry::{Curve, Mesh, Point3},
    surface_kernel::{self, SurfaceIso},
};
use uuid::Uuid;

fn line(a: Point3, b: Point3) -> Curve {
    let length = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, length],
    }
}

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= tolerance)
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
fn m03_02_04_001_line_polyline_point_creation() {
    let limits = Limits::default();
    let segment = line([0.0; 3], [3.0, 4.0, 0.0]);
    let length = arc_length(&segment, &limits).unwrap();
    let chain = polyline(&[[0.0; 3], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]]).unwrap();
    let polyline_length = arc_length(&chain, &limits).unwrap();
    assert!((length - 5.0).abs() < 1e-10);
    assert!((polyline_length - 20.0).abs() < 1e-9);
    assert_eq!(explode_curve(&chain, &limits).unwrap().len(), 2);
    assert!(polyline(&[[0.0; 3], [0.0; 3]]).is_err());
    assert!(line([0.0; 3], [0.0; 3]).validate(&limits).is_err());
    println!("line_length_mm={length:.12}");
    println!("polyline_length_mm={polyline_length:.12}");
}

#[test]
fn m03_02_04_002_circle_arc_ellipse_polygon_creation() {
    let limits = Limits::default();
    let circle = circle_on_frame([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).unwrap();
    let circle_length = arc_length(&circle, &limits).unwrap();
    assert!((circle_length - std::f64::consts::TAU * 5.0).abs() < 1e-6);
    let arc = arc_by_center(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        0.0,
        std::f64::consts::FRAC_PI_2,
    )
    .unwrap();
    assert!((arc_length(&arc, &limits).unwrap() - std::f64::consts::FRAC_PI_2 * 5.0).abs() < 1e-6);
    let arc3 = arc_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0]).unwrap();
    assert!((arc_length(&arc3, &limits).unwrap() - std::f64::consts::PI * 5.0).abs() < 1e-6);
    let ellipse = Curve::Nurbs(
        ellipse(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            5.0,
            5.0,
            &limits,
        )
        .unwrap(),
    );
    assert!((arc_length(&ellipse, &limits).unwrap() - std::f64::consts::TAU * 5.0).abs() < 1e-5);
    let square = polygon(&[
        [0.0; 3],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
    ])
    .unwrap();
    assert!((arc_length(&square, &limits).unwrap() - 40.0).abs() < 1e-9);
    assert!(polygon(&[[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]).is_err());
    assert!(circle_on_frame([0.0; 3], [2.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).is_err());
    println!("circle_length_mm={circle_length:.12}");
    println!(
        "polygon_length_mm={:.12}",
        arc_length(&square, &limits).unwrap()
    );
}

#[test]
fn m03_02_04_003_interpolated_and_controlpoint_nurbs() {
    let limits = Limits::default();
    let points = [[0.0; 3], [2.0, 5.0, 0.0], [6.0, 4.0, 0.0], [9.0, 0.0, 0.0]];
    let curve = Curve::Nurbs(interpolate(&points, 3, KnotMode::ChordLength, &limits).unwrap());
    let parameters =
        om9_core::curve_build::parameters(&points, KnotMode::ChordLength, false).unwrap();
    for (index, point) in points.iter().enumerate() {
        assert!(close(
            point_at(&curve, parameters[index], &limits).unwrap(),
            *point,
            1e-7
        ));
    }
    let edited = insert_control_point(&curve, 0.5, &limits).unwrap();
    for step in 0..=16 {
        let fraction = step as f64 / 16.0;
        assert!(close(
            point_at(&curve, fraction, &limits).unwrap(),
            point_at(&edited, fraction, &limits).unwrap(),
            1e-8
        ));
    }
    let control = Curve::Nurbs(om9_core::geometry::NurbsCurve {
        degree: 2,
        poles: vec![[0.0; 3], [1.0, 2.0, 0.0], [3.0, 2.0, 0.0]],
        weights: vec![1.0; 3],
        knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    });
    control.validate(&limits).unwrap();
    assert!(interpolate(&points, 2, KnotMode::ChordLength, &limits).is_err());
    assert!(interpolate(&points[..2], 3, KnotMode::Uniform, &limits).is_err());
    println!(
        "interpolated_pole_count={}",
        curve_grips(&curve, 1e-9, &limits).unwrap().poles.len()
    );
}

#[test]
fn m03_02_04_004_curve_edit_points_and_grips() {
    let limits = Limits::default();
    let curve = Curve::Nurbs(
        interpolate(
            &[[0.0; 3], [2.0, 3.0, 0.0], [5.0, 2.0, 0.0], [8.0, 0.0, 0.0]],
            3,
            KnotMode::ChordLength,
            &limits,
        )
        .unwrap(),
    );
    let grips = curve_grips(&curve, 1e-9, &limits).unwrap();
    assert_eq!(grips.poles.len(), 4);
    assert_eq!(grips.edit_points.len(), 4);
    let moved = move_grip(&curve, 1, [2.0, 6.0, 0.0], &limits).unwrap();
    assert!(close(
        curve_grips(&moved, 1e-9, &limits).unwrap().poles[1],
        [2.0, 6.0, 0.0],
        0.0
    ));
    let inserted = insert_control_point(&curve, 0.4, &limits).unwrap();
    assert_eq!(
        curve_grips(&inserted, 1e-9, &limits).unwrap().poles.len(),
        5
    );
    assert!(move_grip(&curve, 9, [0.0; 3], &limits).is_err());
    let line = line([0.0; 3], [10.0, 0.0, 0.0]);
    assert_eq!(curve_grips(&line, 1e-9, &limits).unwrap().poles.len(), 2);
    println!("nurbs_grip_count={}", grips.poles.len());
    println!(
        "inserted_grip_count={}",
        curve_grips(&inserted, 1e-9, &limits).unwrap().poles.len()
    );
}

#[test]
fn m03_02_04_005_join_explode_close_and_split() {
    let limits = Limits::default();
    let first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let second = line([10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let joined = join_curves(&[first, second], 1e-9, &limits).unwrap();
    assert!((arc_length(&joined, &limits).unwrap() - 20.0).abs() < 1e-9);
    assert_eq!(explode_curve(&joined, &limits).unwrap().len(), 2);
    let closed = close_curve(&joined, 1e-9, &limits).unwrap();
    assert!(is_closed(&closed, 1e-9, &limits).unwrap());
    let (left, right) = split_curve(&joined, 0.25, &limits).unwrap();
    assert!((arc_length(&left, &limits).unwrap() - 5.0).abs() < 1e-9);
    assert!((arc_length(&right, &limits).unwrap() - 15.0).abs() < 1e-9);
    let trimmed = trim_curve(&joined, 0.25, 0.75, &limits).unwrap();
    assert!((arc_length(&trimmed, &limits).unwrap() - 10.0).abs() < 1e-9);
    assert!(
        join_curves(
            &[
                line([0.0; 3], [1.0, 0.0, 0.0]),
                line([2.0, 0.0, 0.0], [3.0, 0.0, 0.0])
            ],
            1e-9,
            &limits
        )
        .is_err()
    );
    println!(
        "joined_length_mm={:.12}",
        arc_length(&joined, &limits).unwrap()
    );
    println!(
        "trimmed_length_mm={:.12}",
        arc_length(&trimmed, &limits).unwrap()
    );
}

#[test]
fn m03_02_04_006_offset_fillet_chamfer() {
    let limits = Limits::default();
    let horizontal = line([0.0; 3], [10.0, 0.0, 0.0]);
    let offset = offset_curve(&horizontal, 2.0, [0.0, 0.0, 1.0], 16, &limits).unwrap();
    assert!(close(
        curve_endpoints(&offset, &limits).unwrap()[0],
        [0.0, -2.0, 0.0],
        1e-12
    ));
    let circle = circle_on_frame([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).unwrap();
    let grown = offset_curve(&circle, 2.0, [0.0, 0.0, 1.0], 16, &limits).unwrap();
    assert!((arc_length(&grown, &limits).unwrap() - std::f64::consts::TAU * 7.0).abs() < 1e-6);
    let first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let second = line([0.0; 3], [0.0, 10.0, 0.0]);
    let fillet = fillet_lines(&first, &second, 2.0, [0.0, 0.0, 1.0], 1e-7, &limits).unwrap();
    assert!((arc_length(&fillet.arc, &limits).unwrap() - std::f64::consts::PI).abs() < 1e-6);
    assert!((arc_length(&fillet.first_trimmed, &limits).unwrap() - 8.0).abs() < 1e-9);
    let chamfer = chamfer_lines(&first, &second, 3.0, 4.0, 1e-7, &limits).unwrap();
    assert!((arc_length(&chamfer.line, &limits).unwrap() - 5.0).abs() < 1e-9);
    assert!(fillet_lines(&first, &second, 20.0, [0.0, 0.0, 1.0], 1e-7, &limits).is_err());
    assert!(offset_curve(&circle, -6.0, [0.0, 0.0, 1.0], 16, &limits).is_err());
    println!(
        "fillet_arc_length_mm={:.12}",
        arc_length(&fillet.arc, &limits).unwrap()
    );
    println!(
        "chamfer_length_mm={:.12}",
        arc_length(&chamfer.line, &limits).unwrap()
    );
}

#[test]
fn m03_02_04_007_blend_arcblend_and_match() {
    let limits = Limits::default();
    let first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let second = line([20.0, 10.0, 0.0], [20.0, 20.0, 0.0]);
    let g1 = blend_curves(&first, &second, Continuity::G1, &limits).unwrap();
    assert!(close(
        curve_endpoints(&g1, &limits).unwrap()[0],
        [10.0, 0.0, 0.0],
        1e-9
    ));
    assert!(close(
        curve_endpoints(&g1, &limits).unwrap()[1],
        [20.0, 10.0, 0.0],
        1e-9
    ));
    assert!(close(
        tangent_at(&g1, 0.0, &limits).unwrap(),
        [1.0, 0.0, 0.0],
        1e-9
    ));
    assert!(close(
        tangent_at(&g1, 1.0, &limits).unwrap(),
        [0.0, 1.0, 0.0],
        1e-9
    ));
    let g2 = blend_curves(&first, &second, Continuity::G2, &limits).unwrap();
    assert!(close(
        fraction_derivatives(&g2, 0.0, 2, &limits).unwrap()[2],
        [0.0; 3],
        1e-9
    ));
    let g0 = blend_curves(&first, &second, Continuity::G0, &limits).unwrap();
    assert!((arc_length(&g0, &limits).unwrap() - 14.142135623730951).abs() < 1e-9);
    let corner_first = line([0.0; 3], [10.0, 0.0, 0.0]);
    let corner_second = line([0.0; 3], [0.0, 10.0, 0.0]);
    let arc = arc_blend_lines(
        &corner_first,
        &corner_second,
        2.0,
        [0.0, 0.0, 1.0],
        1e-7,
        &limits,
    )
    .unwrap();
    assert!((arc_length(&arc, &limits).unwrap() - std::f64::consts::PI).abs() < 1e-6);
    let matched = match_curve(&first, true, &second, false, Continuity::G1, &limits).unwrap();
    assert!(close(
        curve_endpoints(&matched, &limits).unwrap()[1],
        [20.0, 10.0, 0.0],
        1e-9
    ));
    assert!(close(
        tangent_at(&matched, 1.0, &limits).unwrap(),
        [0.0, 1.0, 0.0],
        1e-9
    ));
    let parallel = line([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    assert!(fillet_lines(&first, &parallel, 1.0, [0.0, 0.0, 1.0], 1e-7, &limits).is_err());
    println!(
        "g1_blend_length_mm={:.12}",
        arc_length(&g1, &limits).unwrap()
    );
    println!(
        "g2_blend_length_mm={:.12}",
        arc_length(&g2, &limits).unwrap()
    );
}

#[test]
fn m03_02_04_008_rebuild_refit_and_knot_operations() {
    let limits = Limits::default();
    let circle = circle_on_frame([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).unwrap();
    let rebuilt = rebuild_curve(&circle, 48, 3, None, KnotMode::ChordLength, &limits).unwrap();
    let mut worst: f64 = 0.0;
    for step in 0..=32 {
        let point = point_at(&rebuilt, step as f64 / 32.0, &limits).unwrap();
        worst = worst.max((point[0].hypot(point[1]) - 5.0).abs());
    }
    assert!(worst < 0.02, "rebuild deviation {worst}");
    let control = Curve::Nurbs(
        interpolate(
            &[[0.0; 3], [2.0, 4.0, 0.0], [6.0, 4.0, 0.0], [8.0, 0.0, 0.0]],
            3,
            KnotMode::Uniform,
            &limits,
        )
        .unwrap(),
    );

    let Curve::Nurbs(source) = &control else {
        unreachable!()
    };
    let once = insert_knot(source, 0.5, 1, &limits).unwrap();
    assert_eq!(once.poles.len(), source.poles.len() + 1);
    for step in 0..=16 {
        assert!(close(
            point_at(&control, step as f64 / 16.0, &limits).unwrap(),
            point_at(&Curve::Nurbs(once.clone()), step as f64 / 16.0, &limits).unwrap(),
            1e-8
        ));
    }
    let promoted = om9_core::curve_build::elevate_degree(&once, &limits).unwrap();
    assert_eq!(promoted.degree, once.degree + 1);
    let samples = sample_curve(&control, 20, &limits).unwrap();
    let fitted = fit_points(&samples, 3, 6, KnotMode::ChordLength, &limits).unwrap();
    assert_eq!(fitted.poles.len(), 6);
    assert!(fit_points(&samples, 3, 2, KnotMode::ChordLength, &limits).is_err());
    assert!(insert_knot(&once, 2.5, 1, &limits).is_err());
    println!("rebuild_max_radial_deviation_mm={worst:.12}");
    println!("knot_inserted_poles={}", once.poles.len());
}

#[test]
fn m03_02_04_009_project_pullback_and_intersections() {
    let limits = Limits::default();
    let surface = plane();
    let projection = closest_point_on_surface(&surface, [2.0, 3.0, 5.0], &limits).unwrap();
    assert!((projection.distance - 5.0).abs() < 1e-9);
    assert!(close(projection.point, [2.0, 3.0, 0.0], 1e-9));
    let source = line([1.0, 1.0, 1.0], [9.0, 8.0, 1.0]);
    let pullback = pullback_curve(&source, &surface, 17, 1.0 + 1e-6, &limits).unwrap();
    assert!((pullback.max_deviation - 1.0).abs() < 1e-6);
    assert!(pullback_curve(&source, &surface, 17, 0.5, &limits).is_err());
    let first = line([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    let second = line([5.0, 0.0, 0.0], [5.0, 10.0, 0.0]);
    let hits = curve_curve_intersections(&first, &second, 1e-8, &limits).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(close(hits[0].point, [5.0, 5.0, 0.0], 1e-9));
    let circle = circle_on_frame([5.0, 5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0).unwrap();
    let chord = line([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    assert_eq!(
        curve_curve_intersections(&chord, &circle, 1e-6, &limits)
            .unwrap()
            .len(),
        2
    );
    let pierce = line([2.0, 2.0, -3.0], [2.0, 2.0, 3.0]);
    let hits = curve_surface_intersections(&pierce, &surface, 1e-4, &limits).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(close(hits[0].point, [2.0, 2.0, 0.0], 1e-4));
    println!("projection_distance_mm={:.12}", projection.distance);
    println!("pullback_deviation_mm={:.12}", pullback.max_deviation);
}

#[test]
fn m03_02_04_010_duplicate_edge_border_and_isocurve() {
    let limits = Limits::default();
    let surface = plane();
    let iso = duplicate_isocurve(&surface, SurfaceIso::U, 0.5, &limits).unwrap();
    assert!((point_at(&iso, 0.5, &limits).unwrap()[0] - 5.0).abs() < 1e-9);
    let face = Uuid::from_u128(1);
    let edges: Vec<Uuid> = (2..6).map(Uuid::from_u128).collect();
    let vertices: Vec<Uuid> = (6..10).map(Uuid::from_u128).collect();
    let brep = Brep::from_native(
        "CASCADE Topology V3, (c) Open Cascade".to_string(),
        BrepDisplay {
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
            edges: edges.clone(),
            vertex_ids: vertices,
            face_edges: vec![[0, 0], [0, 1], [0, 2], [0, 3]],
        },
        &limits,
    )
    .unwrap();
    let boundary = boundary_edges(&brep, &limits).unwrap();
    assert_eq!(boundary.len(), 4);
    let edge = duplicate_brep_edge(&brep, edges[3], &limits).unwrap();
    assert!((arc_length(&edge, &limits).unwrap() - 10.0).abs() < 1e-9);
    assert!(duplicate_brep_edge(&brep, Uuid::from_u128(99), &limits).is_err());
    println!("boundary_edge_count={}", boundary.len());
    println!(
        "duplicated_edge_length_mm={:.12}",
        arc_length(&edge, &limits).unwrap()
    );
}

#[test]
fn m03_02_04_011_curve_seams_directions_and_divisions() {
    let limits = Limits::default();
    let square = rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0).unwrap();
    assert!((arc_length(&square, &limits).unwrap() - 40.0).abs() < 1e-9);
    let quarters = divide_by_count(&square, 4, &limits).unwrap();
    assert_eq!(quarters.len(), 5);
    for pair in quarters.windows(2) {
        assert!(pair[1] > pair[0]);
    }
    let tenths = divide_by_length(&square, 10.0, &limits).unwrap();
    assert_eq!(tenths.len(), 5);
    let seamed = set_seam(&square, 0.25, 1e-9, &limits).unwrap();
    let old_quarter = point_at(&square, 0.25, &limits).unwrap();
    assert!(close(
        curve_endpoints(&seamed, &limits).unwrap()[0],
        old_quarter,
        1e-9
    ));
    let reversed = reverse_curve(&square, &limits).unwrap();
    assert!(close(
        point_at(&reversed, 0.25, &limits).unwrap(),
        point_at(&square, 0.75, &limits).unwrap(),
        1e-8
    ));
    let open = line([0.0; 3], [10.0, 0.0, 0.0]);
    assert!(set_seam(&open, 0.5, 1e-9, &limits).is_err());
    assert!(close(
        tangent_at(&open, 0.5, &limits).unwrap(),
        [1.0, 0.0, 0.0],
        1e-12
    ));
    println!(
        "square_length_mm={:.12}",
        arc_length(&square, &limits).unwrap()
    );
    println!("division_count={}", quarters.len());
}

#[test]
fn m03_02_04_012_sketch_on_surface_mesh_and_two_views() {
    let limits = Limits::default();
    let surface = plane();
    let surface_curve =
        sketch_on_surface(&surface, &[[0.1, 0.2], [0.5, 0.5], [0.9, 0.8]], &limits).unwrap();
    assert!(close(
        point_at(&surface_curve, 0.0, &limits).unwrap(),
        [1.0, 2.0, 0.0],
        1e-9
    ));
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
    let mesh_curve = sketch_on_mesh(
        &mesh,
        &[[1.0, 1.0, 2.0], [4.0, 1.0, 2.0], [4.0, 4.0, 2.0]],
        2.0 + 1e-6,
        &limits,
    )
    .unwrap();
    for step in 0..=8 {
        assert!(point_at(&mesh_curve, step as f64 / 8.0, &limits).unwrap()[2].abs() < 1e-9);
    }
    assert!(sketch_on_mesh(&mesh, &[[1.0, 1.0, 2.0], [4.0, 1.0, 2.0]], 0.5, &limits).is_err());
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
    println!(
        "surface_sketch_start={:?}",
        point_at(&surface_curve, 0.0, &limits).unwrap()
    );
    println!("two_view_point={point:?}");
}

#[test]
fn m03_02_04_003_stress_interpolated_curve_editing() {
    let limits = Limits::default();
    let mut max_deviation: f64 = 0.0;
    let mut curves = 0usize;
    for count in 4..=12usize {
        for degree in [1usize, 3, 5] {
            if degree >= count {
                continue;
            }
            let points: Vec<Point3> = (0..count)
                .map(|index| {
                    let x = index as f64 * 1.5;
                    let y = (index as f64 * 0.7).sin() * 3.0;
                    let z = (index as f64 * 0.3).cos();
                    [x, y, z]
                })
                .collect();
            let curve =
                Curve::Nurbs(interpolate(&points, degree, KnotMode::ChordLength, &limits).unwrap());
            let parameters =
                om9_core::curve_build::parameters(&points, KnotMode::ChordLength, false).unwrap();
            for (index, point) in points.iter().enumerate() {
                assert!(close(
                    point_at(&curve, parameters[index], &limits).unwrap(),
                    *point,
                    1e-7
                ));
            }
            let edited = insert_control_point(&curve, 0.37, &limits).unwrap();
            for step in 0..=16 {
                let fraction = step as f64 / 16.0;
                max_deviation = max_deviation.max({
                    let a = point_at(&curve, fraction, &limits).unwrap();
                    let b = point_at(&edited, fraction, &limits).unwrap();
                    (0..3)
                        .map(|axis| (a[axis] - b[axis]).abs())
                        .fold(0.0, f64::max)
                });
            }
            curves += 1;
        }
    }
    assert_eq!(curves, 25);
    assert!(
        max_deviation < 1e-7,
        "control point insertion deviation {max_deviation}"
    );
    println!("stress_curves={curves}");
    println!("stress_max_control_insertion_deviation_mm={max_deviation:.12}");
}

#[test]
fn m03_02_04_005_stress_join_split_round_trip() {
    let limits = Limits::default();
    let mut max_length_error: f64 = 0.0;
    let mut cases = 0usize;
    for vertices in 4..=12usize {
        let points: Vec<Point3> = (0..vertices)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / vertices as f64;
                [10.0 * angle.cos(), 10.0 * angle.sin(), 0.0]
            })
            .collect();
        let polygon = polygon(&points).unwrap();
        let total = arc_length(&polygon, &limits).unwrap();
        let (left, right) = split_curve(&polygon, 0.5, &limits).unwrap();
        let rejoined = join_curves(&[left, right], 1e-8, &limits).unwrap();
        let measured = arc_length(&rejoined, &limits).unwrap();
        max_length_error = max_length_error.max((measured - total).abs());
        assert!(is_closed(&rejoined, 1e-7, &limits).unwrap());
        cases += 1;
    }
    assert_eq!(cases, 9);
    assert!(
        max_length_error < 1e-8,
        "join/split length drift {max_length_error}"
    );
    println!("stress_polygons={cases}");
    println!("stress_max_length_drift_mm={max_length_error:.12}");
}

#[test]
fn m03_02_04_006_stress_fillet_and_chamfer_family() {
    let limits = Limits::default();
    let mut max_tangency_error: f64 = 0.0;
    let mut cases = 0usize;
    for step in 0..100usize {
        let span = 10.0 + (step % 7) as f64;
        let radius = 0.5 + (step % 8) as f64 * 0.4;
        let first = line([0.0; 3], [span, 0.0, 0.0]);
        let second = line([0.0; 3], [0.0, span, 0.0]);
        let fillet = fillet_lines(&first, &second, radius, [0.0, 0.0, 1.0], 1e-7, &limits).unwrap();
        let tangent_length = radius;
        max_tangency_error =
            max_tangency_error.max((fillet.tangent_points[0][0] - tangent_length).abs());
        max_tangency_error =
            max_tangency_error.max((fillet.tangent_points[1][1] - tangent_length).abs());
        let measured_arc = arc_length(&fillet.arc, &limits).unwrap();
        assert!((measured_arc - radius * std::f64::consts::FRAC_PI_2).abs() < 1e-6);
        assert!(
            (arc_length(&fillet.first_trimmed, &limits).unwrap() - (span - tangent_length)).abs()
                < 1e-9
        );
        let chamfer = chamfer_lines(&first, &second, radius, radius, 1e-7, &limits).unwrap();
        assert!(
            (arc_length(&chamfer.line, &limits).unwrap() - radius * 2.0f64.sqrt()).abs() < 1e-9
        );
        cases += 1;
    }
    assert_eq!(cases, 100);
    assert!(
        max_tangency_error < 1e-12,
        "fillet tangency error {max_tangency_error}"
    );
    println!("stress_corners={cases}");
    println!("stress_max_tangency_error_mm={max_tangency_error:.12}");
}
