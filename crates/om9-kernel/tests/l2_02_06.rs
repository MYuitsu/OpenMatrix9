// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused transform/orient math for `OM9-L2-02-06`: mirror, linear/polar patterns and the
//! two/three-point orientation solver. The interactive command sessions are covered by
//! `crates/om9-core/tests/transform_sessions.rs`; the UI drag/preview/Undo path is covered by
//! the existing `om9-app` gumball suites.
use om9_core::geometry::Point3;
use om9_kernel::transform_ops::{
    apply, linear_pattern, mirror, orient_three_point, orient_two_point, polar_pattern,
};

fn point(x: f64, y: f64, z: f64) -> Point3 {
    [x, y, z]
}

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

#[test]
fn m03_02_06_mirror_plane_reflects_and_array_patterns_generate_placements() {
    let plane = mirror([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]).expect("mirror plane");
    assert!(close(
        apply(&plane, point(2.0, 3.0, 4.0)).unwrap(),
        point(-2.0, 3.0, 4.0),
        1e-12
    ));
    // A body mirror keeps a positive volume through the adapter; the matrix itself has a
    // negative determinant, which the pattern/body layer handles explicitly.
    assert!(om9_kernel::transform_ops::handedness(&plane).unwrap() < 0.0);
    assert!(mirror([0.0; 3], [0.0; 3]).is_err());

    let linear = linear_pattern(4, [2.0, 0.0, 0.0], true).expect("linear pattern");
    assert_eq!(linear.len(), 4);
    for (index, placement) in linear.iter().enumerate() {
        assert!(close(
            apply(placement, point(1.0, 0.0, 0.0)).unwrap(),
            point(1.0 + 2.0 * index as f64, 0.0, 0.0),
            1e-12
        ));
    }
    assert!(linear_pattern(0, [1.0, 0.0, 0.0], true).is_err());
    assert!(linear_pattern(2, [0.0; 3], true).is_err());

    let polar = polar_pattern(4, [0.0; 3], [0.0, 0.0, 1.0], std::f64::consts::TAU, true)
        .expect("polar pattern");
    assert_eq!(polar.len(), 4);
    let mapped: Vec<Point3> = polar
        .iter()
        .map(|placement| apply(placement, point(5.0, 0.0, 0.0)).unwrap())
        .collect();
    for (index, point) in mapped.iter().enumerate() {
        let angle = std::f64::consts::FRAC_PI_2 * index as f64;
        assert!(close(
            *point,
            [5.0 * angle.cos(), 5.0 * angle.sin(), 0.0],
            1e-12
        ));
    }
    // A full turn never duplicates the seam instance.
    assert!(!close(mapped[0], mapped[3], 1e-9));
    assert!(polar_pattern(1, [0.0; 3], [0.0, 0.0, 1.0], 1.0, true).is_err());
}

#[test]
fn m03_02_06_orientation_solver_maps_two_and_three_reference_points() {
    let two = orient_two_point(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(10.0, 20.0, 30.0),
        point(10.0, 21.0, 30.0),
    )
    .expect("two-point orientation");
    assert!(close(
        apply(&two, point(0.0, 0.0, 0.0)).unwrap(),
        point(10.0, 20.0, 30.0),
        1e-12
    ));
    assert!(close(
        apply(&two, point(1.0, 0.0, 0.0)).unwrap(),
        point(10.0, 21.0, 30.0),
        1e-12
    ));
    assert!(
        orient_two_point(
            point(0.0, 0.0, 0.0),
            point(0.0, 0.0, 0.0),
            point(1.0, 0.0, 0.0),
            point(2.0, 0.0, 0.0)
        )
        .is_err()
    );

    let source = [
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(0.0, 1.0, 0.0),
    ];
    let target = [
        point(2.0, 3.0, 4.0),
        point(2.0, 4.0, 4.0),
        point(1.0, 3.0, 4.0),
    ];
    let three = orient_three_point(source, target).expect("three-point orientation");
    for index in 0..3 {
        let mapped = apply(&three, source[index]).unwrap();
        assert!(close(mapped, target[index], 1e-12), "point {index}");
    }
    assert!(
        orient_three_point(
            [
                point(0.0, 0.0, 0.0),
                point(1.0, 0.0, 0.0),
                point(2.0, 0.0, 0.0)
            ],
            [
                point(0.0, 0.0, 0.0),
                point(0.0, 1.0, 0.0),
                point(0.0, 2.0, 0.0)
            ]
        )
        .is_err()
    );
}

#[test]
fn m03_02_06_receipt() {
    let linear = linear_pattern(3, [1.0, 0.0, 0.0], true).unwrap();
    let polar = polar_pattern(4, [0.0; 3], [0.0, 0.0, 1.0], std::f64::consts::TAU, true).unwrap();
    let orient = orient_two_point(
        point(0.0, 0.0, 0.0),
        point(1.0, 0.0, 0.0),
        point(1.0, 2.0, 3.0),
        point(1.0, 3.0, 3.0),
    )
    .unwrap();
    let receipt = serde_json::json!({
        "status": "measured",
        "linear_placements": linear.len(),
        "polar_placements": polar.len(),
        "orient_maps_a": apply(&orient, point(0.0, 0.0, 0.0)).unwrap(),
        "orient_maps_b": apply(&orient, point(1.0, 0.0, 0.0)).unwrap(),
    });
    println!("{receipt}");
}
