// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused acceptance tests for M03 `OM9-L2-01-09` geometry creation operations.
//!
//! Every test drives the public Rust contract and asserts a measured oracle from the exact OCCT
//! B-Rep (volume/area/topology/bounds), never a display mesh and never a copied planning value.
//! Run one owner at a time, for example:
//!
//! `cargo test -p om9-kernel --features 3dm-brep --test l2_01_09 m03_01_09_001 -- --exact`
use om9_core::budget::Limits;
use om9_kernel::{
    brep::{read_brep, source_volume},
    creation::{
        self, CreationError, CreationSession, DEFAULT_DEFLECTION_MM, EntityKind, Primitive, Profile,
    },
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

fn assert_volume(brep: &om9_core::brep::Brep, expected: f64, tolerance: f64) {
    let measured = creation::measure(brep, &limits()).expect("measure");
    assert!(
        close(measured.volume_mm3, expected, tolerance),
        "volume {} != {expected} (tolerance {tolerance})",
        measured.volume_mm3
    );
    assert_eq!(measured.solids, 1, "expected one closed solid");
    assert!(measured.closed, "expected a closed solid loop");
}

fn rectangle(x: f64, y: f64, z: f64) -> Profile {
    Profile::new(vec![[0.0, 0.0, z], [x, 0.0, z], [x, y, z], [0.0, y, z]])
        .expect("rectangle profile")
}

/* ------------------------- 001 analytic primitives ------------------------- */

#[test]
fn m03_01_09_001() {
    let limits = limits();
    // Box(10,20,30): exact volume and independent bounding dimensions.
    let boxed = creation::primitive(
        Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        deflection(),
        &limits,
    )
    .expect("box");
    assert_volume(&boxed, 6000.0, 1e-6);
    let measured = creation::measure(&boxed, &limits).expect("box measure");
    let size = measured.size_mm();
    assert!(close(size[0], 10.0, 1e-6), "box x {}", size[0]);
    assert!(close(size[1], 20.0, 1e-6), "box y {}", size[1]);
    assert!(close(size[2], 30.0, 1e-6), "box z {}", size[2]);
    // Round trip: the canonical source reopens to the same measured solid.
    let reopened = read_brep(boxed.source(), deflection(), &limits).expect("reopen box");
    assert_volume(&reopened, 6000.0, 1e-6);
    let reopened_volume = source_volume(&reopened, &limits).expect("reopened volume");
    assert!(close(reopened_volume, 6000.0, 1e-6));

    // Sphere(r=5).
    let sphere = creation::primitive(Primitive::Sphere { radius_mm: 5.0 }, deflection(), &limits)
        .expect("sphere");
    assert_volume(&sphere, 4.0 / 3.0 * PI * 125.0, 1e-6);

    // Cylinder(r=5, h=10).
    let cylinder = creation::primitive(
        Primitive::Cylinder {
            radius_mm: 5.0,
            height_mm: 10.0,
        },
        deflection(),
        &limits,
    )
    .expect("cylinder");
    assert_volume(&cylinder, PI * 25.0 * 10.0, 1e-6);

    // Cone(r=5, h=10) is the same factory with a zero top radius.
    let cone = creation::primitive(
        Primitive::Cone {
            bottom_radius_mm: 5.0,
            top_radius_mm: 0.0,
            height_mm: 10.0,
        },
        deflection(),
        &limits,
    )
    .expect("cone");
    assert_volume(&cone, PI * 25.0 * 10.0 / 3.0, 1e-6);

    // Torus(R=10, r=2).
    let torus = creation::primitive(
        Primitive::Torus {
            ring_radius_mm: 10.0,
            tube_radius_mm: 2.0,
        },
        deflection(),
        &limits,
    )
    .expect("torus");
    assert_volume(&torus, 2.0 * PI * PI * 10.0 * 4.0, 1e-6);

    // Negative fixture: zero/negative/NaN sizes are refused before any shape exists.
    for primitive in [
        Primitive::Sphere { radius_mm: 0.0 },
        Primitive::Sphere {
            radius_mm: f64::NAN,
        },
        Primitive::Cylinder {
            radius_mm: 5.0,
            height_mm: -1.0,
        },
        Primitive::Box {
            x_mm: 0.0,
            y_mm: 1.0,
            z_mm: 1.0,
        },
        Primitive::Torus {
            ring_radius_mm: 2.0,
            tube_radius_mm: 2.0,
        },
        Primitive::Pyramid {
            sides: 2,
            base_radius_mm: 1.0,
            height_mm: 1.0,
        },
    ] {
        assert!(
            matches!(
                creation::primitive(primitive, deflection(), &limits),
                Err(CreationError::Invalid(_))
            ),
            "{primitive:?} must be refused"
        );
    }
}

/* ------------------------- 002 curve to solid extrusion ------------------------- */

#[test]
fn m03_01_09_002() {
    let limits = limits();
    // Closed rectangle 10 × 20 extruded z = 30 mm.
    let profile = rectangle(10.0, 20.0, 0.0);
    let solid = creation::extrude(&profile, [0.0, 0.0, 30.0], true, deflection(), &limits)
        .expect("extrusion");
    assert_volume(&solid, 6000.0, 1e-6);
    let measured = creation::measure(&solid, &limits).expect("extrusion measure");
    assert!(close(measured.size_mm()[2], 30.0, 1e-6));

    // An open wire stays an open side shell: 40 mm open chain × 30 mm height, no cap.
    let open = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 20.0, 0.0],
            [0.0, 20.0, 0.0],
        ],
        [0.0, 0.0, 30.0],
        deflection(),
        &limits,
    )
    .expect("open extrusion");
    let open_measured = creation::measure(&open, &limits).expect("open measure");
    assert_eq!(
        open_measured.solids, 0,
        "an open wire must not become a solid"
    );
    assert_eq!(open_measured.shells, 1);
    assert!(!open_measured.closed);
    assert!(
        close(open_measured.area_mm2, 40.0 * 30.0, 1e-5),
        "open side area {}",
        open_measured.area_mm2
    );

    // Negative: self-crossing wire, zero height, zero direction.
    assert!(matches!(
        Profile::new(vec![
            [0.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [10.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
        ]),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        creation::extrude(&profile, [0.0, 0.0, 0.0], true, deflection(), &limits),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        creation::extrude(&profile, [0.0, 0.0, f64::NAN], true, deflection(), &limits),
        Err(CreationError::Invalid(_))
    ));
}

/* ------------------------- 003 revolve ------------------------- */

#[test]
fn m03_01_09_003() {
    let limits = limits();
    // Closed rectangle in the r-z plane, r = 5..7, z = 0..10, revolved 360° about z.
    let profile = Profile::new(vec![
        [5.0, 0.0, 0.0],
        [7.0, 0.0, 0.0],
        [7.0, 0.0, 10.0],
        [5.0, 0.0, 10.0],
    ])
    .expect("annulus profile");
    let solid = creation::revolve(
        &profile,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        2.0 * PI,
        deflection(),
        &limits,
    )
    .expect("revolve");
    assert_volume(&solid, PI * (49.0 - 25.0) * 10.0, 1e-5);

    // A half turn closes through its two end faces: it is a solid with half the full volume.
    let half = creation::revolve(
        &profile,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        PI,
        deflection(),
        &limits,
    )
    .expect("half revolve");
    assert_volume(&half, 0.5 * PI * (49.0 - 25.0) * 10.0, 1e-5);

    // Negative: zero axis, angle out of domain, profile crossing the axis.
    assert!(matches!(
        creation::revolve(
            &profile,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            2.0 * PI,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        creation::revolve(
            &profile,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            0.0,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
    let crossing = Profile::new(vec![
        [-2.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [2.0, 0.0, 10.0],
        [-2.0, 0.0, 10.0],
    ])
    .expect("crossing profile is still a valid planar profile");
    assert!(matches!(
        creation::revolve(
            &crossing,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            2.0 * PI,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
}

/* ------------------------- 004 pipe ------------------------- */

#[test]
fn oriented_hemisphere_has_measured_volume_and_bounds() {
    for forward in [false,true] {
        let body=creation::hemisphere([3.,4.,5.],[1.,0.,0.],2.,forward,deflection(),&limits()).unwrap();
        let m=creation::measure(&body,&limits()).unwrap(); assert!(m.closed && m.solids==1);
        assert!((m.volume_mm3-16.*PI/3.).abs()<1e-6,"{m:?}");
        let expected=if forward {[3.,5.]} else {[1.,3.]};
        assert!((m.bounds_mm[0]-expected[0]).abs()<1e-5 && (m.bounds_mm[3]-expected[1]).abs()<1e-5,"{m:?}");
    }
    assert!(creation::hemisphere([0.;3],[0.;3],1.,true,deflection(),&limits()).is_err());
}

#[test]
fn pipe_miter_polyline_builds_exact_right_angle_tube() {
    let limits = limits();
    let path = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]];
    let body = creation::pipe_miter_polyline(&path, 1.0, true, deflection(), &limits).unwrap();
    assert_volume(&body, 20.0 * PI, 1e-5);
    let m = creation::measure(&body, &limits).unwrap();
    assert!(m.closed && m.solids == 1);
    let shell = creation::pipe_miter_polyline(&path, 1.0, false, deflection(), &limits).unwrap();
    assert!(!creation::measure(&shell, &limits).unwrap().closed);
    let reversed = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    assert!(creation::pipe_miter_polyline(&reversed, 1.0, true, deflection(), &limits).is_err());
}

#[test]
fn pipe_stations_straight_builds_piecewise_linear_cones() {
    let limits = limits();
    let path = [[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]];
    let stations = [(0.0, 1.0), (0.5, 2.0), (1.0, 3.0)];
    let brep = creation::pipe_stations_straight(path, &stations, true, deflection(), &limits).unwrap();
    assert_volume(&brep, PI * 130.0 / 3.0, 1e-5);
    assert!(creation::measure(&brep, &limits).unwrap().closed);
    let shell = creation::pipe_stations_straight(path, &stations, false, deflection(), &limits).unwrap();
    assert!(!creation::measure(&shell, &limits).unwrap().closed);
    for invalid in [vec![(0.0, 1.0), (0.5, 2.0), (0.5, 3.0), (1.0, 1.0)],
        vec![(0.1, 1.0), (1.0, 1.0)], vec![(0.0, 1.0), (1.0, 0.0)]] {
        assert!(creation::pipe_stations_straight(path, &invalid, true, deflection(), &limits).is_err());
    }
}

#[test]
fn round_pipe_straight_conical_body_and_hemispheres_are_exact() {
    let limits = limits();
    let path = [[3.0, -2.0, 5.0], [3.0, -2.0, 15.0]];
    for (r0, r1) in [(1.0, 2.0), (2.0, 1.0), (1.0, 1.0)] {
        let brep = creation::pipe_round_straight(path, r0, r1, deflection(), &limits).unwrap();
        let expected = PI * 10.0 * (r0 * r0 + r0 * r1 + r1 * r1) / 3.0
            + 2.0 * PI * (r0.powi(3) + r1.powi(3)) / 3.0;
        assert_volume(&brep, expected, 1e-5);
        let m = creation::measure(&brep, &limits).unwrap();
        assert!(m.closed && m.solids == 1);
        assert!(close(m.bounds_mm[2], 5.0 - r0, 1e-5));
        assert!(close(m.bounds_mm[5], 15.0 + r1, 1e-5));
    }
}

#[test]
fn pipe_linear_straight_frustum_has_exact_volume_and_open_variant() {
    let limits = limits();
    let path = [[3.0, -2.0, 5.0], [3.0, -2.0, 15.0]];
    for (start, end) in [(1.0, 2.0), (2.0, 1.0), (1.0, 1.0)] {
        let body = creation::pipe_linear(&path, start, end, true, deflection(), &limits).unwrap();
        assert_volume(&body, PI * 10.0 * (start * start + start * end + end * end) / 3.0, 1e-6);
        assert!(creation::measure(&body, &limits).unwrap().closed);
    }
    let shell = creation::pipe_linear(&path, 1.0, 2.0, false, deflection(), &limits).unwrap();
    assert!(!creation::measure(&shell, &limits).unwrap().closed);
    for (start, end) in [(0.0, 1.0), (1.0, -1.0), (f64::NAN, 1.0)] {
        assert!(creation::pipe_linear(&path, start, end, true, deflection(), &limits).is_err());
    }
    assert!(creation::pipe_linear(&[path[0], path[0]], 1.0, 2.0, true, deflection(), &limits).is_err());
}

#[test]
fn m03_01_09_004() {
    let limits = limits();
    let pipe = creation::pipe(
        &[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]],
        2.0,
        true,
        deflection(),
        &limits,
    )
    .expect("pipe");
    assert_volume(&pipe, 40.0 * PI, 1e-6);
    let measured = creation::measure(&pipe, &limits).expect("pipe measure");
    assert!(close(measured.size_mm()[2], 10.0, 1e-6));
    assert!(close(measured.size_mm()[0], 4.0, 1e-6));
    // The seam is deterministic: a rebuilt pipe publishes the same topology counts.
    let again = creation::pipe(
        &[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]],
        2.0,
        true,
        deflection(),
        &limits,
    )
    .expect("pipe again");
    let again_measured = creation::measure(&again, &limits).expect("pipe again measure");
    assert_eq!(measured.faces, again_measured.faces);

    // Negative: radius <= 0, zero-length path, repeated path point.
    assert!(matches!(
        creation::pipe(
            &[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]],
            0.0,
            true,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        creation::pipe(
            &[[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            2.0,
            true,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
}

/* ------------------------- 005 planar boundary surface ------------------------- */

#[test]
fn m03_01_09_005() {
    let limits = limits();
    let outer = rectangle(10.0, 20.0, 0.0);
    let face = creation::planar(&outer, deflection(), &limits).expect("planar face");
    let measured = creation::measure(&face, &limits).expect("planar measure");
    assert!(close(measured.area_mm2, 200.0, 1e-6));
    assert_eq!(measured.faces, 1);
    assert_eq!(measured.solids, 0);
    // A standalone face is not a shell in OCCT; it is an open, unclosed surface.
    assert_eq!(measured.shells, 0);
    assert!(!measured.closed);
    assert!(close(outer.area_mm2(), 200.0, 1e-9));

    // Negative: non-coplanar boundary and self-intersecting boundary.
    assert!(matches!(
        Profile::new(vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 20.0, 5.0],
            [0.0, 20.0, 0.0],
        ]),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        Profile::new(vec![
            [0.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [10.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
        ]),
        Err(CreationError::Invalid(_))
    ));
}

/* ------------------------- 006 loft ------------------------- */

#[test]
fn m03_01_09_006() {
    let limits = limits();
    let sections = [
        rectangle(10.0, 10.0, 0.0),
        rectangle(10.0, 10.0, 5.0),
        rectangle(10.0, 10.0, 10.0),
    ];
    let loft = creation::loft(&sections, true, false, deflection(), &limits).expect("loft");
    assert_volume(&loft, 1000.0, 1e-5);
    let measured = creation::measure(&loft, &limits).expect("loft measure");
    assert!(close(measured.bounds_mm[2], 0.0, 1e-6));
    assert!(close(measured.bounds_mm[5], 10.0, 1e-6));

    // Negative: a reversed section must be refused, not silently corrected.
    let mut reversed = rectangle(10.0, 10.0, 5.0);
    let mut points = reversed.points().to_vec();
    points.reverse();
    reversed = Profile::new(points).expect("reversed section is still a valid profile");
    assert!(matches!(
        creation::loft(
            &[
                rectangle(10.0, 10.0, 0.0),
                reversed,
                rectangle(10.0, 10.0, 10.0)
            ],
            true,
            false,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        creation::loft(
            &[rectangle(10.0, 10.0, 0.0)],
            true,
            false,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
}

/* ------------------------- 007 sweep single and double rail ------------------------- */

#[test]
fn m03_01_09_007() {
    let limits = limits();
    // Rectangle 2 × 3 perpendicular to a straight 10 mm rail: 60 mm³.
    let profile = rectangle(2.0, 3.0, 0.0);
    let rail = [[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]];
    let sweep = creation::sweep1(&profile, &rail, true, deflection(), &limits).expect("sweep1");
    assert_volume(&sweep, 60.0, 1e-6);
    let measured = creation::measure(&sweep, &limits).expect("sweep measure");
    assert!(close(measured.size_mm()[0], 2.0, 1e-6));
    assert!(close(measured.size_mm()[1], 3.0, 1e-6));
    assert!(close(measured.size_mm()[2], 10.0, 1e-6));

    // Sweep2: profile spans both rails 3 mm apart, height 2 mm, length 10 mm.
    let spanning = Profile::new(vec![
        [0.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [3.0, 2.0, 0.0],
        [0.0, 2.0, 0.0],
    ])
    .expect("spanning profile");
    let rail_a = [[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]];
    let rail_b = [[3.0, 0.0, 0.0], [3.0, 0.0, 10.0]];
    let two = creation::sweep2(
        &spanning,
        &rail_a,
        &rail_b,
        true,
        true,
        deflection(),
        &limits,
    )
    .expect("sweep2");
    assert_volume(&two, 60.0, 1e-6);

    // Negative: zero tangent, profile not perpendicular, coincident rails, profile not spanning.
    assert!(matches!(
        creation::sweep1(
            &profile,
            &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            true,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
    let upright = Profile::new(vec![
        [0.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [2.0, 0.0, 3.0],
        [0.0, 0.0, 3.0],
    ])
    .expect("upright profile");
    assert!(matches!(
        creation::sweep1(&upright, &rail, true, deflection(), &limits),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        creation::sweep2(
            &spanning,
            &rail_a,
            &rail_a,
            true,
            true,
            deflection(),
            &limits
        ),
        Err(CreationError::Invalid(_))
    ));
    let short = rectangle(2.0, 2.0, 0.0);
    assert!(matches!(
        creation::sweep2(&short, &rail_a, &rail_b, true, true, deflection(), &limits),
        Err(CreationError::Invalid(_))
    ));
}

/* ------------------------- 008 surface from edge curves ------------------------- */

#[test]
fn m03_01_09_008() {
    let limits = limits();
    let curves = vec![
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        vec![[1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        vec![[0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
    ];
    let patch =
        creation::surface_from_edges(&curves, 1.0e-6, deflection(), &limits).expect("filled patch");
    assert!(
        patch.max_deviation_mm <= 1.0e-6,
        "deviation {}",
        patch.max_deviation_mm
    );
    let measured = creation::measure(&patch.brep, &limits).expect("patch measure");
    assert!(
        close(measured.area_mm2, 1.0, 1e-6),
        "area {}",
        measured.area_mm2
    );
    assert_eq!(measured.faces, 1);
    assert_eq!(measured.solids, 0);

    // Negative: a boundary gap beyond tolerance and an impossible curve count.
    let mut gap = curves.clone();
    gap[2] = vec![[1.0, 1.0, 0.0], [0.0, 1.5, 0.0]];
    assert!(matches!(
        creation::surface_from_edges(&gap, 1.0e-6, deflection(), &limits),
        Err(CreationError::Invalid(_))
    ));
    assert!(matches!(
        creation::surface_from_edges(&curves[..1], 1.0e-6, deflection(), &limits),
        Err(CreationError::Invalid(_))
    ));
}

/* ------------------------- 009 general profile to B-Rep adapter ------------------------- */

#[test]
fn m03_01_09_009() {
    let limits = limits();
    let profile = rectangle(10.0, 20.0, 0.0);
    let face = creation::profile_to_brep(&profile, deflection(), &limits).expect("face");
    let measured = creation::measure(&face, &limits).expect("face measure");
    assert!(close(measured.area_mm2, 200.0, 1e-6));

    // Handle session: owned publication, named entities, stale after release.
    let mut session = CreationSession::new(limits, deflection()).expect("session");
    let handle = session.build_profile(&profile).expect("publish face");
    assert_eq!(session.live(), 1);
    let names = session.names(handle).expect("names");
    assert!(names.iter().any(|name| name.kind == EntityKind::Face));
    assert!(names.iter().any(|name| name.kind == EntityKind::Edge));
    assert!(
        names
            .iter()
            .filter(|name| name.kind == EntityKind::Face)
            .all(|name| name.name().starts_with("face["))
    );
    let ids: std::collections::BTreeSet<_> = names.iter().map(|name| name.id).collect();
    assert_eq!(ids.len(), names.len(), "names must be unique");
    let body = session.body(handle).expect("live body");
    assert!(close(
        creation::measure(body, &limits)
            .expect("live measure")
            .area_mm2,
        200.0,
        1e-6
    ));
    session.release(handle).expect("release");
    assert_eq!(session.live(), 0);
    assert!(matches!(
        session.body(handle),
        Err(CreationError::StaleHandle)
    ));
    assert!(matches!(
        session.release(handle),
        Err(CreationError::StaleHandle)
    ));
    // A new body reuses the slot with a new generation; the old handle stays stale.
    let fresh = session.build_profile(&profile).expect("fresh body");
    assert!(!session.body(handle).is_ok());
    assert!(session.body(fresh).is_ok());

    // Negative: degenerate profile, and a failed build leaves no live body.
    assert!(matches!(
        Profile::new(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]),
        Err(CreationError::Invalid(_))
    ));
    let mut empty = CreationSession::new(limits, deflection()).expect("empty session");
    assert!(matches!(
        empty.build_primitive(Primitive::Sphere { radius_mm: -1.0 }),
        Err(CreationError::Invalid(_))
    ));
    assert_eq!(empty.live(), 0, "a refused build must not publish a body");
}

/* ------------------------- regression and stress ------------------------- */

#[test]
fn m03_01_09_regression_every_operation_is_a_valid_exact_brep() {
    let limits = limits();
    let boxed = creation::primitive(
        Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        deflection(),
        &limits,
    )
    .expect("box");
    let reopened = read_brep(boxed.source(), deflection(), &limits).expect("reopen");
    assert_eq!(
        reopened.display.faces.len(),
        boxed.display.faces.len(),
        "topology must survive the canonical source round trip"
    );
    let profile = rectangle(10.0, 20.0, 0.0);
    for brep in [
        creation::extrude(&profile, [0.0, 0.0, 30.0], true, deflection(), &limits)
            .expect("extrude"),
        creation::planar(&profile, deflection(), &limits).expect("planar"),
        creation::revolve(
            &Profile::new(vec![
                [5.0, 0.0, 0.0],
                [7.0, 0.0, 0.0],
                [7.0, 0.0, 10.0],
                [5.0, 0.0, 10.0],
            ])
            .expect("annulus"),
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            2.0 * PI,
            deflection(),
            &limits,
        )
        .expect("revolve"),
        creation::loft(
            &[
                rectangle(10.0, 10.0, 0.0),
                rectangle(10.0, 10.0, 5.0),
                rectangle(10.0, 10.0, 10.0),
            ],
            true,
            false,
            deflection(),
            &limits,
        )
        .expect("loft"),
        creation::pipe(
            &[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]],
            2.0,
            true,
            deflection(),
            &limits,
        )
        .expect("pipe"),
    ] {
        let measured = creation::measure(&brep, &limits).expect("measure");
        assert!(measured.faces > 0);
        assert!(!brep.source().is_empty());
        let reopened = read_brep(brep.source(), deflection(), &limits).expect("reopen");
        assert!(close(
            source_volume(&reopened, &limits).expect("reopened volume"),
            measured.volume_mm3,
            1e-5
        ));
    }
}

#[test]
fn m03_01_09_stress_a_sweep_family_keeps_its_volume_and_topology() {
    let limits = limits();
    let profile = rectangle(2.0, 3.0, 0.0);
    for step in [0.5, 1.0, 2.0, 5.0] {
        let rail = [[0.0, 0.0, 0.0], [0.0, 0.0, step]];
        let sweep =
            creation::sweep1(&profile, &rail, true, deflection(), &limits).expect("sweep step");
        let measured = creation::measure(&sweep, &limits).expect("measure step");
        assert!(
            close(measured.volume_mm3, 6.0 * step, 1e-6),
            "step {step}: volume {}",
            measured.volume_mm3
        );
        assert_eq!(measured.solids, 1);
    }
    // A 64-line profile and a 33-station rail stay within budget and stay valid.
    let count = 64;
    let circle: Vec<[f64; 3]> = (0..count)
        .map(|index| {
            let angle = 2.0 * PI * index as f64 / count as f64;
            [2.0 * angle.cos(), 2.0 * angle.sin(), 0.0]
        })
        .collect();
    let round = Profile::new(circle).expect("64-line circle");
    let rail: Vec<[f64; 3]> = (0..33)
        .map(|index| [0.0, 0.0, index as f64 * 0.5])
        .collect();
    let swept = creation::sweep1(&round, &rail, true, deflection(), &limits).expect("long sweep");
    let measured = creation::measure(&swept, &limits).expect("long measure");
    assert_eq!(measured.solids, 1);
    assert!(measured.volume_mm3 > 0.0);
}

/* ------------------------- measured receipt ------------------------- */

/// Prints the measured oracle for every L3 fixture as one JSON object. The values are measured from
/// the exact B-Rep on this run; the same test asserts them, so a printed line is also a pass.
#[test]
fn m03_01_09_receipt() {
    let limits = limits();
    let mut report = String::from("{\"measured\":{");
    let mut push = |name: &str, measured: f64| {
        report.push_str(&format!("\"{name}\":{measured:.12},"));
    };
    let boxed = creation::primitive(
        Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        deflection(),
        &limits,
    )
    .expect("box");
    let box_measures = creation::measure(&boxed, &limits).expect("box measure");
    push("box_volume_mm3", box_measures.volume_mm3);
    assert!(close(box_measures.volume_mm3, 6000.0, 1e-6));
    let sphere = creation::primitive(Primitive::Sphere { radius_mm: 5.0 }, deflection(), &limits)
        .expect("sphere");
    let sphere_measures = creation::measure(&sphere, &limits).expect("sphere measure");
    push("sphere_volume_mm3", sphere_measures.volume_mm3);
    assert!(close(
        sphere_measures.volume_mm3,
        4.0 / 3.0 * PI * 125.0,
        1e-6
    ));
    let cylinder = creation::primitive(
        Primitive::Cylinder {
            radius_mm: 5.0,
            height_mm: 10.0,
        },
        deflection(),
        &limits,
    )
    .expect("cylinder");
    let cylinder_measures = creation::measure(&cylinder, &limits).expect("cylinder measure");
    push("cylinder_volume_mm3", cylinder_measures.volume_mm3);
    assert!(close(cylinder_measures.volume_mm3, PI * 250.0, 1e-6));
    let cone = creation::primitive(
        Primitive::Cone {
            bottom_radius_mm: 5.0,
            top_radius_mm: 0.0,
            height_mm: 10.0,
        },
        deflection(),
        &limits,
    )
    .expect("cone");
    let cone_measures = creation::measure(&cone, &limits).expect("cone measure");
    push("cone_volume_mm3", cone_measures.volume_mm3);
    assert!(close(cone_measures.volume_mm3, PI * 250.0 / 3.0, 1e-6));
    let torus = creation::primitive(
        Primitive::Torus {
            ring_radius_mm: 10.0,
            tube_radius_mm: 2.0,
        },
        deflection(),
        &limits,
    )
    .expect("torus");
    let torus_measures = creation::measure(&torus, &limits).expect("torus measure");
    push("torus_volume_mm3", torus_measures.volume_mm3);
    assert!(close(torus_measures.volume_mm3, 2.0 * PI * PI * 40.0, 1e-6));
    let ellipsoid = creation::primitive(
        Primitive::Ellipsoid {
            x_mm: 3.0,
            y_mm: 4.0,
            z_mm: 5.0,
        },
        deflection(),
        &limits,
    )
    .expect("ellipsoid");
    let ellipsoid_measures = creation::measure(&ellipsoid, &limits).expect("ellipsoid measure");

    push("ellipsoid_volume_mm3", ellipsoid_measures.volume_mm3);
    // Exact rational net, measured by adaptive integration of the stored B-Rep.
    assert!(close(
        ellipsoid_measures.volume_mm3,
        4.0 / 3.0 * PI * 60.0,
        1e-8
    ));
    let pyramid = creation::primitive(
        Primitive::Pyramid {
            sides: 4,
            base_radius_mm: 10.0,
            height_mm: 10.0,
        },
        deflection(),
        &limits,
    )
    .expect("pyramid");
    let pyramid_measures = creation::measure(&pyramid, &limits).expect("pyramid measure");
    push("pyramid_volume_mm3", pyramid_measures.volume_mm3);
    assert!(close(
        pyramid_measures.volume_mm3,
        2.0 * 100.0 * 10.0 / 3.0,
        1e-5
    ));

    let profile = rectangle(10.0, 10.0, 0.0);
    let extrusion = creation::extrude(&profile, [0.0, 0.0, 30.0], true, deflection(), &limits)
        .expect("extrude");
    let extrusion_measures = creation::measure(&extrusion, &limits).expect("extrude measure");
    push("extrusion_volume_mm3", extrusion_measures.volume_mm3);
    assert!(close(extrusion_measures.volume_mm3, 3000.0, 1e-6));

    let annulus = Profile::new(vec![
        [5.0, 0.0, 0.0],
        [7.0, 0.0, 0.0],
        [7.0, 0.0, 10.0],
        [5.0, 0.0, 10.0],
    ])
    .expect("annulus");
    let revolve = creation::revolve(
        &annulus,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        2.0 * PI,
        deflection(),
        &limits,
    )
    .expect("revolve");
    let revolve_measures = creation::measure(&revolve, &limits).expect("revolve measure");
    push("revolve_volume_mm3", revolve_measures.volume_mm3);
    assert!(close(revolve_measures.volume_mm3, PI * 240.0, 1e-5));

    let pipe = creation::pipe(
        &[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]],
        2.0,
        true,
        deflection(),
        &limits,
    )
    .expect("pipe");
    let pipe_measures = creation::measure(&pipe, &limits).expect("pipe measure");
    push("pipe_volume_mm3", pipe_measures.volume_mm3);
    assert!(close(pipe_measures.volume_mm3, 40.0 * PI, 1e-6));

    let face =
        creation::planar(&rectangle(10.0, 20.0, 0.0), deflection(), &limits).expect("planar");
    let face_measures = creation::measure(&face, &limits).expect("planar measure");
    push("planar_area_mm2", face_measures.area_mm2);
    assert!(close(face_measures.area_mm2, 200.0, 1e-6));

    let loft = creation::loft(
        &[
            rectangle(10.0, 10.0, 0.0),
            rectangle(10.0, 10.0, 5.0),
            rectangle(10.0, 10.0, 10.0),
        ],
        true,
        false,
        deflection(),
        &limits,
    )
    .expect("loft");
    let loft_measures = creation::measure(&loft, &limits).expect("loft measure");
    push("loft_volume_mm3", loft_measures.volume_mm3);
    assert!(close(loft_measures.volume_mm3, 1000.0, 1e-5));

    let rail = [[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]];
    let sweep = creation::sweep1(
        &rectangle(2.0, 3.0, 0.0),
        &rail,
        true,
        deflection(),
        &limits,
    )
    .expect("sweep1");
    let sweep_measures = creation::measure(&sweep, &limits).expect("sweep measure");
    push("sweep1_volume_mm3", sweep_measures.volume_mm3);
    assert!(close(sweep_measures.volume_mm3, 60.0, 1e-6));

    let spanning = Profile::new(vec![
        [0.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [3.0, 2.0, 0.0],
        [0.0, 2.0, 0.0],
    ])
    .expect("spanning profile");
    let two = creation::sweep2(
        &spanning,
        &[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]],
        &[[3.0, 0.0, 0.0], [3.0, 0.0, 10.0]],
        true,
        true,
        deflection(),
        &limits,
    )
    .expect("sweep2");
    let two_measures = creation::measure(&two, &limits).expect("sweep2 measure");
    push("sweep2_volume_mm3", two_measures.volume_mm3);
    assert!(close(two_measures.volume_mm3, 60.0, 1e-6));

    let open = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 20.0, 0.0],
            [0.0, 20.0, 0.0],
        ],
        [0.0, 0.0, 30.0],
        deflection(),
        &limits,
    )
    .expect("open extrusion");
    let open_measures = creation::measure(&open, &limits).expect("open measure");
    push("open_extrusion_area_mm2", open_measures.area_mm2);
    assert!(close(open_measures.area_mm2, 1200.0, 1e-5));

    let curves = vec![
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        vec![[1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        vec![[0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
    ];
    let patch = creation::surface_from_edges(&curves, 1.0e-6, deflection(), &limits)
        .expect("surface patch");
    let patch_measures = creation::measure(&patch.brep, &limits).expect("patch measure");
    push("surface_area_mm2", patch_measures.area_mm2);
    push("surface_deviation_mm", patch.max_deviation_mm);
    assert!(close(patch_measures.area_mm2, 1.0, 1e-6));
    assert!(patch.max_deviation_mm <= 1.0e-6);

    let general = creation::profile_to_brep(&rectangle(10.0, 20.0, 0.0), deflection(), &limits)
        .expect("profile face");
    let general_measures = creation::measure(&general, &limits).expect("general measure");
    push("profile_face_area_mm2", general_measures.area_mm2);
    assert!(close(general_measures.area_mm2, 200.0, 1e-6));

    report.pop();
    report.push_str("}}");
    println!("{report}");
}
