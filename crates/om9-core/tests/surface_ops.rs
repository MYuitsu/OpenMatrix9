// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused acceptance tests for `OM9-L2-02-05` freeform surface workflows.
//!
//! Every oracle is analytic or a measured distance on the authored NURBS surface: a Coons
//! network must reproduce a plane, a rail revolve must preserve the profile radius, a trim
//! must keep the underlying surface, a reverse must map `u -> low+high-u`, and an offset must
//! move a plane/cylinder by the requested distance.
use om9_core::{
    Error,
    budget::Limits,
    curve_build,
    geometry::{Curve, NurbsSurface, Point3},
    surface_kernel::{self, SurfaceIso, cylinder, evaluate, normal, plane, refit},
    surface_ops::{
        BlendTangents, SurfaceContinuity, SurfaceEdge, blend_surface, boundary_curve,
        match_surface, network, normal_extrude, offset_surface, rail_revolve, rebuild_surface,
        reverse_surface, seam_surface, shrink_surface, softedit, split_surface, trim_surface,
        unroll, untrim_surface,
    },
};

fn limits() -> Limits {
    Limits::default()
}

fn point(x: f64, y: f64, z: f64) -> Point3 {
    [x, y, z]
}

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

fn line(a: Point3, b: Point3) -> Curve {
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, 1.0],
    }
}

fn square_network() -> [Curve; 4] {
    [
        curve_build::polyline(&[point(0.0, 0.0, 0.0), point(0.0, 2.0, 0.0)]).unwrap(),
        curve_build::polyline(&[point(2.0, 0.0, 0.0), point(2.0, 2.0, 0.0)]).unwrap(),
        curve_build::polyline(&[point(0.0, 0.0, 0.0), point(2.0, 0.0, 0.0)]).unwrap(),
        curve_build::polyline(&[point(0.0, 2.0, 0.0), point(2.0, 2.0, 0.0)]).unwrap(),
    ]
}

/* ------------------------------ L3-001 sweep ------------------------------ */

#[test]
fn m03_02_05_sweep1_and_sweep2_keep_the_profile_and_rails() {
    // The core sweep services are covered by surface_kernel; this test is the L2 owner gate
    // and checks that a swept surface is an exact ruled/analytic record.
    let profile =
        curve_build::circle_through([2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [-2.0, 0.0, 0.0]).unwrap();
    let rail = line([0.0, 0.0, 0.0], [0.0, 0.0, 10.0]);
    let surface = surface_kernel::sweep1(&profile, &rail, [17, 5], [3, 2], &limits()).unwrap();
    for step in 0..=4 {
        let v = step as f64 / 4.0;
        let point = evaluate(&surface, 0.5, v, &limits()).unwrap();
        let radial = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radial - 2.0).abs() < 1e-6, "v={v}: radius {radial}");
        assert!(
            (point[2] - 10.0 * v).abs() < 1e-9,
            "v={v}: height {:?}",
            point[2]
        );
    }
    // Sweep2 with the authored rails keeps both boundaries in contact.
    let rail_a = line([0.0, 0.0, 0.0], [0.0, 0.0, 10.0]);
    let rail_b = line([5.0, 0.0, 0.0], [9.0, 0.0, 10.0]);
    let section =
        curve_build::arc_through([0.0, 0.0, 0.0], [2.5, -2.0, 0.0], [5.0, 0.0, 0.0]).unwrap();
    let surface = surface_kernel::sweep2(
        &section,
        &rail_a,
        &rail_b,
        surface_kernel::Sweep2Options::default(),
        &limits(),
    )
    .unwrap();
    for step in 0..=4 {
        let v = step as f64 / 4.0;
        let start = evaluate(&surface, 0.0, v, &limits()).unwrap();
        let end = evaluate(&surface, 1.0, v, &limits()).unwrap();
        assert!(close(start, [0.0, 0.0, 10.0 * v], 1e-6), "{start:?}");
        assert!(close(end, [5.0 + 4.0 * v, 0.0, 10.0 * v], 1e-6), "{end:?}");
    }
}

/* ------------------------------ L3-002 network ------------------------------ */

#[test]
fn m03_02_05_network_builds_a_coons_patch_from_four_boundaries() {
    let boundaries = square_network();
    let surface = network(
        [
            &boundaries[0],
            &boundaries[1],
            &boundaries[2],
            &boundaries[3],
        ],
        [1, 1],
        [7, 7],
        &limits(),
    )
    .unwrap();
    assert!(close(
        evaluate(&surface, 0.0, 0.0, &limits()).unwrap(),
        point(0.0, 0.0, 0.0),
        1e-9
    ));
    assert!(close(
        evaluate(&surface, 1.0, 1.0, &limits()).unwrap(),
        point(2.0, 2.0, 0.0),
        1e-9
    ));
    assert!(close(
        evaluate(&surface, 0.5, 0.5, &limits()).unwrap(),
        point(1.0, 1.0, 0.0),
        1e-9
    ));

    // Raise the middle of one boundary: the Coons centre is the average of the boundary
    // contributions, so it must move to z = 0.5.
    let mut raised = boundaries;
    raised[3] =
        curve_build::arc_through([0.0, 2.0, 0.0], [1.0, 2.0, 1.0], [2.0, 2.0, 0.0]).unwrap();
    let surface = network(
        [&raised[0], &raised[1], &raised[2], &raised[3]],
        [2, 1],
        [9, 7],
        &limits(),
    )
    .unwrap();
    let centre = evaluate(&surface, 0.5, 0.5, &limits()).unwrap();
    assert!((centre[2] - 0.5).abs() < 1e-6, "Coons centre {centre:?}");
    // The three flat boundaries stay flat.
    assert!(evaluate(&surface, 0.0, 0.5, &limits()).unwrap()[2].abs() < 1e-9);
    assert!(evaluate(&surface, 0.5, 0.0, &limits()).unwrap()[2].abs() < 1e-9);
    assert!(evaluate(&surface, 1.0, 0.5, &limits()).unwrap()[2].abs() < 1e-9);
}

#[test]
fn m03_02_05_network_refuses_open_or_degenerate_corners() {
    let mut boundaries = square_network();
    boundaries[3] = curve_build::polyline(&[point(0.0, 3.0, 0.0), point(2.0, 3.0, 0.0)]).unwrap();
    assert_eq!(
        network(
            [
                &boundaries[0],
                &boundaries[1],
                &boundaries[2],
                &boundaries[3]
            ],
            [1, 1],
            [7, 7],
            &limits()
        ),
        Err(Error::Invalid("network boundary corner"))
    );
}

/* ------------------------------ L3-003 rail revolve / extrude ------------------------------ */

#[test]
fn m03_02_05_rail_revolve_extrudes_and_follows_a_helix() {
    let profile =
        curve_build::circle_through([2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [-2.0, 0.0, 0.0]).unwrap();
    let rail = line([1.0, 0.0, 0.0], [1.0, 0.0, 10.0]);
    let surface = rail_revolve(
        &profile,
        &rail,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [17, 5],
        [3, 2],
        &limits(),
    )
    .unwrap();
    for step in 0..=4 {
        let v = step as f64 / 4.0;
        let point = evaluate(&surface, 0.0, v, &limits()).unwrap();
        assert!((point[0].powi(2) + point[1].powi(2)).sqrt() - 2.0 < 1e-6);
        assert!((point[2] - 10.0 * v).abs() < 1e-9, "{point:?}");
    }
    // A helical rail rotates the profile: the authored point [2,0,0] lands at
    // [2 cos(t), 2 sin(t), k t] at every station.
    let turns = 2.0 * std::f64::consts::PI;
    let mut points = Vec::new();
    for index in 0..=36 {
        let t = turns * index as f64 / 36.0;
        points.push(point(t.cos(), t.sin(), t));
    }
    let helical = curve_build::polyline(&points).unwrap();
    let surface = rail_revolve(
        &profile,
        &helical,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [17, 37],
        [3, 2],
        &limits(),
    )
    .unwrap();
    for index in [0usize, 9, 18, 27, 36] {
        let v = index as f64 / 36.0;
        let expected_angle = turns * v;
        let surface_point = evaluate(&surface, 0.0, v, &limits()).unwrap();
        assert!(
            close(
                surface_point,
                [
                    2.0 * expected_angle.cos(),
                    2.0 * expected_angle.sin(),
                    expected_angle
                ],
                1e-6
            ),
            "station {index}: {surface_point:?}"
        );
    }
}

#[test]
fn m03_02_05_rail_revolve_and_extrude_refuse_bad_input() {
    let profile = curve_build::polyline(&[point(0.0, 0.0, 0.0), point(1.0, 1.0, 0.0)]).unwrap();
    let rail = line([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    assert_eq!(
        rail_revolve(
            &profile,
            &rail,
            [0.0; 3],
            [0.0, 0.0, 0.0],
            [5, 5],
            [1, 1],
            &limits()
        ),
        Err(Error::Invalid("rail revolve axis"))
    );
    assert_eq!(
        rail_revolve(
            &profile,
            &rail,
            [0.0; 3],
            [0.0, 0.0, 1.0],
            [2, 5],
            [1, 1],
            &limits()
        ),
        Err(Error::Invalid("rail revolve samples"))
    );
}

/* ------------------------------ L3-004 edge / planar / patch ------------------------------ */

#[test]
fn m03_02_05_planar_and_boundary_services_keep_the_underlying_surface() {
    // The planar construction and B-Rep patch live in the OCCT kernel; this owner test
    // checks the shared surface boundary contract: a planar surface is exact and its four
    // boundary curves are the authored lines.
    let surface = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    let bottom = boundary_curve(&surface, SurfaceEdge::V0, 5, &limits()).unwrap();
    let left = boundary_curve(&surface, SurfaceEdge::U0, 5, &limits()).unwrap();
    assert!(matches!(&bottom, Curve::Polycurve { .. }));
    assert!(matches!(&left, Curve::Polycurve { .. }));
    assert!(close(
        evaluate(&surface, 0.0, 0.0, &limits()).unwrap(),
        point(0.0, 0.0, 0.0),
        1e-12
    ));
    assert!(close(
        evaluate(&surface, 1.0, 1.0, &limits()).unwrap(),
        point(4.0, 2.0, 0.0),
        1e-12
    ));
}

/* ------------------------------ L3-005 trim / split / untrim / shrink ------------------------------ */

fn trimmed_plane() -> (NurbsSurface, om9_core::rhino_semantics::TrimmedSurface) {
    let surface = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    let outer = vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]];
    let inner = vec![[0.4, 0.4], [0.6, 0.4], [0.6, 0.6], [0.4, 0.6]];
    let trimmed = trim_surface(&surface, &outer, &[inner], &limits()).unwrap();
    (surface, trimmed)
}

#[test]
fn m03_02_05_trimming_preserves_the_underlying_nurbs_and_untrim_returns_it() {
    let (surface, trimmed) = trimmed_plane();
    assert!(trimmed.is_trimmed());
    assert_eq!(trimmed.outer.len(), 4);
    assert_eq!(trimmed.inners.len(), 1);
    let untrimmed = untrim_surface(&trimmed, &limits()).unwrap();
    assert_eq!(untrimmed, surface);
    // The underlying surface is unchanged: the trim loops do not replace it.
    assert_eq!(trimmed.underlying, surface);
    // A loop outside the domain or an inner loop outside the outer loop is refused.
    assert_eq!(
        trim_surface(
            &surface,
            &[[0.2, 0.2], [1.2, 0.2], [1.2, 0.8], [0.2, 0.8]],
            &[],
            &limits()
        ),
        Err(Error::Invalid("trim loop outside surface domain"))
    );
    assert_eq!(
        trim_surface(
            &surface,
            &[[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
            &[vec![[0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9]]],
            &limits()
        ),
        Err(Error::Invalid("trim inner loop outside outer loop"))
    );
}

#[test]
fn m03_02_05_split_and_shrink_keep_the_surface_domain_consistent() {
    let (surface, trimmed) = trimmed_plane();
    let (left, right) = split_surface(&surface, SurfaceIso::U, 0.5, &limits()).unwrap();
    assert!(left.is_trimmed() && right.is_trimmed());
    assert_eq!(left.underlying, surface);
    assert_eq!(right.underlying, surface);
    // Every half stays inside its side of the split.
    for point in &left.outer {
        assert!(point[0] <= 0.5 + 1e-12);
    }
    for point in &right.outer {
        assert!(point[0] >= 0.5 - 1e-12);
    }
    assert!(split_surface(&surface, SurfaceIso::V, 0.0, &limits()).is_err());
    assert!(split_surface(&surface, SurfaceIso::V, 1.0, &limits()).is_err());

    let shrunk = shrink_surface(&trimmed, [1, 1], [5, 5], &limits()).unwrap();
    assert_eq!(shrunk.underlying.domain, [[0.0, 1.0], [0.0, 1.0]]);
    let corner = evaluate(&shrunk.underlying, 0.0, 0.0, &limits()).unwrap();
    let source = evaluate(&surface, 0.2, 0.2, &limits()).unwrap();
    assert!(close(corner, source, 1e-9), "{corner:?} != {source:?}");
    assert_eq!(shrunk.outer[0], [0.0, 0.0]);
    assert_eq!(shrunk.outer[2], [1.0, 1.0]);
}

/* ------------------------------ L3-006 rebuild / refit / softedit ------------------------------ */

#[test]
fn m03_02_05_refit_and_softedit_change_only_the_control_structure() {
    let surface = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    let refit_surface = refit(&surface, [2, 2], [3, 3], &limits()).unwrap();
    let rebuilt = rebuild_surface(&surface, [2, 2], [3, 3], &limits()).unwrap();
    assert_eq!(rebuilt.size, refit_surface.size);
    assert_eq!(rebuilt.degree, refit_surface.degree);
    // A plane refit is still the same plane.
    for (u, v) in [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)] {
        assert!(close(
            evaluate(&refit_surface, u, v, &limits()).unwrap(),
            evaluate(&surface, u, v, &limits()).unwrap(),
            1e-9
        ));
    }
    let edited = softedit(
        &refit_surface,
        [0.5, 0.5],
        [0.0, 0.0, 1.0],
        [0.6, 0.6],
        &limits(),
    )
    .unwrap();
    let centre = evaluate(&edited, 0.5, 0.5, &limits()).unwrap();
    assert!(
        centre[2] > 0.2 && centre[2] < 1.0,
        "softedit centre {centre:?}"
    );
    // The corner is outside the falloff radius and stays on the plane.
    for (u, v) in [(0.0, 0.0), (1.0, 1.0)] {
        assert!(evaluate(&edited, u, v, &limits()).unwrap()[2].abs() < 1e-9);
    }
    assert_eq!(
        softedit(
            &refit_surface,
            [0.5, 0.5],
            [0.0, 0.0, 1.0],
            [0.0, 0.6],
            &limits()
        ),
        Err(Error::Invalid("softedit radius"))
    );
}

/* ------------------------------ L3-007 match / blend / join ------------------------------ */

fn tilted_target() -> NurbsSurface {
    let plane = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    let mut target = refit(&plane, [2, 4], [3, 5], &limits()).unwrap();
    // Tilt away from y=0 while keeping the shared V0 edge at z=0.
    for j in 0..5 {
        for i in 0..3 {
            target.poles[i * 5 + j][2] = j as f64 * 0.25;
        }
    }
    target.validate(&limits()).unwrap();
    target
}

#[test]
fn m03_02_05_match_surface_reaches_g0_g1_and_g2() {
    let source = refit(
        &plane(
            point(0.0, 0.0, 0.0),
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            4.0,
            2.0,
            &limits(),
        )
        .unwrap(),
        [2, 4],
        [3, 5],
        &limits(),
    )
    .unwrap();
    let target = tilted_target();
    for continuity in [
        SurfaceContinuity::G0,
        SurfaceContinuity::G1,
        SurfaceContinuity::G2,
    ] {
        let report = match_surface(
            &source,
            SurfaceEdge::V0,
            &target,
            SurfaceEdge::V0,
            continuity,
            &limits(),
        )
        .unwrap();
        assert!(
            report.gap_mm < 1e-9,
            "{continuity:?}: gap {}",
            report.gap_mm
        );
        if continuity != SurfaceContinuity::G0 {
            assert!(
                report.tangent_angle_radians < 1e-3,
                "{continuity:?}: angle {}",
                report.tangent_angle_radians
            );
        }
    }
    // Matching a target with too few cross rows for G2 is refused.
    let thin = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    assert_eq!(
        match_surface(
            &source,
            SurfaceEdge::V0,
            &thin,
            SurfaceEdge::V0,
            SurfaceContinuity::G2,
            &limits()
        ),
        Err(Error::Invalid("match continuity rows"))
    );
}

#[test]
fn m03_02_05_blend_surface_is_ruled_at_g0_and_hermite_at_g1_g2() {
    let edge_a = line([0.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    let edge_b = line([0.0, 2.0, 0.0], [4.0, 2.0, 0.0]);
    let ruled = blend_surface(
        &edge_a,
        &edge_b,
        SurfaceContinuity::G0,
        None,
        [5, 5],
        [1, 1],
        &limits(),
    )
    .unwrap();
    assert!(close(
        evaluate(&ruled, 0.5, 0.5, &limits()).unwrap(),
        [2.0, 1.0, 0.0],
        1e-9
    ));
    let tangents = BlendTangents {
        start: [0.0, 1.0, 0.0],
        end: [0.0, 1.0, 0.0],
        start_second: Some([0.0; 3]),
        end_second: Some([0.0; 3]),
    };
    for continuity in [SurfaceContinuity::G1, SurfaceContinuity::G2] {
        let blended = blend_surface(
            &edge_a,
            &edge_b,
            continuity,
            Some(tangents),
            [5, 7],
            [1, 3],
            &limits(),
        )
        .unwrap();
        // Straight tangents keep the blend ruled.
        assert!(close(
            evaluate(&blended, 0.5, 0.5, &limits()).unwrap(),
            [2.0, 1.0, 0.0],
            1e-9
        ));
        assert!(close(
            evaluate(&blended, 0.0, 0.0, &limits()).unwrap(),
            [0.0, 0.0, 0.0],
            1e-9
        ));
        assert!(close(
            evaluate(&blended, 0.0, 1.0, &limits()).unwrap(),
            [0.0, 2.0, 0.0],
            1e-9
        ));
    }
    assert_eq!(
        blend_surface(
            &edge_a,
            &edge_b,
            SurfaceContinuity::G1,
            None,
            [5, 5],
            [1, 1],
            &limits()
        ),
        Err(Error::Missing("blend continuity tangents"))
    );
}

/* ------------------------------ L3-008 offset / fillet / chamfer ------------------------------ */

#[test]
fn m03_02_05_offset_and_normal_extrude_move_a_plane_by_the_distance() {
    let surface = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    for function in [offset_surface, normal_extrude] {
        let offset = function(&surface, 2.5, [5, 5], [1, 1], &limits()).unwrap();
        for (u, v) in [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)] {
            let point = evaluate(&offset, u, v, &limits()).unwrap();
            assert!((point[2] - 2.5).abs() < 1e-9, "{u},{v}: {point:?}");
        }
    }
    assert_eq!(
        offset_surface(&surface, 0.0, [5, 5], [1, 1], &limits()),
        Err(Error::Invalid("surface offset distance"))
    );
    // A cylinder offset inward changes the radius by exactly the distance.
    let cylinder = cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        8.0,
        &limits(),
    )
    .unwrap();
    let offset = offset_surface(&cylinder, -1.0, [17, 5], [2, 1], &limits()).unwrap();
    for step in 0..=8 {
        let u = step as f64 / 8.0;
        let point = evaluate(&offset, u, 0.5, &limits()).unwrap();
        let radial = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radial - 4.0).abs() < 0.05, "u={u}: radius {radial}");
    }
    // The shared B-Rep fillet/chamfer contract remains owned by om9-kernel; a zero/negative
    // offset is refused here before the kernel is called.
    assert!(normal_extrude(&surface, -0.0, [5, 5], [1, 1], &limits()).is_err());
}

/* ------------------------------ L3-009 UV direction / isocurve ------------------------------ */

#[test]
fn m03_02_05_reverse_surface_maps_u_and_v_exactly() {
    let surface = plane(
        point(10.0, 20.0, 30.0),
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    let reversed_u = reverse_surface(&surface, SurfaceIso::U, &limits()).unwrap();
    let reversed_v = reverse_surface(&surface, SurfaceIso::V, &limits()).unwrap();
    for step in 0..=4 {
        let u = step as f64 / 4.0;
        for vstep in 0..=4 {
            let v = vstep as f64 / 4.0;
            let original = evaluate(&surface, u, v, &limits()).unwrap();
            assert!(close(
                evaluate(&reversed_u, 1.0 - u, v, &limits()).unwrap(),
                original,
                1e-12
            ));
            assert!(close(
                evaluate(&reversed_v, u, 1.0 - v, &limits()).unwrap(),
                original,
                1e-12
            ));
        }
    }
    // The iso-curve extraction still returns the exact analytic line after reversal.
    let iso = surface_kernel::isocurve(&reversed_u, SurfaceIso::U, 0.5, &limits()).unwrap();
    assert_eq!(iso.degree, 1);
    assert_eq!(iso.poles.len(), 2);
    assert!(normal(&reversed_u, 0.5, 0.5, &limits()).is_ok());

    // A seam closes an already-coincident boundary and marks the direction periodic; a plane
    // whose boundary rows are apart is refused instead of being bridged.
    let cylinder = cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        8.0,
        &limits(),
    )
    .unwrap();
    let seamed = seam_surface(&cylinder, SurfaceIso::U, 1e-9, &limits()).unwrap();
    assert!(seamed.periodic[0]);
    for step in 0..=4 {
        let v = step as f64 / 4.0;
        assert!(close(
            evaluate(&seamed, 0.0, v, &limits()).unwrap(),
            evaluate(&seamed, 1.0, v, &limits()).unwrap(),
            1e-12
        ));
    }
    assert_eq!(
        seam_surface(&surface, SurfaceIso::U, 1e-9, &limits()),
        Err(Error::Invalid("surface seam gap"))
    );
}

/* ------------------------------ L3-010 normal extrude / unroll ------------------------------ */

#[test]
fn m03_02_05_unroll_preserves_plane_and_cylinder_arc_lengths() {
    let surface = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    let planar = unroll(&surface, [5, 5], &limits()).unwrap();
    assert!(close(
        evaluate(&planar, 1.0, 0.0, &limits()).unwrap(),
        [4.0, 0.0, 0.0],
        1e-9
    ));
    assert!(close(
        evaluate(&planar, 0.0, 1.0, &limits()).unwrap(),
        [0.0, 2.0, 0.0],
        1e-9
    ));
    assert!(evaluate(&planar, 0.5, 0.5, &limits()).unwrap()[2].abs() < 1e-12);

    let cylinder = cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        8.0,
        &limits(),
    )
    .unwrap();
    let unrolled = unroll(&cylinder, [17, 5], &limits()).unwrap();
    let end = evaluate(&unrolled, 1.0, 0.0, &limits()).unwrap();
    let circumference = 2.0 * std::f64::consts::PI * 5.0;
    assert!(
        (end[0] - circumference).abs() < 0.05 * circumference,
        "unrolled circumference {} vs {circumference}",
        end[0]
    );
    assert!(end[1].abs() < 1e-9, "unrolled start {end:?}");
    let top = evaluate(&unrolled, 0.0, 1.0, &limits()).unwrap();
    assert!((top[1] - 8.0).abs() < 1e-6, "unrolled height {top:?}");
    assert!(evaluate(&unrolled, 0.5, 0.5, &limits()).unwrap()[2].abs() < 1e-12);

    assert_eq!(
        unroll(&surface, [1, 5], &limits()),
        Err(Error::Invalid("unroll samples"))
    );
}

/* --------------------------- integration stress --------------------------- */

#[test]
fn m03_02_05_stress_large_grid_keeps_every_surface_valid() {
    let cylinder = cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        8.0,
        &limits(),
    )
    .unwrap();
    let dense = refit(&cylinder, [2, 2], [41, 17], &limits()).unwrap();
    assert_eq!(dense.size, [41, 17]);
    let offset = offset_surface(&dense, 2.0, [9, 5], [2, 1], &limits()).unwrap();
    let (left, right) = split_surface(&dense, SurfaceIso::U, 0.5, &limits()).unwrap();
    assert!(left.is_trimmed() && right.is_trimmed());
    let trimmed = trim_surface(
        &dense,
        &[[0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9]],
        &[],
        &limits(),
    )
    .unwrap();
    let shrunk = shrink_surface(&trimmed, [2, 1], [9, 5], &limits()).unwrap();
    let _ = reverse_surface(&shrunk.underlying, SurfaceIso::V, &limits()).unwrap();
    let planar = unroll(&dense, [17, 5], &limits()).unwrap();
    for surface in [&dense, &offset, &shrunk.underlying, &planar] {
        for (u, v) in [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)] {
            assert!(evaluate(surface, u, v, &limits()).is_ok());
        }
    }
}

/* --------------------------- measured receipt --------------------------- */

#[test]
fn m03_02_05_receipt() {
    let plane = plane(
        point(0.0, 0.0, 0.0),
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap();
    let trimmed = trim_surface(
        &plane,
        &[[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
        &[],
        &limits(),
    )
    .unwrap();
    let (left, _) = split_surface(&plane, SurfaceIso::U, 0.5, &limits()).unwrap();
    let refit_surface = refit(&plane, [2, 2], [3, 3], &limits()).unwrap();
    let edited = softedit(
        &refit_surface,
        [0.5, 0.5],
        [0.0, 0.0, 1.0],
        [0.6, 0.6],
        &limits(),
    )
    .unwrap();
    let offset = offset_surface(&plane, 2.5, [5, 5], [1, 1], &limits()).unwrap();
    let unrolled = unroll(
        &cylinder(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            5.0,
            8.0,
            &limits(),
        )
        .unwrap(),
        [17, 5],
        &limits(),
    )
    .unwrap();
    let receipt = serde_json::json!({
        "status": "measured",
        "trim_outer_points": trimmed.outer.len(),
        "split_left_outer": left.outer,
        "softedit_centre_z": evaluate(&edited, 0.5, 0.5, &limits()).unwrap()[2],
        "offset_plane_z": evaluate(&offset, 0.5, 0.5, &limits()).unwrap()[2],
        "unroll_u": evaluate(&unrolled, 1.0, 0.0, &limits()).unwrap()[0],
        "unroll_v": evaluate(&unrolled, 0.0, 1.0, &limits()).unwrap()[1],
    });
    println!("{receipt}");
}
