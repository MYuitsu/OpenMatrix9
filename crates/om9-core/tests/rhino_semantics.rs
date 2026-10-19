// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L2-02-01` — Rhino-like object semantics.
//!
//! Every expected value is structural (kind, count, domain, loop, class) or analytic (a 10x20x30
//! box, a unit split), so a wrong classifier cannot pass by matching its own output.

use om9_core::{
    Error,
    brep::{Brep, BrepDisplay},
    brep_kernel::{BrepBody, Edge, Face, ShapeKind, Shell, Vertex, Wire},
    budget::Limits,
    curve_build,
    geometry::{Curve, Geometry, Mesh, NurbsCurve, Point3, Transform},
    identity::ObjectId,
    model::{Definition, Document, Entity},
    rhino_semantics::{
        BoundaryEdge, CurveForm, DisplayIsocurve, Extrusion, SemanticKind, ShapeClass,
        SurfaceExpectation, TrimmedSurface, block_instances, classify_curve, classify_shell,
        classify_surface, clear_trimmed_surface, entity_semantic_kind, extrusion_of,
        geometry_semantic_kind, inspect_nurbs_curve, is_exact_geometry, require_exact_nurbs,
        set_instance_transform, set_trimmed_surface, trimmed_surface_of,
    },
    surface_kernel::{self, SurfaceIso},
};
use uuid::Uuid;

fn limits() -> Limits {
    Limits::default()
}

fn point(x: f64, y: f64, z: f64) -> Point3 {
    [x, y, z]
}

fn line(a: Point3, b: Point3) -> Curve {
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, 1.0],
    }
}

fn cubic(planar: bool) -> NurbsCurve {
    let poles = if planar {
        vec![
            point(0.0, 0.0, 0.0),
            point(1.0, 0.0, 0.0),
            point(1.0, 1.0, 0.0),
            point(0.0, 1.0, 0.0),
        ]
    } else {
        vec![
            point(0.0, 0.0, 0.0),
            point(1.0, 0.0, 0.0),
            point(1.0, 1.0, 0.0),
            point(0.0, 1.0, 1.0),
        ]
    };
    NurbsCurve {
        degree: 3,
        poles,
        weights: vec![1.0; 4],
        knots: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    }
}

fn circle() -> Curve {
    Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 5.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    }
}

#[test]
fn point_line_polyline_and_polycurve_have_distinct_kinds() {
    let polyline = curve_build::polyline(&[
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(1.0, 1.0, 0.0),
    ])
    .unwrap();
    let disconnected = Curve::Polycurve {
        segments: vec![
            line(point(0.0, 0.0, 0.0), point(1.0, 0.0, 0.0)),
            line(point(2.0, 0.0, 0.0), point(3.0, 0.0, 0.0)),
        ],
        reversed: false,
    };
    assert_eq!(
        geometry_semantic_kind(&Geometry::Point(point(1.0, 2.0, 3.0))),
        SemanticKind::Point
    );
    assert_eq!(
        geometry_semantic_kind(&Geometry::Curve(line(
            point(0.0, 0.0, 0.0),
            point(1.0, 0.0, 0.0)
        ))),
        SemanticKind::Line
    );
    assert_eq!(
        geometry_semantic_kind(&Geometry::Curve(polyline)),
        SemanticKind::Polyline
    );
    assert_eq!(
        geometry_semantic_kind(&Geometry::Curve(disconnected.clone())),
        SemanticKind::Polycurve
    );
    assert_eq!(
        geometry_semantic_kind(&Geometry::Curve(circle())),
        SemanticKind::Circle
    );
    let measured = classify_curve(&disconnected, 1.0e-9, &limits()).unwrap();
    assert_eq!(measured.form, CurveForm::Polycurve);
    assert!(!measured.closed);

    let straight = line(point(0.0, 0.0, 0.0), point(2.0, 0.0, 0.0));
    let measured = classify_curve(&straight, 1.0e-9, &limits()).unwrap();
    assert_eq!(measured.form, CurveForm::Line);
    assert_eq!(
        measured.endpoints,
        [point(0.0, 0.0, 0.0), point(2.0, 0.0, 0.0)]
    );
}

#[test]
fn a_degenerate_line_is_refused_and_never_becomes_a_point_or_polygon() {
    let limits = limits();
    let exact = Curve::Line {
        start: point(0.0, 0.0, 0.0),
        end: point(0.0, 0.0, 0.0),
        domain: [0.0, 1.0],
    };
    assert_eq!(
        exact.validate(&limits),
        Err(Error::Invalid("degenerate line"))
    );
    let tiny = line(point(0.0, 0.0, 0.0), point(0.0005, 0.0, 0.0));
    assert_eq!(
        classify_curve(&tiny, 0.001, &limits),
        Err(Error::Invalid("degenerate curve within tolerance"))
    );
    // The same record is admitted when the tolerance is stricter than its length.
    let measured = classify_curve(&tiny, 1.0e-6, &limits).unwrap();
    assert_eq!(measured.form, CurveForm::Line);
}

#[test]
fn cubic_nurbs_curve_reports_degree_knots_weights_and_rationality() {
    let curve = cubic(true);
    let measured = inspect_nurbs_curve(&curve, 1.0e-9, &limits()).unwrap();
    assert_eq!(measured.degree, 3);
    assert_eq!(measured.pole_count, 4);
    assert_eq!(measured.weight_count, 4);
    assert_eq!(measured.knot_count, 8);
    assert_eq!(measured.expected_knot_count, 8);
    assert!(!measured.rational);
    assert!(measured.planar);
    assert!(!measured.closed);

    let mut rational = curve.clone();
    rational.weights = vec![1.0, 0.5, 0.5, 1.0];
    assert!(
        inspect_nurbs_curve(&rational, 1.0e-9, &limits())
            .unwrap()
            .rational
    );

    let mut short_knots = curve.clone();
    short_knots.knots.pop();
    assert_eq!(
        inspect_nurbs_curve(&short_knots, 1.0e-9, &limits()),
        Err(Error::Invalid("NURBS knots"))
    );
    let mut zero_weight = curve;
    zero_weight.weights[1] = 0.0;
    assert_eq!(
        inspect_nurbs_curve(&zero_weight, 1.0e-9, &limits()),
        Err(Error::Invalid("supported positive rational weights"))
    );
}

#[test]
fn closed_and_planar_rules_use_the_stated_tolerance_not_raw_equality() {
    let limits = limits();
    let closed_circle = classify_curve(&circle(), 1.0e-9, &limits).unwrap();
    assert!(closed_circle.closed);
    assert!(closed_circle.planar);

    let open_spline = Curve::Nurbs(cubic(false));
    let open = classify_curve(&open_spline, 1.0e-9, &limits).unwrap();
    assert!(!open.closed);
    assert!(!open.planar, "the last pole leaves the z=0 plane");

    // Endpoints are 0.0005 apart while the curve has real length.
    let near_closed = curve_build::polyline(&[
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(1.0, 1.0, 0.0),
        point(0.0005, 0.0, 0.0),
    ])
    .unwrap();
    assert!(classify_curve(&near_closed, 0.001, &limits).unwrap().closed);
    assert!(
        !classify_curve(&near_closed, 0.0001, &limits)
            .unwrap()
            .closed
    );
    assert!(classify_curve(&near_closed, 0.001, &limits).unwrap().planar);
}

fn periodic_surface() -> om9_core::geometry::NurbsSurface {
    let mut surface = surface_kernel::cylinder(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        5.0,
        10.0,
        &limits(),
    )
    .unwrap();
    // Drop the duplicated closing row and use an unclamped U knot vector; the domain starts at
    // the first interior knot. This is the periodic representation the seam rule must accept.
    surface.poles.truncate(16);
    surface.weights.truncate(16);
    surface.size = [8, 2];
    surface.knots[0] = vec![0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0];
    surface.domain[0] = [0.25, 1.0];
    surface.periodic = [true, false];
    surface.validate(&limits()).unwrap();
    surface
}

#[test]
fn surface_domains_seams_periodicity_and_orientation_are_explicit() {
    let limits = limits();
    let plane = surface_kernel::plane(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        10.0,
        20.0,
        &limits,
    )
    .unwrap();
    let plane_semantics = classify_surface(&plane, 1.0e-9, &limits).unwrap();
    assert_eq!(plane_semantics.domain, [[0.0, 1.0], [0.0, 1.0]]);
    assert_eq!(plane_semantics.periodic, [false, false]);
    assert_eq!(plane_semantics.seam, [None, None]);
    assert!(!plane_semantics.v_reversed);

    let cylinder = surface_kernel::cylinder(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        5.0,
        10.0,
        &limits,
    )
    .unwrap();
    let closed = classify_surface(&cylinder, 1.0e-9, &limits).unwrap();
    assert_eq!(closed.periodic, [false, false]);
    assert_eq!(
        closed.seam,
        [Some(0.0), None],
        "a closed but non-periodic surface still has a seam"
    );

    let periodic = periodic_surface();
    let semantics = classify_surface(&periodic, 1.0e-9, &limits).unwrap();
    assert_eq!(semantics.periodic, [true, false]);
    assert_eq!(semantics.seam, [Some(0.25), None]);
    assert_eq!(semantics.domain, [[0.25, 1.0], [0.0, 1.0]]);
    semantics
        .require_orientation(SurfaceExpectation {
            periodic: [true, false],
            v_reversed: false,
        })
        .unwrap();
    assert_eq!(
        semantics.require_orientation(SurfaceExpectation {
            periodic: [false, true],
            v_reversed: false,
        }),
        Err(Error::Invalid("surface U/V orientation"))
    );

    let mut missing = periodic.clone();
    missing.periodic = [false, false];
    assert_eq!(
        classify_surface(&missing, 1.0e-9, &limits),
        Err(Error::Invalid("surface periodic flag missing"))
    );
    let mut clamped_but_periodic = cylinder;
    clamped_but_periodic.periodic = [true, false];
    assert_eq!(
        classify_surface(&clamped_but_periodic, 1.0e-9, &limits),
        Err(Error::Invalid("surface periodic flag mismatch"))
    );
}

fn trimmed_plane() -> TrimmedSurface {
    let plane = surface_kernel::plane(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        10.0,
        10.0,
        &limits(),
    )
    .unwrap();
    TrimmedSurface {
        underlying: plane,
        outer: vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
        inners: vec![vec![[0.4, 0.4], [0.6, 0.4], [0.5, 0.6]]],
        reversed: false,
    }
}

#[test]
fn trimmed_surface_keeps_the_underlying_surface_and_never_promotes_it() {
    let limits = limits();
    let trimmed = trimmed_plane();
    trimmed.validate(&limits).unwrap();
    assert!(trimmed.is_trimmed());
    assert_eq!(
        trimmed.underlying_surface().domain,
        [[0.0, 1.0], [0.0, 1.0]]
    );
    assert_eq!(
        trimmed.to_untrimmed(),
        Err(Error::Unsupported(
            "trimmed surface cannot be promoted to an untrimmed surface"
        ))
    );

    let mut entity = Entity::new(
        "trimmed patch",
        Geometry::Surface(trimmed.underlying.clone()),
        None,
    );
    set_trimmed_surface(&mut entity, &trimmed, &limits).unwrap();
    assert_eq!(
        entity_semantic_kind(&entity).unwrap(),
        SemanticKind::TrimmedSurface
    );
    assert_eq!(
        trimmed_surface_of(&entity, &limits).unwrap(),
        Some(trimmed.clone())
    );

    let extension = trimmed.to_extension().unwrap();
    assert_eq!(
        TrimmedSurface::from_extension(&extension, &limits).unwrap(),
        trimmed
    );
    clear_trimmed_surface(&mut entity);
    assert_eq!(
        entity_semantic_kind(&entity).unwrap(),
        SemanticKind::NurbsSurface
    );

    let mut outside = trimmed.clone();
    outside.outer = vec![[2.0, 0.0], [3.0, 0.0], [3.0, 1.0]];
    assert_eq!(
        outside.validate(&limits),
        Err(Error::Invalid("trim loop outside surface domain"))
    );
    let mut inner_outside = trimmed.clone();
    inner_outside.inners = vec![vec![[0.05, 0.05], [0.1, 0.05], [0.05, 0.1]]];
    assert_eq!(
        inner_outside.validate(&limits),
        Err(Error::Invalid("trim inner loop outside outer loop"))
    );
    let mismatched = Entity::new("other", Geometry::Point(point(0.0, 0.0, 0.0)), None);
    let mut mismatched = mismatched;
    assert_eq!(
        set_trimmed_surface(&mut mismatched, &trimmed, &limits),
        Err(Error::Invalid("trimmed surface requires surface geometry"))
    );
}

#[test]
fn isocurves_are_display_only_and_boundary_edges_are_pickable_topology() {
    let limits = limits();
    let plane = surface_kernel::plane(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let isocurve = DisplayIsocurve::from_surface(&plane, SurfaceIso::U, 0.5, &limits).unwrap();
    assert!(isocurve.is_display_only());
    assert!(!isocurve.is_model_edge());
    assert!(!isocurve.is_pickable());
    assert_eq!(isocurve.iso(), SurfaceIso::U);
    assert_eq!(isocurve.value(), 0.5);
    assert_eq!(
        isocurve.to_geometry(),
        Err(Error::Unsupported("isocurve is display-only"))
    );

    let edge = BoundaryEdge::new(ObjectId::new(), Uuid::new_v4(), Uuid::new_v4()).unwrap();
    assert!(edge.is_model_edge());
    assert!(edge.is_pickable());
    assert!(!edge.is_display_only());
    assert_eq!(
        BoundaryEdge::new(ObjectId::new(), Uuid::nil(), Uuid::new_v4()),
        Err(Error::Invalid("boundary edge identity"))
    );
}

fn quad_body(
    vertices: Vec<Vertex>,
    tolerance: f64,
    kind: ShapeKind,
    closed_shell: bool,
    faces: &[[u32; 4]],
) -> BrepBody {
    let mut edges = Vec::new();
    let mut wires = Vec::new();
    let mut face_records = Vec::new();
    for quad in faces {
        let mut wire_edges = Vec::new();
        for i in 0..4 {
            let start = quad[i];
            let end = quad[(i + 1) % 4];
            wire_edges.push(edges.len() as u32);
            edges.push(Edge {
                start,
                end,
                tolerance,
                closed: false,
            });
        }
        wires.push(Wire {
            edges: wire_edges,
            closed: true,
        });
        face_records.push(Face {
            outer: (wires.len() - 1) as u32,
            inners: vec![],
            tolerance,
        });
    }
    BrepBody {
        vertices,
        edges,
        wires,
        faces: face_records,
        shells: vec![Shell {
            faces: (0..faces.len() as u32).collect(),
            closed: closed_shell,
        }],
        kind,
        tolerance,
    }
}

fn cube(closed: bool, kind: ShapeKind) -> BrepBody {
    let tolerance = 0.001;
    let vertices = vec![
        Vertex {
            point: point(0.0, 0.0, 0.0),
            tolerance,
        },
        Vertex {
            point: point(10.0, 0.0, 0.0),
            tolerance,
        },
        Vertex {
            point: point(10.0, 20.0, 0.0),
            tolerance,
        },
        Vertex {
            point: point(0.0, 20.0, 0.0),
            tolerance,
        },
        Vertex {
            point: point(0.0, 0.0, 30.0),
            tolerance,
        },
        Vertex {
            point: point(10.0, 0.0, 30.0),
            tolerance,
        },
        Vertex {
            point: point(10.0, 20.0, 30.0),
            tolerance,
        },
        Vertex {
            point: point(0.0, 20.0, 30.0),
            tolerance,
        },
    ];
    quad_body(
        vertices,
        tolerance,
        kind,
        closed,
        &[
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [1, 2, 6, 5],
            [2, 3, 7, 6],
            [3, 0, 4, 7],
        ],
    )
}

#[test]
fn closed_shell_is_a_solid_while_open_shells_are_surface_or_polysurface() {
    let limits = limits();
    let solid = cube(true, ShapeKind::Solid);
    assert_eq!(classify_shell(&solid, &limits).unwrap(), ShapeClass::Solid);

    let mut open = cube(false, ShapeKind::Shell);
    open.faces.pop();
    open.wires.pop();
    open.shells[0].faces.pop();
    assert_eq!(
        classify_shell(&open, &limits).unwrap(),
        ShapeClass::Polysurface
    );

    let single = quad_body(
        open.vertices.clone(),
        open.tolerance,
        ShapeKind::Shell,
        false,
        &[[0, 3, 2, 1]],
    );
    assert_eq!(
        classify_shell(&single, &limits).unwrap(),
        ShapeClass::Surface
    );

    let mut lying = cube(false, ShapeKind::Solid);
    lying.shells[0].closed = false;
    assert_eq!(
        classify_shell(&lying, &limits),
        Err(Error::Invalid("open shell is not a solid"))
    );

    // A second, distinct-index face over the same four corners is a self-overlap, never a solid.
    let mut overlapping = cube(true, ShapeKind::Solid);
    let first_face_quad = [0, 3, 2, 1];
    let wire = overlapping.wires.len() as u32;
    let mut edges = Vec::new();
    for i in 0..4 {
        let start = first_face_quad[i];
        let end = first_face_quad[(i + 1) % 4];
        edges.push(overlapping.edges.len() as u32);
        overlapping.edges.push(Edge {
            start,
            end,
            tolerance: overlapping.tolerance,
            closed: false,
        });
    }
    overlapping.wires.push(Wire {
        edges,
        closed: true,
    });
    overlapping.faces.push(Face {
        outer: wire,
        inners: vec![],
        tolerance: overlapping.tolerance,
    });
    overlapping.shells[0]
        .faces
        .push((overlapping.faces.len() - 1) as u32);
    assert_eq!(
        classify_shell(&overlapping, &limits),
        Err(Error::Invalid("self-overlapping shell"))
    );
}

fn rectangle_profile() -> Curve {
    curve_build::rectangle(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
        10.0,
        20.0,
    )
    .unwrap()
}

fn extrusion() -> Extrusion {
    Extrusion {
        profile: rectangle_profile(),
        direction: point(0.0, 0.0, 1.0),
        height: 30.0,
        capped: true,
    }
}

#[test]
fn extrusion_stays_lightweight_and_converts_to_a_closed_brep_with_provenance() {
    let limits = limits();
    let extrusion = extrusion();
    extrusion.validate(1.0e-6, &limits).unwrap();
    let conversion = extrusion.convert(1.0e-6, &limits).unwrap();
    assert_eq!(conversion.source, extrusion);
    assert_eq!(conversion.body.kind, ShapeKind::Solid);
    assert_eq!(conversion.body.vertices.len(), 8);
    assert_eq!(conversion.body.faces.len(), 6);
    assert_eq!(conversion.body.wires.len(), 6);
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for vertex in &conversion.body.vertices {
        for axis in 0..3 {
            min[axis] = min[axis].min(vertex.point[axis]);
            max[axis] = max[axis].max(vertex.point[axis]);
        }
    }
    assert_eq!(min, [0.0, 0.0, 0.0]);
    assert_eq!(max, [10.0, 20.0, 30.0]);
    assert_eq!(
        classify_shell(&conversion.body, &limits).unwrap(),
        ShapeClass::Solid
    );

    let mut entity = Entity::new(
        "extrusion",
        Geometry::Curve(extrusion.profile.clone()),
        None,
    );
    om9_core::rhino_semantics::set_extrusion(&mut entity, &extrusion, &limits).unwrap();
    assert_eq!(
        entity_semantic_kind(&entity).unwrap(),
        SemanticKind::Extrusion
    );
    assert_eq!(
        extrusion_of(&entity, &limits).unwrap(),
        Some(extrusion.clone())
    );
    let extension = extrusion.to_extension().unwrap();
    assert_eq!(
        Extrusion::from_extension(&extension, 1.0e-6, &limits).unwrap(),
        extrusion
    );

    let mut open = extrusion.clone();
    open.profile = curve_build::polyline(&[
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(1.0, 1.0, 0.0),
    ])
    .unwrap();
    assert_eq!(
        open.validate(1.0e-6, &limits),
        Err(Error::Invalid("extrusion profile must be closed"))
    );
    let mut zero = extrusion.clone();
    zero.height = 0.0;
    assert_eq!(
        zero.validate(1.0e-6, &limits),
        Err(Error::Invalid("extrusion height"))
    );
    let mut direction = extrusion;
    direction.direction = point(0.0, 0.0, 0.0);
    assert_eq!(
        direction.validate(1.0e-6, &limits),
        Err(Error::Invalid("extrusion direction"))
    );
}

fn mesh_cube() -> Mesh {
    let vertices = vec![
        point(0.0, 0.0, 0.0),
        point(10.0, 0.0, 0.0),
        point(10.0, 20.0, 0.0),
        point(0.0, 20.0, 0.0),
        point(0.0, 0.0, 30.0),
        point(10.0, 0.0, 30.0),
        point(10.0, 20.0, 30.0),
        point(0.0, 20.0, 30.0),
    ];
    Mesh {
        vertices,
        faces: vec![
            vec![0, 3, 2, 1],
            vec![4, 5, 6, 7],
            vec![0, 1, 5, 4],
            vec![1, 2, 6, 5],
            vec![2, 3, 7, 6],
            vec![3, 0, 4, 7],
        ],
        normals: vec![],
        colors: vec![],
    }
}

fn brep_box() -> Brep {
    let points = vec![
        point(0.0, 0.0, 0.0),
        point(10.0, 0.0, 0.0),
        point(10.0, 20.0, 0.0),
        point(0.0, 20.0, 0.0),
        point(0.0, 0.0, 30.0),
        point(10.0, 0.0, 30.0),
        point(10.0, 20.0, 30.0),
        point(0.0, 20.0, 30.0),
    ];
    let triangles = vec![
        [0, 2, 1, 0],
        [0, 3, 2, 0],
        [4, 5, 6, 1],
        [4, 6, 7, 1],
        [0, 1, 5, 2],
        [0, 5, 4, 2],
        [2, 3, 7, 3],
        [2, 7, 6, 3],
        [1, 2, 6, 4],
        [1, 6, 5, 4],
        [0, 4, 7, 5],
        [0, 7, 3, 5],
    ];
    let faces = (0..6).map(|_| Uuid::new_v4()).collect();
    let edges = (0..12).map(|_| Uuid::new_v4()).collect();
    let vertex_ids = (0..8).map(|_| Uuid::new_v4()).collect();
    let display = BrepDisplay {
        source_sha256: [0; 32],
        deflection_mm: 0.01,
        points,
        triangles,
        segments: vec![[0, 1, 0], [1, 2, 1], [2, 3, 2], [3, 0, 3]],
        vertices: (0..8).collect(),
        faces,
        edges,
        vertex_ids,
        face_edges: vec![[0, 0], [0, 1], [0, 2], [0, 3]],
    };
    Brep::from_native(
        "CASCADE Topology synthetic box".into(),
        display,
        &Limits::default(),
    )
    .unwrap()
}

#[test]
fn a_mesh_and_an_exact_box_never_share_a_semantic_kind() {
    let mesh = Geometry::Mesh(mesh_cube());
    let brep = Geometry::Brep(brep_box());
    assert_eq!(geometry_semantic_kind(&mesh), SemanticKind::Mesh);
    assert_eq!(geometry_semantic_kind(&brep), SemanticKind::Brep);
    assert!(!is_exact_geometry(&mesh));
    assert!(is_exact_geometry(&brep));
    assert_eq!(
        require_exact_nurbs(&mesh),
        Err(Error::Unsupported("mesh is not exact NURBS"))
    );
    require_exact_nurbs(&brep).unwrap();
    // The same twelve display triangles do not make the mesh exact.
    let mesh_triangles: usize = mesh_cube().faces.iter().map(|face| face.len() - 2).sum();
    assert_eq!(mesh_triangles, 12);
}

#[test]
fn block_instances_share_a_definition_but_not_identity_or_transform() {
    let limits = limits();
    let mut document = Document::new("blocks");
    let definition = Definition::new(
        "shared block",
        vec![Entity::new(
            "member",
            Geometry::Point(point(0.0, 0.0, 0.0)),
            None,
        )],
    );
    let definition_id = definition.id;
    let definition_before = definition.clone();
    document.definitions.insert(definition_id, definition);
    let first = document
        .instance(definition_id, Transform::translation([0.0, 0.0, 0.0]), None)
        .unwrap();
    let second = document
        .instance(
            definition_id,
            Transform::translation([50.0, 0.0, 0.0]),
            None,
        )
        .unwrap();
    let first_id = first.id;
    let second_id = second.id;
    document.entities.insert(first_id, first);
    document.entities.insert(second_id, second);
    document.validate(&limits).unwrap();

    let instances = block_instances(&document, definition_id).unwrap();
    assert_eq!(instances.len(), 2);
    assert_eq!(instances[0].definition, definition_id);
    assert_eq!(instances[1].definition, definition_id);
    assert_ne!(instances[0].transform, instances[1].transform);
    assert!(instances.iter().any(|instance| instance.object == first_id));
    assert!(
        instances
            .iter()
            .any(|instance| instance.object == second_id)
    );

    set_instance_transform(
        &mut document,
        first_id,
        Transform::translation([5.0, 2.0, 1.0]),
    )
    .unwrap();
    document.validate(&limits).unwrap();
    assert_eq!(document.definitions[&definition_id], definition_before);
    assert_eq!(
        document.entities[&second_id].placement,
        Transform::translation([50.0, 0.0, 0.0])
    );
    assert_eq!(
        document.entities[&first_id].placement,
        Transform::translation([5.0, 2.0, 1.0])
    );
    assert_eq!(
        block_instances(&document, om9_core::identity::DefinitionId::new()),
        Err(Error::Missing("instance definition"))
    );
}
