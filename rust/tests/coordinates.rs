use openmatrix9_rust::coordinates::{Frame, PointInput};

fn resolved(frame: Frame, text: &str) -> [f64; 3] {
    PointInput::parse(text, |value| value.parse().map_err(|_| "bad number".into()))
        .unwrap()
        .resolve(frame, None)
        .unwrap()
        .world
}

#[test]
fn near_scaled_axes_reject_instead_of_disagreeing_with_a_rigid_native_placement() {
    // The old 1e-6 Gram-matrix allowance accepted the first basis and mapped
    // x=1e6 to 1000000.4 mm; a native rotation discards its 0.4 mm scale error.
    for scale in [1.0000004, 1. + 1e-10, 1. - 1e-10] {
        let axes = [[scale, 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        assert!(Frame::new([0.; 3], axes).is_err(), "accepted scale {scale}");
    }
}

#[test]
fn near_sheared_axes_reject_even_when_individual_axes_have_unit_length() {
    for shear in [4e-7_f64, 1e-10, -1e-10] {
        let axes = [
            [1., 0., 0.],
            [shear, (1. - shear * shear).sqrt(), 0.],
            [0., 0., 1.],
        ];
        assert!(Frame::new([0.; 3], axes).is_err(), "accepted shear {shear}");
    }
}

#[test]
fn arbitrary_rigid_frame_preserves_supplied_axes_and_large_coordinate_meaning() {
    // Exact rational rotation from unit quaternion (1,2,3,4)/sqrt(30).
    let axes = [
        [2. / 15., 14. / 15., -1. / 3.],
        [-2. / 3., 1. / 3., 2. / 3.],
        [11. / 15., 2. / 15., 2. / 3.],
    ];
    let frame = Frame::new([10., 20., 30.], axes).unwrap();
    assert_eq!(frame.axes(), axes); // Validation must not repair/normalize inputs.
    let point = resolved(frame, "150000,0,0");
    for (actual, expected) in point.into_iter().zip([20010., 140020., -49970.]) {
        assert!((actual - expected).abs() < 1e-9);
    }
}

#[test]
fn normally_rounded_quaternion_rotations_remain_accepted_without_repair() {
    let mut largest_gram_error = 0.0_f64;
    // Independent normalized-quaternion rotation matrices sample general and
    // almost-identity/half-turn orientations, rather than axis-aligned cases.
    for i in 0..4096 {
        let a = f64::from(i) * 0.0137;
        let mut q = [a.sin(), (a * 1.7).cos(), (a * 2.3).sin(), (a * 3.1).cos()];
        let length = q.iter().map(|v| v * v).sum::<f64>().sqrt();
        q.iter_mut().for_each(|v| *v /= length);
        let [x, y, z, w] = q;
        let axes = [
            [
                1. - 2. * (y * y + z * z),
                2. * (x * y + w * z),
                2. * (x * z - w * y),
            ],
            [
                2. * (x * y - w * z),
                1. - 2. * (x * x + z * z),
                2. * (y * z + w * x),
            ],
            [
                2. * (x * z + w * y),
                2. * (y * z - w * x),
                1. - 2. * (x * x + y * y),
            ],
        ];
        let frame = Frame::new([10., 20., 30.], axes)
            .unwrap_or_else(|error| panic!("rotation {i}: {error}"));
        assert_eq!(frame.axes(), axes);
        for row in 0..3 {
            for column in 0..3 {
                let dot: f64 = axes[row].iter().zip(axes[column]).map(|(a, b)| a * b).sum();
                largest_gram_error =
                    largest_gram_error.max((dot - if row == column { 1. } else { 0. }).abs());
            }
        }
    }
    println!("largest_rotation_gram_error={largest_gram_error:e}");
}

#[test]
fn arbitrary_three_point_construction_still_builds_a_valid_rigid_frame() {
    let frame =
        Frame::from_three_points([10., 20., 30.], [12., 19., 32.], [15., 24., 31.]).unwrap();
    assert_eq!(frame.origin(), [10., 20., 30.]);
    for (actual, expected) in resolved(frame, "3,0,0").into_iter().zip([12., 19., 32.]) {
        assert!((actual - expected).abs() < 1e-12);
    }
    // A non-orthogonal Y hint determines the plane; it is not an imported axis.
    let expected_y = [29., 44., -7.].map(|value| value / 2826_f64.sqrt());
    for (actual, expected) in frame.axes()[1].into_iter().zip(expected_y) {
        assert!((actual - expected).abs() < 1e-14);
    }
    assert_eq!(Frame::new(frame.origin(), frame.axes()).unwrap(), frame);
}
