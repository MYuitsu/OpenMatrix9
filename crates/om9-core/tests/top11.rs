// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    curve_build::{polyline, rectangle},
    geometry::{Curve, Geometry, Mesh, NurbsSurface, Point3, Transform},
    model::{Document, Entity},
    top11::{
        GripMode, Top11Action, Top11Context, availability, duplicate_entities,
        duplicate_entities_offset, explode_entities, grip_handles, grip_points, join_entities,
        mirror_entities, move_grip_geometry, rotate_placements, split_by_cutter, split_entity,
        toolbar, translate_placements, trim_by_cutter, trim_entity,
    },
};
use std::collections::BTreeSet;

fn line(a: Point3, b: Point3) -> Curve {
    let length = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, length],
    }
}

fn doc_with(entities: Vec<Entity>) -> (Document, BTreeSet<om9_core::identity::ObjectId>) {
    let mut document = Document::new("top11");
    let mut ids = BTreeSet::new();
    for entity in entities {
        ids.insert(entity.id);
        document.entities.insert(entity.id, entity);
    }
    (document, ids)
}

#[test]
fn availability_matches_the_selection_and_the_deferred_ring_rail() {
    let empty = Top11Context {
        running: false,
        selected: 0,
        curves: 0,
        points: 0,
        meshes: 0,
        surfaces: 0,
        breps: 0,
        other: 0,
    };
    for state in toolbar(&empty) {
        assert!(!state.enabled);
    }
    assert!(!availability(Top11Action::RingRail, &empty).enabled);
    let two_curves = Top11Context {
        running: false,
        selected: 2,
        curves: 2,
        points: 0,
        meshes: 0,
        surfaces: 0,
        breps: 0,
        other: 0,
    };
    let states = toolbar(&two_curves);
    for action in [
        Top11Action::Duplicate,
        Top11Action::Mirror,
        Top11Action::Move,
        Top11Action::Rotate,
        Top11Action::Explode,
        Top11Action::Join,
    ] {
        let state = states
            .iter()
            .find(|state| state.action == action)
            .copied()
            .unwrap();
        assert!(state.enabled, "{action:?}");
    }
    assert!(!availability(Top11Action::Split, &two_curves).enabled);
    assert!(!availability(Top11Action::Trim, &two_curves).enabled);
    assert!(!availability(Top11Action::RingRail, &two_curves).enabled);
    let running = Top11Context {
        running: true,
        ..two_curves
    };
    assert!(!availability(Top11Action::Duplicate, &running).enabled);
    assert_eq!(Top11Action::ControlPoints.shortcut(), Some("F10"));
    assert_eq!(Top11Action::Duplicate.command(), "Copy");
}

#[test]
fn duplicate_is_in_place_with_fresh_identity_and_same_geometry() {
    let limits = Limits::default();
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [10.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source.clone()]);
    let copies = duplicate_entities(&document, &ids, &limits).unwrap();
    assert_eq!(copies.len(), 1);
    assert_ne!(copies[0].id, source.id);
    assert_eq!(copies[0].geometry, source.geometry);
    assert_eq!(copies[0].placement, source.placement);
    assert_eq!(copies[0].generation, 1);
}

#[test]
fn move_and_rotate_change_only_placements() {
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [10.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source.clone()]);
    let moved = translate_placements(&document, &ids, [5.0, -1.0, 2.0]).unwrap();
    assert_eq!(moved.len(), 1);
    let placement = moved[0].1;
    assert_eq!(placement.point([0.0; 3]).unwrap(), [5.0, -1.0, 2.0]);
    let rotated = rotate_placements(&document, &ids, [0.0; 3], [0.0, 0.0, 1.0], 90.0).unwrap();
    let point = rotated[0].1.point([10.0, 0.0, 0.0]).unwrap();
    assert!(point[0].abs() < 1e-9 && (point[1] - 10.0).abs() < 1e-9);
    assert_eq!(document.entities[&source.id].placement, source.placement);
}

#[test]
fn mirror_reflects_line_mesh_and_surface_without_mutating_the_source() {
    let limits = Limits::default();
    let line_entity = Entity::new(
        "line",
        Geometry::Curve(line([1.0, 2.0, 0.0], [4.0, 2.0, 0.0])),
        None,
    );
    let mesh = Mesh {
        vertices: vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        faces: vec![vec![0, 1, 2]],
        normals: Vec::new(),
        colors: Vec::new(),

        corner_uvs: Vec::new(),
    };
    let mesh_entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let surface = om9_core::surface_kernel::plane(
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        2.0,
        2.0,
        &limits,
    )
    .unwrap();
    let surface_entity = Entity::new("surface", Geometry::Surface(surface), None);
    let (document, ids) = doc_with(vec![
        line_entity.clone(),
        mesh_entity.clone(),
        surface_entity.clone(),
    ]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], true, &limits).unwrap();
    assert_eq!(outcome.copies.len(), 3);
    for copy in &outcome.copies {
        match &copy.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => {
                assert!((start[0] + 1.0).abs() < 1e-9);
                assert!((end[0] + 4.0).abs() < 1e-9);
            }
            Geometry::Mesh(mesh) => {
                assert!((mesh.vertices[0][0] + 1.0).abs() < 1e-9);
                assert_eq!(mesh.faces[0], vec![0, 2, 1]);
            }
            Geometry::Surface(surface) => {
                assert!((surface.poles[0][0] + 1.0).abs() < 1e-9);
            }
            other => panic!("unexpected mirror geometry {other:?}"),
        }
    }
    assert_eq!(
        document.entities[&line_entity.id].geometry,
        line_entity.geometry
    );
    assert_eq!(
        document.entities[&surface_entity.id].geometry,
        surface_entity.geometry
    );
}

#[test]
fn explode_join_split_and_trim_round_trip_a_polyline() {
    let limits = Limits::default();
    let polyline = polyline(&[[0.0; 3], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]]).unwrap();
    let entity = Entity::new("polyline", Geometry::Curve(polyline), None);
    let (document, ids) = doc_with(vec![entity.clone()]);
    let pieces = explode_entities(&document, &ids, &limits).unwrap();
    assert_eq!(pieces.len(), 2);
    let joined_ids: BTreeSet<_> = pieces.iter().map(|entity| entity.id).collect();
    let joined_document = doc_with(pieces.clone()).0;
    let joined = join_entities(&joined_document, &joined_ids, 1e-9, &limits).unwrap();
    let split = split_entity(&document, entity.id, 0.5, &limits).unwrap();
    assert_eq!(split.len(), 2);
    let trimmed = trim_entity(&document, entity.id, 0.25, 0.75, &limits).unwrap();
    let Geometry::Curve(curve) = joined.geometry else {
        panic!("join did not return a curve");
    };
    assert!((om9_core::curve_ops::arc_length(&curve, &limits).unwrap() - 20.0).abs() < 1e-9);
    let Geometry::Curve(curve) = trimmed else {
        panic!("trim did not return a curve");
    };
    assert!((om9_core::curve_ops::arc_length(&curve, &limits).unwrap() - 10.0).abs() < 1e-9);
    let plain = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![plain]);
    assert!(explode_entities(&document, &ids, &limits).is_err());
}

#[test]
fn grips_expose_edit_and_control_points_in_world_space() {
    let limits = Limits::default();
    let square = rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0).unwrap();
    let mut entity = Entity::new("square", Geometry::Curve(square), None);
    entity.placement = om9_core::geometry::Transform::translation([5.0, 0.0, 0.0]);
    let (document, ids) = doc_with(vec![entity]);
    let control = grip_points(&document, &ids, GripMode::Control, &limits).unwrap();
    let edit = grip_points(&document, &ids, GripMode::Edit, &limits).unwrap();
    assert_eq!(control.len(), 8);
    assert_eq!(edit.len(), 8);
    assert_eq!(control[0], [5.0, 0.0, 0.0]);
    let nurbs = om9_core::curve_build::interpolate(
        &[[0.0; 3], [2.0, 3.0, 0.0], [5.0, 2.0, 0.0], [8.0, 0.0, 0.0]],
        3,
        om9_core::curve_build::KnotMode::ChordLength,
        &limits,
    )
    .unwrap();
    let entity = Entity::new("nurbs", Geometry::Curve(Curve::Nurbs(nurbs)), None);
    let (document, ids) = doc_with(vec![entity]);
    assert_eq!(
        grip_points(&document, &ids, GripMode::Control, &limits)
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        grip_points(&document, &ids, GripMode::Edit, &limits)
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn base_point_duplicate_offsets_the_copy_placement() {
    let limits = Limits::default();
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source]);
    let copies = duplicate_entities_offset(&document, &ids, [5.0, -2.0, 0.0], &limits).unwrap();
    assert_eq!(copies.len(), 1);
    assert_eq!(
        copies[0].placement.point([0.0; 3]).unwrap(),
        [5.0, -2.0, 0.0]
    );
}

#[test]
fn grip_handles_move_control_and_edit_points() {
    let limits = Limits::default();
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [10.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source]);
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 2);
    let geometry = move_grip_geometry(&document, handles[1], [15.0, 2.0, 0.0], &limits).unwrap();
    let Geometry::Curve(Curve::Line { end, .. }) = geometry else {
        panic!("line grip lost the line");
    };
    assert!((end[0] - 15.0).abs() < 1e-9 && (end[1] - 2.0).abs() < 1e-9);

    let points = [[0.0; 3], [2.0, 3.0, 0.0], [5.0, 2.0, 0.0], [8.0, 0.0, 0.0]];
    let nurbs = om9_core::curve_build::interpolate(
        &points,
        3,
        om9_core::curve_build::KnotMode::ChordLength,
        &limits,
    )
    .unwrap();
    let entity = Entity::new("nurbs", Geometry::Curve(Curve::Nurbs(nurbs)), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    assert_eq!(handles.len(), 4);
    let target = [1.0, 6.0, 0.0];
    let geometry = move_grip_geometry(&document, handles[1], target, &limits).unwrap();
    let Geometry::Curve(curve) = geometry else {
        panic!("edit grip lost the curve");
    };
    let mut nearest = f64::INFINITY;
    for step in 0..=512 {
        let point = om9_core::curve_ops::point_at(&curve, step as f64 / 512.0, &limits).unwrap();
        nearest = nearest.min(
            (0..3)
                .map(|axis| (point[axis] - target[axis]).powi(2))
                .sum::<f64>()
                .sqrt(),
        );
    }
    assert!(nearest < 0.05, "edited curve misses target by {nearest}");
}

#[test]
fn split_and_trim_by_a_cutting_curve_use_the_measured_intersections() {
    let limits = Limits::default();
    let horizontal = line([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let vertical = line([0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    let pieces = split_by_cutter(&horizontal, &vertical, 1e-6, &limits).unwrap();
    assert_eq!(pieces.len(), 2);
    assert!((om9_core::curve_ops::arc_length(&pieces[0], &limits).unwrap() - 10.0).abs() < 1e-6);
    let kept = trim_by_cutter(&horizontal, &vertical, [-5.0, 0.0, 0.0], 1e-6, &limits).unwrap();
    assert_eq!(kept.len(), 1);
    let start = om9_core::curve_ops::curve_endpoints(&kept[0], &limits).unwrap()[0];
    assert!(start[0].abs() < 1e-6, "{start:?}");
}

#[test]
fn mirror_without_copy_keeps_the_placement_and_returns_local_geometry() {
    let limits = Limits::default();
    let mut source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    source.placement = om9_core::geometry::Transform::translation([5.0, 0.0, 0.0]);
    let (document, ids) = doc_with(vec![source.clone()]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], false, &limits).unwrap();
    assert!(outcome.copies.is_empty());
    assert_eq!(outcome.replacements.len(), 1);
    let (id, Geometry::Curve(Curve::Line { start, end, .. })) = &outcome.replacements[0] else {
        panic!("mirror did not replace the line");
    };
    assert_eq!(*id, source.id);
    assert!((start[0] + 10.0).abs() < 1e-9, "{start:?}");
    assert!((end[0] + 11.0).abs() < 1e-9, "{end:?}");
}

#[test]
fn circle_and_polycurve_grips_are_editable() {
    let limits = Limits::default();
    let circle = Curve::Circle {
        center: [0.0; 3],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 2.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let entity = Entity::new("circle", Geometry::Curve(circle), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 5);
    let geometry = move_grip_geometry(&document, handles[1], [5.0, 0.0, 0.0], &limits).unwrap();
    let Geometry::Curve(Curve::Circle { radius, center, .. }) = geometry else {
        panic!("circle grip lost the circle");
    };
    assert!((radius - 5.0).abs() < 1e-9);
    assert_eq!(center, [0.0; 3]);
    println!("circle_grip_radius={radius:.12}");

    let polycurve = Curve::Polycurve {
        segments: vec![
            line([0.0; 3], [5.0, 0.0, 0.0]),
            line([5.0, 0.0, 0.0], [5.0, 5.0, 0.0]),
        ],
        reversed: false,
    };
    let entity = Entity::new("poly", Geometry::Curve(polycurve), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 4);
    let geometry = move_grip_geometry(&document, handles[1], [5.0, 1.0, 0.0], &limits).unwrap();
    let Geometry::Curve(Curve::Polycurve { segments, .. }) = geometry else {
        panic!("polycurve grip lost the polycurve");
    };
    let Curve::Line { end, .. } = &segments[0] else {
        panic!("first segment is not a line");
    };
    assert!((end[1] - 1.0).abs() < 1e-9);
}

#[test]
fn surface_control_and_edit_grips_move_the_surface_without_reparameterising() {
    let limits = Limits::default();
    let expected = NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let entity = Entity::new("surface", Geometry::Surface(expected.clone()), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 4);
    let geometry = move_grip_geometry(&document, handles[3], [2.0, 2.0, 1.0], &limits).unwrap();
    let Geometry::Surface(edited) = geometry else {
        panic!("surface grip lost the surface");
    };
    assert_eq!(edited.poles[3], [2.0, 2.0, 1.0]);
    let edit_handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    assert_eq!(edit_handles.len(), 4);
    let geometry =
        move_grip_geometry(&document, edit_handles[3], [3.0, 3.0, 2.0], &limits).unwrap();
    let Geometry::Surface(edited) = geometry else {
        panic!("surface edit grip lost the surface");
    };
    let moved = om9_core::surface_kernel::evaluate(&edited, 1.0, 1.0, &limits).unwrap();
    assert!((moved[0] - 3.0).abs() < 1e-9, "{moved:?}");
    assert!((moved[2] - 2.0).abs() < 1e-9, "{moved:?}");
    println!("surface_control_grips=4");
    println!("surface_edit_grips=4");
}

#[test]
fn non_rigid_mirror_bakes_a_line_and_turns_a_circle_into_an_ellipse() {
    let limits = Limits::default();
    let mut line_entity = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    line_entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let (document, ids) = doc_with(vec![line_entity]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], true, &limits).unwrap();
    let Geometry::Curve(Curve::Line { start, end, .. }) = &outcome.copies[0].geometry else {
        panic!("non-rigid mirror lost the line");
    };
    assert!((start[0] + 0.0).abs() < 1e-9);
    assert!((end[0] + 2.0).abs() < 1e-9);

    let circle_entity = Entity::new(
        "circle",
        Geometry::Curve(Curve::Circle {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let mut circle_entity = circle_entity;
    circle_entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let (document, ids) = doc_with(vec![circle_entity]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [0.0, 1.0, 0.0], true, &limits).unwrap();
    assert!(matches!(
        outcome.copies[0].geometry,
        Geometry::Curve(Curve::Nurbs(_))
    ));
}

#[test]
fn non_rigid_mirror_transforms_mesh_vertices_normals_and_winding() {
    let limits = Limits::default();
    let mut mesh_entity = Entity::new(
        "mesh",
        Geometry::Mesh(Mesh {
            vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        }),
        None,
    );
    mesh_entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let (document, ids) = doc_with(vec![mesh_entity]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], true, &limits).unwrap();
    let Geometry::Mesh(mesh) = &outcome.copies[0].geometry else {
        panic!("non-rigid mirror lost the mesh");
    };
    assert!(
        (mesh.vertices[1][0] + 2.0).abs() < 1e-9,
        "{:?}",
        mesh.vertices
    );
    assert_eq!(mesh.faces, vec![vec![0, 2, 1]]);
    assert!((mesh.normals[0][2] - 1.0).abs() < 1e-9);
}

#[test]
fn surface_edit_grips_keep_a_reversed_or_periodic_surface_record() {
    let limits = Limits::default();
    let mut surface = om9_core::surface_kernel::cylinder(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        10.0,
        &limits,
    )
    .unwrap();
    surface.reversed = true;
    let entity = Entity::new("cylinder", Geometry::Surface(surface.clone()), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    let geometry = move_grip_geometry(&document, handles[10], handles[10].point, &limits).unwrap();
    let Geometry::Surface(edited) = geometry else {
        panic!("surface edit lost the surface");
    };
    assert!(edited.reversed);
    assert_eq!(edited.poles.len(), surface.poles.len());

    surface.periodic = [true, false];
    let entity = Entity::new("periodic", Geometry::Surface(surface), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    let target = [
        handles[0].point[0],
        handles[0].point[1],
        handles[0].point[2] + 1.0,
    ];
    let Geometry::Surface(edited) =
        move_grip_geometry(&document, handles[0], target, &limits).unwrap()
    else {
        panic!("periodic surface edit lost the surface");
    };
    assert!(edited.periodic[0]);
}

fn large_surface(size: usize) -> NurbsSurface {
    let degree = 3usize;
    let mut knots = vec![0.0; degree + 1];
    let spans = size - degree;
    for step in 1..spans {
        knots.push(step as f64 / spans as f64);
    }
    knots.extend(std::iter::repeat_n(1.0, degree + 1));
    let mut poles = Vec::with_capacity(size * size);
    for i in 0..size {
        for j in 0..size {
            let u = i as f64 / (size - 1) as f64;
            let v = j as f64 / (size - 1) as f64;
            poles.push([
                u * 100.0,
                v * 100.0,
                (u * 8.0).sin() * (v * 8.0).cos() * 5.0,
            ]);
        }
    }
    NurbsSurface {
        degree: [degree, degree],
        size: [size, size],
        weights: vec![1.0; size * size],
        poles,
        knots: [knots.clone(), knots],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

#[test]
fn surface_edit_grid_is_fast_on_a_large_surface() {
    let limits = Limits::default();
    let surface = large_surface(96);
    let started = std::time::Instant::now();
    let points = om9_core::surface_kernel::edit_points(&surface, &limits).unwrap();
    let grid_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(points.len(), 96 * 96);
    assert!(grid_ms < 1500.0, "edit-point grid took {grid_ms:.3} ms");
    let entity = Entity::new("large", Geometry::Surface(surface), None);
    let (document, ids) = doc_with(vec![entity]);
    let started = std::time::Instant::now();
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    let handles_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(handles.len(), 96 * 96);
    assert!(handles_ms < 1500.0, "edit handles took {handles_ms:.3} ms");
    let started = std::time::Instant::now();
    let target = [
        handles[48 * 96 + 48].point[0] + 1.0,
        handles[48 * 96 + 48].point[1],
        handles[48 * 96 + 48].point[2],
    ];
    move_grip_geometry(&document, handles[48 * 96 + 48], target, &limits).unwrap();
    let move_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(move_ms < 500.0, "one edit move took {move_ms:.3} ms");
    println!("surface_edit_points_96_ms={grid_ms:.3}");
    println!("surface_edit_handles_96_ms={handles_ms:.3}");
    println!("surface_edit_move_96_ms={move_ms:.3}");
}

#[test]
fn surface_edit_grips_are_capped_for_large_grids() {
    let limits = Limits::default();
    let entity = Entity::new("huge", Geometry::Surface(large_surface(101)), None);
    let (document, ids) = doc_with(vec![entity]);
    let error = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap_err();
    assert!(error.to_string().contains("10,000"), "{error}");
    assert!(grip_handles(&document, &ids, GripMode::Control, &limits).is_ok());
}
