// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused acceptance tests for M03 `OM9-L2-01-11` transforms and pattern operations.
//!
//! Matrix oracles are analytic; body oracles measure the exact OCCT B-Rep (volume, bounds,
//! topology). Run one owner at a time, for example:
//!
//! `cargo test -p om9-kernel --features 3dm-brep --test l2_01_11 m03_01_11_001 -- --exact`
use om9_core::{
    brep::Brep,
    budget::Limits,
    geometry::{NurbsSurface, Point3, Transform},
    surface_kernel,
};
use om9_kernel::{
    creation::{self, DEFAULT_DEFLECTION_MM, Primitive},
    transform_ops::{self, MAX_PATTERN_INSTANCES, PatternMode, PatternResult, TransformError},
};
use std::f64::consts::{FRAC_PI_2, PI};

fn limits() -> Limits {
    Limits::default()
}

fn deflection() -> f64 {
    DEFAULT_DEFLECTION_MM
}

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

fn point_close(actual: Point3, expected: Point3, tolerance: f64) -> bool {
    (0..3).all(|index| close(actual[index], expected[index], tolerance))
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

fn measured(brep: &Brep) -> creation::Measures {
    creation::measure(brep, &limits()).expect("measure")
}

/// A plane surface whose parameters are millimetres: domain [0,width]×[0,height].
fn plane_mm(width: f64, height: f64) -> NurbsSurface {
    NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![
            [0.0, 0.0, 0.0],
            [0.0, height, 0.0],
            [width, 0.0, 0.0],
            [width, height, 0.0],
        ],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, width, width], vec![0.0, 0.0, height, height]],
        domain: [[0.0, width], [0.0, height]],
        periodic: [false, false],
        reversed: false,
    }
}

/* ------------------------- 001 matrices ------------------------- */

#[test]
fn m03_01_11_001() {
    let limits = limits();
    // translate (1,2,3) by (4,-1,2) -> (5,1,5)
    let translate = transform_ops::translation([4.0, -1.0, 2.0]).expect("translation");
    let moved = transform_ops::apply(&translate, [1.0, 2.0, 3.0]).expect("apply");
    assert!(point_close(moved, [5.0, 1.0, 5.0], 1e-12));

    // rotate 90° about Z: (1,0,0) -> (0,1,0)
    let rotate =
        transform_ops::rotation([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], FRAC_PI_2).expect("rotation");
    let turned = transform_ops::apply(&rotate, [1.0, 0.0, 0.0]).expect("apply");
    assert!(point_close(turned, [0.0, 1.0, 0.0], 1e-12));

    // inverse restores the composed point
    let composed = transform_ops::compose(&translate, &rotate).expect("compose");
    let source = [3.0, -2.0, 7.0];
    let mapped = transform_ops::apply(&composed, source).expect("mapped");
    let back = transform_ops::apply(&transform_ops::inverse(&composed).expect("inverse"), mapped)
        .expect("restored");
    assert!(point_close(back, source, 1e-9));

    // body transform: 2×3×4 box translated by (5,0,0)
    let body = box_shape(2.0, 3.0, 4.0);
    let body_translation = transform_ops::translation([5.0, 0.0, 0.0]).expect("body translation");
    let moved_body = transform_ops::transform_brep(&body, &body_translation, deflection(), &limits)
        .expect("body");
    let body_measures = measured(&moved_body);
    assert!(close(body_measures.volume_mm3, 24.0, 1e-6));
    assert!(close(body_measures.bounds_mm[0], 5.0, 1e-6));
    assert!(close(body_measures.bounds_mm[3], 7.0, 1e-6));
    assert!(body_measures.closed);

    // Negative: a singular matrix has no inverse; NaN matrices are refused.
    let mut singular = Transform::identity();
    singular.0[0][0] = 0.0;
    assert!(matches!(
        transform_ops::inverse(&singular),
        Err(TransformError::Invalid(_))
    ));
    let nan = Transform([[f64::NAN; 4]; 4]);
    assert!(matches!(
        transform_ops::apply(&nan, [0.0, 0.0, 0.0]),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::transform_brep(&body, &singular, deflection(), &limits),
        Err(TransformError::Invalid(_))
    ));
}

/* ------------------------- 002 mirror ------------------------- */

#[test]
fn m03_01_11_002() {
    let limits = limits();
    let mirror = transform_ops::mirror([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]).expect("mirror");
    let mirrored_point = transform_ops::apply(&mirror, [2.0, 3.0, 4.0]).expect("apply");
    assert!(point_close(mirrored_point, [-2.0, 3.0, 4.0], 1e-12));
    assert!(close(
        transform_ops::handedness(&mirror).expect("handed"),
        -1.0,
        1e-12
    ));

    // A mirrored solid keeps a positive volume: the adapter reverses the inside-out result.
    let body = box_shape(2.0, 3.0, 4.0);
    let original = measured(&body);
    let mirrored = transform_ops::transform_brep(&body, &mirror, deflection(), &limits)
        .expect("mirrored body");
    let mirrored_measures = measured(&mirrored);
    assert!(
        close(mirrored_measures.volume_mm3, 24.0, 1e-6),
        "mirrored volume {}",
        mirrored_measures.volume_mm3
    );
    assert_eq!(mirrored_measures.faces, 6);
    assert!(mirrored_measures.closed);
    assert!(close(mirrored_measures.bounds_mm[0], -2.0, 1e-6));
    assert!(close(mirrored_measures.bounds_mm[3], 0.0, 1e-6));
    // The source body is an immutable value and is unchanged by the mirror.
    assert!(close(measured(&body).volume_mm3, original.volume_mm3, 1e-9));
    assert_eq!(body.source(), body.source());

    // Negative: zero/missing normal is refused.
    assert!(matches!(
        transform_ops::mirror([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::mirror([0.0, 0.0, 0.0], [f64::NAN, 0.0, 1.0]),
        Err(TransformError::Invalid(_))
    ));
}

/* ------------------------- 003 non-uniform scaling ------------------------- */

#[test]
fn m03_01_11_003() {
    let limits = limits();
    let body = box_shape(2.0, 3.0, 4.0);
    let before = measured(&body);
    assert!(close(before.volume_mm3, 24.0, 1e-9));

    let scale = transform_ops::scaling([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]).expect("scale");
    assert!(close(
        transform_ops::handedness(&scale).expect("det"),
        24.0,
        1e-9
    ));
    let scaled =
        transform_ops::transform_brep(&body, &scale, deflection(), &limits).expect("scaled body");
    let scaled_measures = measured(&scaled);
    assert!(
        close(scaled_measures.volume_mm3, 24.0 * 24.0, 1e-5),
        "scaled volume {}",
        scaled_measures.volume_mm3
    );
    let size = scaled_measures.size_mm();
    assert!(close(size[0], 4.0, 1e-6), "size x {}", size[0]);
    assert!(close(size[1], 9.0, 1e-6), "size y {}", size[1]);
    assert!(close(size[2], 16.0, 1e-6), "size z {}", size[2]);

    // A negative determinant mirrors the solid; the volume stays positive and the body closed.
    let reflected = transform_ops::scaling([0.0, 0.0, 0.0], [-1.0, 1.0, 1.0]).expect("reflect");
    assert!(close(
        transform_ops::handedness(&reflected).expect("det"),
        -1.0,
        1e-12
    ));
    let reflected_body =
        transform_ops::transform_brep(&body, &reflected, deflection(), &limits).expect("reflected");
    let reflected_measures = measured(&reflected_body);
    assert!(close(reflected_measures.volume_mm3, 24.0, 1e-6));
    assert!(reflected_measures.closed);
    assert!(close(reflected_measures.bounds_mm[0], -2.0, 1e-6));

    // Negative: zero/NaN factors are refused before the kernel.
    assert!(matches!(
        transform_ops::scaling([0.0, 0.0, 0.0], [0.0, 1.0, 1.0]),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::scaling([0.0, 0.0, 0.0], [f64::NAN, 1.0, 1.0]),
        Err(TransformError::Invalid(_))
    ));
}

/* ------------------------- 004 polar and linear pattern ------------------------- */

#[test]
fn m03_01_11_004() {
    let limits = limits();
    let placements =
        transform_ops::linear_pattern(4, [10.0, 0.0, 0.0], true).expect("linear pattern");
    assert_eq!(placements.len(), 4);
    for (index, placement) in placements.iter().enumerate() {
        let origin = transform_ops::apply(placement, [0.0, 0.0, 0.0]).expect("placement origin");
        assert!(close(origin[0], index as f64 * 10.0, 1e-9));
        assert!(close(origin[1], 0.0, 1e-9));
        assert!(close(origin[2], 0.0, 1e-9));
    }

    let body = box_shape(10.0, 10.0, 10.0);
    let instanced = transform_ops::apply_pattern(
        &body,
        &placements,
        PatternMode::Instance,
        deflection(),
        &limits,
    )
    .expect("instance pattern");
    match &instanced {
        PatternResult::Instanced(pattern) => {
            assert_eq!(pattern.placements.len(), 4);
            // One shared definition: the geometry text is not duplicated four times.
            assert_eq!(pattern.definition.source(), body.source());
        }
        PatternResult::Baked(_) => panic!("instance mode baked the geometry"),
    }
    assert_eq!(instanced.placements(), 4);

    let baked =
        transform_ops::apply_pattern(&body, &placements, PatternMode::Bake, deflection(), &limits)
            .expect("bake pattern");
    match baked {
        PatternResult::Baked(bodies) => {
            assert_eq!(bodies.len(), 4);
            for (index, body) in bodies.iter().enumerate() {
                let measures = measured(body);
                assert!(close(measures.volume_mm3, 1000.0, 1e-6));
                assert!(close(measures.bounds_mm[0], index as f64 * 10.0, 1e-6));
            }
        }
        PatternResult::Instanced(_) => panic!("bake mode kept an instance"),
    }

    // Polar pattern: four instances around Z, no duplicate at the seam for a full turn.
    let polar = transform_ops::polar_pattern(4, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0 * PI, true)
        .expect("polar pattern");
    assert_eq!(polar.len(), 4);
    let angles: Vec<f64> = polar
        .iter()
        .map(|placement| {
            let point = transform_ops::apply(placement, [1.0, 0.0, 0.0]).expect("polar point");
            point[1].atan2(point[0]).rem_euclid(2.0 * PI)
        })
        .collect();
    for index in 1..angles.len() {
        assert!((angles[index] - angles[index - 1] - FRAC_PI_2).abs() < 1e-9);
    }

    // Negative: zero count, negative spacing, overflow, empty placements.
    assert!(matches!(
        transform_ops::linear_pattern(0, [1.0, 0.0, 0.0], true),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::linear_pattern(4, [-1.0, 0.0, 0.0], true),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::linear_pattern(MAX_PATTERN_INSTANCES + 1, [1.0, 0.0, 0.0], true),
        Err(TransformError::Budget(_))
    ));
    assert!(matches!(
        transform_ops::apply_pattern(&body, &[], PatternMode::Instance, deflection(), &limits),
        Err(TransformError::Invalid(_))
    ));
}

/* ------------------------- 005 curve sampled frames ------------------------- */

#[test]
fn m03_01_11_005() {
    let rail = [[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]];
    let frames = transform_ops::frames_along_polyline(&rail, 3).expect("frames");
    assert_eq!(frames.len(), 3);
    let expected_origins = [0.0, 5.0, 10.0];
    for (frame, expected) in frames.iter().zip(expected_origins) {
        frame.validate().expect("orthonormal frame");
        assert!(close(frame.origin[2], expected, 1e-9));
        assert!(close(frame.z[2], 1.0, 1e-9), "tangent must be +Z");
        // Right-handed: x × y = z.
        let cross = [
            frame.x[1] * frame.y[2] - frame.x[2] * frame.y[1],
            frame.x[2] * frame.y[0] - frame.x[0] * frame.y[2],
            frame.x[0] * frame.y[1] - frame.x[1] * frame.y[0],
        ];
        assert!(point_close(cross, frame.z, 1e-9));
    }

    // The frame matrix places a local point at origin + x*1 + y*2.
    let matrix = frames[1].matrix();
    let local = transform_ops::apply(&matrix, [1.0, 2.0, 0.0]).expect("local point");
    let expected = [
        frames[1].origin[0] + frames[1].x[0] + frames[1].y[0] * 2.0,
        frames[1].origin[1] + frames[1].x[1] + frames[1].y[1] * 2.0,
        frames[1].origin[2] + frames[1].x[2] + frames[1].y[2] * 2.0,
    ];
    assert!(point_close(local, expected, 1e-9));

    // Negative: one sample, a repeated rail point (zero tangent), non-finite input.
    assert!(matches!(
        transform_ops::frames_along_polyline(&rail, 1),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::frames_along_polyline(&[[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]], 3),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::frames_along_polyline(&[[0.0, 0.0, 0.0], [0.0, f64::NAN, 1.0]], 3),
        Err(TransformError::Invalid(_))
    ));
}

/* ------------------------- 006 surface UV frames ------------------------- */

#[test]
fn m03_01_11_006() {
    let limits = limits();
    let plane = plane_mm(10.0, 10.0);
    let frame = transform_ops::frame_on_surface(&plane, 2.0, 3.0, &limits).expect("plane frame");
    frame.validate().expect("plane frame orthonormal");
    assert!(point_close(frame.origin, [2.0, 3.0, 0.0], 1e-9));
    assert!(point_close(frame.z, [0.0, 0.0, 1.0], 1e-9));
    assert!(point_close(frame.x, [1.0, 0.0, 0.0], 1e-9));
    assert!(point_close(frame.y, [0.0, 1.0, 0.0], 1e-9));

    let cylinder = surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        2.0,
        10.0,
        &limits,
    )
    .expect("cylinder");
    let radial =
        transform_ops::frame_on_surface(&cylinder, 0.25, 0.5, &limits).expect("cylinder frame");
    radial.validate().expect("cylinder frame orthonormal");
    // The normal of a cylinder is the unit radial direction.
    let origin = radial.origin;
    let radius = (origin[0] * origin[0] + origin[1] * origin[1]).sqrt();
    assert!(close(radius, 2.0, 1e-9));
    assert!(close(
        radial.z[0] * origin[0] + radial.z[1] * origin[1],
        radius,
        1e-9
    ));
    assert!(close(radial.z[2], 0.0, 1e-9));

    // Negative: a fully degenerate surface has zero derivatives and no frame; out-of-domain UV.
    let degenerate = NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![[0.0; 3]; 4],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    assert!(matches!(
        transform_ops::frame_on_surface(&degenerate, 0.5, 0.5, &limits),
        Err(TransformError::Invalid(_))
    ));
    assert!(transform_ops::frame_on_surface(&plane, 11.0, 0.5, &limits).is_err());
}

/* ------------------------- 007 orientation solver ------------------------- */

#[test]
fn m03_01_11_007() {
    let matrix = transform_ops::orient_two_point(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 1.0, 0.0],
    )
    .expect("two-point orientation");
    let a = transform_ops::apply(&matrix, [0.0, 0.0, 0.0]).expect("a");
    let b = transform_ops::apply(&matrix, [1.0, 0.0, 0.0]).expect("b");
    assert!(point_close(a, [10.0, 0.0, 0.0], 1e-9));
    assert!(point_close(b, [10.0, 1.0, 0.0], 1e-9));
    assert!(close(
        transform_ops::handedness(&matrix).expect("det"),
        1.0,
        1e-9
    ));
    // Rigid: the AB distance is preserved.
    let distance = |p: Point3, q: Point3| {
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
    };
    assert!(close(distance(a, b), 1.0, 1e-9));

    let source = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let target = [[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.0, 6.0, 5.0]];
    let three = transform_ops::orient_three_point(source, target).expect("three-point");
    for index in 0..3 {
        let mapped = transform_ops::apply(&three, source[index]).expect("mapped");
        assert!(point_close(mapped, target[index], 1e-9));
    }
    assert!(close(
        transform_ops::handedness(&three).expect("det"),
        1.0,
        1e-9
    ));

    // Negative: coincident two-point references and collinear three-point references.
    assert!(matches!(
        transform_ops::orient_two_point(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 1.0, 0.0]
        ),
        Err(TransformError::Invalid(_))
    ));
    assert!(matches!(
        transform_ops::orient_three_point(
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]
        ),
        Err(TransformError::Invalid(_))
    ));
}

/* ------------------------- 008 bake vs instance ------------------------- */

#[test]
fn m03_01_11_008() {
    let limits = limits();
    let body = box_shape(10.0, 10.0, 10.0);
    let placements = transform_ops::linear_pattern(2, [10.0, 0.0, 0.0], true).expect("placements");
    let instanced = transform_ops::apply_pattern(
        &body,
        &placements,
        PatternMode::Instance,
        deflection(),
        &limits,
    )
    .expect("instance");
    let PatternResult::Instanced(pattern) = instanced else {
        panic!("instance mode must not bake");
    };
    assert_eq!(pattern.definition.source(), body.source());
    assert_eq!(pattern.placements.len(), 2);

    let baked =
        transform_ops::apply_pattern(&body, &placements, PatternMode::Bake, deflection(), &limits)
            .expect("bake");
    let PatternResult::Baked(bodies) = baked else {
        panic!("bake mode must not keep an instance");
    };
    assert_eq!(bodies.len(), 2);
    assert!(close(measured(&bodies[0]).bounds_mm[0], 0.0, 1e-6));
    assert!(close(measured(&bodies[1]).bounds_mm[0], 10.0, 1e-6));
    // Independent bodies: each has its own canonical source and its own measured extent.
    assert_ne!(bodies[0].source().as_ptr(), bodies[1].source().as_ptr());
    assert!(close(measured(&bodies[0]).volume_mm3, 1000.0, 1e-6));
    assert!(close(measured(&bodies[1]).volume_mm3, 1000.0, 1e-6));

    // Negative: empty placements and an inadmissible (singular) placement reference.
    assert!(matches!(
        transform_ops::apply_pattern(&body, &[], PatternMode::Bake, deflection(), &limits),
        Err(TransformError::Invalid(_))
    ));
    let mut singular = Transform::identity();
    singular.0[0][0] = 0.0;
    assert!(matches!(
        transform_ops::apply_pattern(
            &body,
            &[singular],
            PatternMode::Instance,
            deflection(),
            &limits
        ),
        Err(TransformError::Invalid(_))
    ));
}

/* ------------------------- regression and stress ------------------------- */

#[test]
fn m03_01_11_regression_matrix_round_trip_and_body_measure() {
    let limits = limits();
    let body = box_shape(2.0, 3.0, 4.0);
    let matrices = [
        transform_ops::translation([3.0, -2.0, 5.0]).expect("translate"),
        transform_ops::rotation([1.0, 2.0, 3.0], [1.0, 1.0, 1.0], 0.7).expect("rotate"),
        transform_ops::scaling([1.0, 1.0, 1.0], [2.0, 0.5, 3.0]).expect("scale"),
        transform_ops::mirror([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).expect("mirror"),
    ];
    for matrix in matrices {
        let inverse = transform_ops::inverse(&matrix).expect("inverse");
        let composed = transform_ops::compose(&inverse, &matrix).expect("identity");
        for point in [[0.0, 0.0, 0.0], [1.0, -2.0, 4.0]] {
            let restored = transform_ops::apply(&composed, point).expect("restored");
            assert!(point_close(restored, point, 1e-9));
        }
        let transformed =
            transform_ops::transform_brep(&body, &matrix, deflection(), &limits).expect("body");
        let reopened = om9_kernel::brep::read_brep(transformed.source(), deflection(), &limits)
            .expect("reopen");
        assert!(close(
            measured(&reopened).volume_mm3,
            measured(&transformed).volume_mm3,
            1e-6
        ));
    }
}

#[test]
fn m03_01_11_stress_surface_frames_and_baked_pattern() {
    let limits = limits();
    // 200 UV samples on a cylinder: every frame stays orthonormal and right-handed.
    let cylinder = surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        2.0,
        10.0,
        &limits,
    )
    .expect("cylinder");
    for index in 0..200 {
        let u = 0.001 + 0.998 * index as f64 / 199.0;
        let v = 0.001 + 0.998 * (199 - index) as f64 / 199.0;
        let frame = transform_ops::frame_on_surface(&cylinder, u, v, &limits).expect("frame");
        frame.validate().expect("orthonormal frame");
    }

    // A 40-instance baked linear pattern: every body is valid and keeps its own placement.
    let body = box_shape(1.0, 1.0, 1.0);
    let placements = transform_ops::linear_pattern(40, [2.0, 0.0, 0.0], true).expect("placements");
    let PatternResult::Baked(bodies) =
        transform_ops::apply_pattern(&body, &placements, PatternMode::Bake, deflection(), &limits)
            .expect("bake")
    else {
        panic!("bake mode");
    };
    assert_eq!(bodies.len(), 40);
    for (index, baked) in bodies.iter().enumerate() {
        let measures = measured(baked);
        assert!(close(measures.volume_mm3, 1.0, 1e-9));
        assert!(close(measures.bounds_mm[0], index as f64 * 2.0, 1e-6));
        assert!(measures.closed);
    }
}

/* ------------------------- measured receipt ------------------------- */

/// Prints the measured oracle for every L3 fixture as one JSON object. The same test asserts the
/// values, so a printed line is also a pass.
#[test]
fn m03_01_11_receipt() {
    let limits = limits();
    let mut report = String::from("{\"measured\":{");
    let mut push = |name: &str, value: f64| {
        report.push_str(&format!("\"{name}\":{value:.12},"));
    };

    let body = box_shape(2.0, 3.0, 4.0);
    let translated = transform_ops::transform_brep(
        &body,
        &transform_ops::translation([5.0, 0.0, 0.0]).expect("translation"),
        deflection(),
        &limits,
    )
    .expect("translated");
    let translated_volume = measured(&translated).volume_mm3;
    push("translated_volume_mm3", translated_volume);
    assert!(close(translated_volume, 24.0, 1e-6));

    let mirrored = transform_ops::transform_brep(
        &body,
        &transform_ops::mirror([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]).expect("mirror"),
        deflection(),
        &limits,
    )
    .expect("mirrored");
    let mirrored_volume = measured(&mirrored).volume_mm3;
    push("mirrored_volume_mm3", mirrored_volume);
    push("mirrored_x_min_mm", measured(&mirrored).bounds_mm[0]);
    assert!(close(mirrored_volume, 24.0, 1e-6));
    assert!(close(measured(&mirrored).bounds_mm[0], -2.0, 1e-6));

    let scaled = transform_ops::transform_brep(
        &body,
        &transform_ops::scaling([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]).expect("scale"),
        deflection(),
        &limits,
    )
    .expect("scaled");
    let scaled_volume = measured(&scaled).volume_mm3;
    push("scaled_volume_mm3", scaled_volume);
    push("scaled_size_x_mm", measured(&scaled).size_mm()[0]);
    assert!(close(scaled_volume, 576.0, 1e-5));

    let placements = transform_ops::linear_pattern(4, [10.0, 0.0, 0.0], true).expect("placements");
    push("linear_pattern_placements", placements.len() as f64);
    let PatternResult::Instanced(pattern) = transform_ops::apply_pattern(
        &body,
        &placements,
        PatternMode::Instance,
        deflection(),
        &limits,
    )
    .expect("instance") else {
        panic!("instance");
    };
    push("instance_definitions", 1.0);
    assert_eq!(pattern.placements.len(), 4);
    let PatternResult::Baked(bodies) =
        transform_ops::apply_pattern(&body, &placements, PatternMode::Bake, deflection(), &limits)
            .expect("bake")
    else {
        panic!("bake");
    };
    push("baked_bodies", bodies.len() as f64);
    assert_eq!(bodies.len(), 4);

    let two = transform_ops::linear_pattern(2, [10.0, 0.0, 0.0], true).expect("two placements");
    let PatternResult::Instanced(two_instance) =
        transform_ops::apply_pattern(&body, &two, PatternMode::Instance, deflection(), &limits)
            .expect("two instance")
    else {
        panic!("instance");
    };
    push("bake_instance_definitions", 1.0);
    assert_eq!(two_instance.placements.len(), 2);
    let PatternResult::Baked(two_baked) =
        transform_ops::apply_pattern(&body, &two, PatternMode::Bake, deflection(), &limits)
            .expect("two bake")
    else {
        panic!("bake");
    };
    push("bake_bodies", two_baked.len() as f64);
    assert_eq!(two_baked.len(), 2);
    let frames = transform_ops::frames_along_polyline(&[[0.0, 0.0, 0.0], [0.0, 0.0, 10.0]], 3)
        .expect("frames");
    push("frame_mid_z_mm", frames[1].origin[2]);
    assert!(close(frames[1].origin[2], 5.0, 1e-9));
    let plane = plane_mm(10.0, 10.0);
    let frame = transform_ops::frame_on_surface(&plane, 2.0, 3.0, &limits).expect("surface frame");
    push("surface_frame_x_mm", frame.origin[0]);
    push("surface_frame_y_mm", frame.origin[1]);
    assert!(point_close(frame.origin, [2.0, 3.0, 0.0], 1e-9));

    let orient = transform_ops::orient_two_point(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 1.0, 0.0],
    )
    .expect("orientation");
    let mapped_a = transform_ops::apply(&orient, [0.0, 0.0, 0.0]).expect("a");
    let mapped_b = transform_ops::apply(&orient, [1.0, 0.0, 0.0]).expect("b");
    push(
        "orientation_distance_mm",
        ((mapped_a[0] - mapped_b[0]).powi(2)
            + (mapped_a[1] - mapped_b[1]).powi(2)
            + (mapped_a[2] - mapped_b[2]).powi(2))
        .sqrt(),
    );
    assert!(point_close(mapped_a, [10.0, 0.0, 0.0], 1e-9));

    report.pop();
    report.push_str("}}");
    println!("{report}");
}
