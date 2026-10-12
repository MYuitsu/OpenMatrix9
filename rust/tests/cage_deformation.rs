use openmatrix9_rust::cage::*;

fn box_cage() -> Cage {
    let mut points = Vec::new();
    for k in 0..2 {
        for j in 0..2 {
            for i in 0..2 {
                points.push([i as f64, j as f64, k as f64]);
            }
        }
    }
    Cage {
        counts: [2; 3],
        degrees: [1; 3],
        knots: std::array::from_fn(|_| vec![0., 0., 1., 1.]),
        points,
        weights: vec![1.; 8],
    }
}
fn close(a: [f64; 3], b: [f64; 3]) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() < 1e-10, "{a:?} != {b:?}");
    }
}
#[test]
fn displaced_corner_has_trilinear_influence() {
    let mut c = box_cage();
    c.points[7][2] = 2.;
    close(c.evaluate([0.25, 0.5, 0.75]).unwrap(), [0.25, 0.5, 0.84375]);
}
#[test]
fn rational_weights_change_interpolation() {
    let mut c = box_cage();
    c.weights[7] = 3.;
    close(c.evaluate([0.5; 3]).unwrap(), [0.6; 3]);
}
#[test]
fn unchanged_cage_extrapolates_global_identity() {
    let d = Deformation::new(box_cage(), IDENTITY, Region::Global).unwrap();
    for p in [[0.1, 0.2, 0.3], [-2., 3., 1.5]] {
        close(d.deform(p).unwrap(), p);
    }
}
#[test]
fn reference_frame_supports_rotation_scale_and_reflection() {
    let m = [
        0., 0.5, 0., -1., -1., 0., 0., 3., 0., 0., 0.25, -1., 0., 0., 0., 1.,
    ];
    let d = Deformation::new(box_cage(), m, Region::Global).unwrap();
    close(d.deform([2.5, 2.5, 6.]).unwrap(), [0.25, 0.5, 0.5]);
}
#[test]
fn non_affine_results_always_use_original_input() {
    let mut c = box_cage();
    c.points[7][2] += 1.;
    let d = Deformation::new(c, IDENTITY, Region::Global).unwrap();
    let original = [[0.5; 3], [1.; 3]];
    assert_eq!(
        d.deform_points(&original).unwrap(),
        d.deform_points(&original).unwrap()
    );
    close(d.deform(original[0]).unwrap(), [0.5, 0.5, 0.625]);
    assert!(d.exact_affine_transform().unwrap().is_none());
}
#[test]
fn local_falloff_is_smooth_bounded_and_finite() {
    let mut c = box_cage();
    for p in &mut c.points {
        p[2] += 2.;
    }
    let d = Deformation::new(
        c,
        IDENTITY,
        Region::Local {
            min: [0.; 3],
            max: [1.; 3],
            falloff: 2.,
        },
    )
    .unwrap();
    close(d.deform([0.5; 3]).unwrap(), [0.5, 0.5, 2.5]);
    close(d.deform([2., 0.5, 0.5]).unwrap(), [2., 0.5, 1.5]);
    close(d.deform([4., 0.5, 0.5]).unwrap(), [4., 0.5, 0.5]);
    assert!(d.exact_affine_transform().unwrap().is_none());
}
#[test]
fn exact_affine_uses_all_control_points() {
    let mut c = box_cage();
    for p in &mut c.points {
        *p = [2. * p[0] + 3., -p[1] + 2., p[2] + 4.];
    }
    let d = Deformation::new(c.clone(), IDENTITY, Region::Global).unwrap();
    assert_eq!(
        d.exact_affine_transform().unwrap().unwrap(),
        [
            2., 0., 0., 3., 0., -1., 0., 2., 0., 0., 1., 4., 0., 0., 0., 1.
        ]
    );
    c.weights[7] = 2.;
    assert!(
        Deformation::new(c, IDENTITY, Region::Global)
            .unwrap()
            .exact_affine_transform()
            .unwrap()
            .is_none()
    );
}
#[test]
fn rejects_invalid_model_and_reference_frame() {
    let mut c = box_cage();
    c.weights[0] = 0.;
    assert!(c.validate().is_err());
    c = box_cage();
    c.knots[0][2] = -1.;
    assert!(c.validate().is_err());
    c = box_cage();
    c.points[0][0] = f64::NAN;
    assert!(c.validate().is_err());
    c = box_cage();
    c.counts[0] = usize::MAX;
    assert!(c.validate().is_err());
    assert!(Deformation::new(box_cage(), [0.; 16], Region::Global).is_err());
}
#[test]
fn arbitrary_degree_partition_and_nonuniform_knots() {
    for degree in 1..=16 {
        let n = degree + 1;
        let mut c = box_cage();
        c.counts[0] = n;
        c.degrees[0] = degree;
        c.knots[0] = [vec![2.; n], vec![5.; n]].concat();
        c.points = (0..4)
            .flat_map(|_| (0..n).map(|i| [i as f64 / degree as f64, 7., 9.]))
            .collect();
        c.weights = vec![1.; 4 * n];
        close(c.evaluate([2.75, 0.2, 0.8]).unwrap(), [0.25, 7., 9.]);
    }
    let mut c = box_cage();
    c.counts[0] = 4;
    c.degrees[0] = 2;
    c.knots[0] = vec![0., 0., 0., 0.25, 1., 1., 1.];
    c.points = (0..4)
        .flat_map(|_| [0., 0.125, 0.625, 1.].map(|x| [x, 4., 6.]))
        .collect();
    c.weights = vec![1.; 16];
    close(c.evaluate([0.4, 0.2, 0.8]).unwrap(), [0.4, 4., 6.]);
}
#[test]
fn current_cage_inverse_binds_without_double_deformation() {
    let mut c = box_cage();
    c.points[7][2] = 2.;
    let d = Deformation::new(c, IDENTITY, Region::Global).unwrap();
    let uvw = d.inverse_point([0.5, 0.5, 0.625]).unwrap();
    close(uvw, [0.5; 3]);
    close(d.evaluate_normalized(uvw).unwrap(), [0.5, 0.5, 0.625]);
    assert!(d.inverse_point([3., 0., 0.]).is_err());
}
#[test]
fn affine_inverse_supports_points_outside_and_ignores_reference_frame() {
    let mut c = box_cage();
    for p in &mut c.points {
        *p = [2. * p[0] + 3., -p[1] + 2., p[2] + 4.];
    }
    let mut frame = IDENTITY;
    frame[3] = 100.;
    let d = Deformation::new(c, frame, Region::Global).unwrap();
    close(d.inverse_point([9., 4., 3.]).unwrap(), [3., -2., -1.]);
}
#[test]
fn inverse_rejects_folded_and_singular_current_cage() {
    let mut c = box_cage();
    for p in &mut c.points {
        p[2] = 0.;
    }
    assert!(
        Deformation::new(c, IDENTITY, Region::Global)
            .unwrap()
            .inverse_point([0.5, 0.5, 0.])
            .is_err()
    );
    let mut c = box_cage();
    c.points[7][2] = -2.;
    assert!(
        Deformation::new(c, IDENTITY, Region::Global)
            .unwrap()
            .inverse_point([0.5; 3])
            .is_err()
    );
}
fn descriptor(c: &Cage) -> Om9CageDescriptor {
    Om9CageDescriptor {
        counts: c.counts,
        degrees: c.degrees,
        knots: std::array::from_fn(|a| c.knots[a].as_ptr()),
        knot_lengths: std::array::from_fn(|a| c.knots[a].len()),
        points: c.points.as_ptr().cast(),
        point_count: c.points.len(),
        weights: c.weights.as_ptr(),
        weight_count: c.weights.len(),
        world_to_parameter: IDENTITY,
        region: 0,
        local_min: [0.; 3],
        local_max: [1.; 3],
        falloff: 0.,
    }
}
#[test]
fn ffi_copies_descriptor_and_batches_are_atomic() {
    let mut c = box_cage();
    let desc = descriptor(&c);
    let mut handle = std::ptr::null_mut();
    unsafe {
        assert_eq!(om9_cage_create(&desc, &mut handle), 0);
        c.points[7] = [100.; 3];
        drop(c);
        let input = [0.5, 0.5, 0.5, f64::NAN, 0., 0.];
        let mut output = [42.; 6];
        assert!(om9_cage_deform(handle, input.as_ptr(), 2, output.as_mut_ptr(), 2) < 0);
        assert_eq!(output, [42.; 6]);
        assert!(om9_cage_deform(handle, input.as_ptr(), 1, output.as_mut_ptr(), 0) < 0);
        assert_eq!(output, [42.; 6]);
        assert_eq!(
            om9_cage_deform(handle, input.as_ptr(), 1, output.as_mut_ptr(), 2),
            0
        );
        close([output[0], output[1], output[2]], [0.5; 3]);
        assert_eq!(
            om9_cage_inverse(handle, output.as_ptr(), 1, output.as_mut_ptr(), 2),
            0
        );
        close([output[0], output[1], output[2]], [0.5; 3]);
        om9_cage_destroy(handle);
    }
}
#[test]
fn ffi_checks_nulls_lengths_and_affine_output() {
    let c = box_cage();
    let mut desc = descriptor(&c);
    let mut handle = std::ptr::null_mut();
    unsafe {
        assert!(om9_cage_create(std::ptr::null(), &mut handle) < 0);
        assert!(handle.is_null());
        desc.point_count = usize::MAX;
        assert!(om9_cage_create(&desc, &mut handle) < 0);
        assert!(handle.is_null());
        desc.point_count = 8;
        assert_eq!(om9_cage_create(&desc, &mut handle), 0);
        let mut output = [77.; 16];
        assert_eq!(om9_cage_affine(handle, output.as_mut_ptr()), 0);
        assert_eq!(output, IDENTITY);
        assert!(om9_cage_deform(handle, std::ptr::null(), 1, output.as_mut_ptr(), 1) < 0);
        assert_eq!(
            om9_cage_deform(handle, std::ptr::null(), 0, std::ptr::null_mut(), 0),
            0
        );
        assert_eq!(
            om9_cage_evaluate(handle, [0.2, 0.3, 0.4].as_ptr(), 1, output.as_mut_ptr(), 1),
            0
        );
        close([output[0], output[1], output[2]], [0.2, 0.3, 0.4]);
        om9_cage_destroy(handle);
        om9_cage_destroy(std::ptr::null_mut());
    }
}

#[test]
fn binding_reuses_current_parameters_across_edits() {
    let mut c = box_cage();
    c.points[7][2] = 2.;
    let current = Deformation::new(c.clone(), IDENTITY, Region::Global).unwrap();
    let binding = CageBinding::capture(&current, &[[0.5, 0.5, 0.625]]).unwrap();
    close(binding.parameters()[0], [0.5; 3]);
    c.points[7][2] = 3.;
    let mut frame = IDENTITY;
    frame[3] = 100.;
    let edited = Deformation::new(c, frame, Region::Global).unwrap();
    close(binding.apply(&edited).unwrap()[0], [0.5, 0.5, 0.75]);
    close(binding.apply(&edited).unwrap()[0], [0.5, 0.5, 0.75]);
}
#[test]
fn binding_local_edits_blend_from_captured_world_points() {
    let current = Deformation::new(box_cage(), IDENTITY, Region::Global).unwrap();
    let binding = CageBinding::capture(&current, &[[2., 0.5, 0.5], [4., 0.5, 0.5]]).unwrap();
    let mut c = box_cage();
    for p in &mut c.points {
        p[2] += 2.;
    }
    let edited = Deformation::new(
        c,
        IDENTITY,
        Region::Local {
            min: [0.; 3],
            max: [1.; 3],
            falloff: 2.,
        },
    )
    .unwrap();
    close(binding.apply(&edited).unwrap()[0], [2., 0.5, 1.5]);
    close(binding.apply(&edited).unwrap()[1], [4., 0.5, 0.5]);
}
#[test]
fn global_rational_extrapolation_rejects_nonpositive_denominator() {
    let mut c = box_cage();
    for (index, w) in c.weights.iter_mut().enumerate() {
        *w = if index % 2 == 0 { 1. } else { 2. };
    }
    assert_eq!(c.evaluate([-2., 0.5, 0.5]), Err(Error::Denominator));
}
#[test]
fn ffi_inverse_failure_does_not_write_partial_output() {
    let mut c = box_cage();
    c.points[7][2] = 2.;
    let desc = descriptor(&c);
    let mut handle = std::ptr::null_mut();
    unsafe {
        assert_eq!(om9_cage_create(&desc, &mut handle), 0);
        let input = [0.5, 0.5, 0.625, 3., 0., 0.];
        let mut out = [17.; 6];
        assert!(om9_cage_inverse(handle, input.as_ptr(), 2, out.as_mut_ptr(), 2) < 0);
        assert_eq!(out, [17.; 6]);
        om9_cage_destroy(handle);
    }
}
#[test]
fn affine_proof_checks_interior_cvs_without_fit_tolerance() {
    let mut c = box_cage();
    c.counts[0] = 3;
    c.degrees[0] = 2;
    c.knots[0] = vec![0., 0., 0., 1., 1., 1.];
    c.points = (0..2)
        .flat_map(|k| (0..2).flat_map(move |j| [0., 0.5, 1.].map(|x| [x, j as f64, k as f64])))
        .collect();
    c.weights = vec![1.; 12];
    assert_eq!(
        Deformation::new(c.clone(), IDENTITY, Region::Global)
            .unwrap()
            .exact_affine_transform()
            .unwrap(),
        Some(IDENTITY)
    );
    c.points[1][0] += 1e-12;
    assert!(
        Deformation::new(c, IDENTITY, Region::Global)
            .unwrap()
            .exact_affine_transform()
            .unwrap()
            .is_none()
    );
}
#[test]
fn endpoint_interpolation_ignores_inactive_extreme_weights() {
    let mut c = box_cage();
    c.weights = vec![1e300; 8];
    c.weights[7] = 1e-300;
    close(c.evaluate([1.; 3]).unwrap(), [1.; 3]);
}
#[test]
fn unchanged_rotated_reference_stays_in_original_world_space() {
    let mut c = box_cage();
    for p in &mut c.points {
        *p = [10. - 2. * p[0], 6. + 3. * p[1], 4. + 4. * p[2]];
    }
    let frame = [
        -0.5,
        0.,
        0.,
        5.,
        0.,
        1. / 3.,
        0.,
        -2.,
        0.,
        0.,
        0.25,
        -1.,
        0.,
        0.,
        0.,
        1.,
    ];
    let d = Deformation::new(c, frame, Region::Global).unwrap();
    for p in [[9., 7.5, 6.], [15., 0., 12.]] {
        close(d.deform(p).unwrap(), p);
    }
}
#[test]
fn ffi_apply_uses_bound_parameters_and_local_source_distance() {
    let mut c = box_cage();
    for p in &mut c.points {
        p[2] += 2.;
    }
    let mut desc = descriptor(&c);
    desc.region = 1;
    desc.falloff = 2.;
    desc.world_to_parameter[3] = 100.;
    let mut handle = std::ptr::null_mut();
    unsafe {
        assert_eq!(om9_cage_create(&desc, &mut handle), 0);
        let uvw = [0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
        let mut source = [2., 0.5, 0.5, 4., 0.5, 0.5];
        assert_eq!(
            om9_cage_apply(
                handle,
                source.as_ptr(),
                uvw.as_ptr(),
                2,
                source.as_mut_ptr(),
                2
            ),
            0
        );
        close([source[0], source[1], source[2]], [1.25, 0.5, 1.5]);
        close([source[3], source[4], source[5]], [4., 0.5, 0.5]);
        om9_cage_destroy(handle);
    }
}
#[test]
fn ffi_apply_validates_both_inputs_and_stages_atomic_output() {
    let c = box_cage();
    let desc = descriptor(&c);
    let mut handle = std::ptr::null_mut();
    unsafe {
        assert_eq!(om9_cage_create(&desc, &mut handle), 0);
        let good = [0.5; 6];
        let bad = [0.5, 0.5, 0.5, f64::NAN, 0., 0.];
        let mut output = [19.; 6];
        assert!(
            om9_cage_apply(
                handle,
                bad.as_ptr(),
                good.as_ptr(),
                2,
                output.as_mut_ptr(),
                2
            ) < 0
        );
        assert_eq!(output, [19.; 6]);
        assert!(
            om9_cage_apply(
                handle,
                good.as_ptr(),
                bad.as_ptr(),
                2,
                output.as_mut_ptr(),
                2
            ) < 0
        );
        assert_eq!(output, [19.; 6]);
        assert!(
            om9_cage_apply(
                handle,
                good.as_ptr(),
                good.as_ptr(),
                2,
                output.as_mut_ptr(),
                1
            ) < 0
        );
        assert_eq!(output, [19.; 6]);
        assert!(
            om9_cage_apply(
                handle,
                std::ptr::null(),
                good.as_ptr(),
                2,
                output.as_mut_ptr(),
                2
            ) < 0
        );
        assert_eq!(output, [19.; 6]);
        assert_eq!(
            om9_cage_apply(
                handle,
                std::ptr::null(),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0
            ),
            0
        );
        om9_cage_destroy(handle);
    }
}
#[test]
fn box_builder_generates_clamped_mixed_degree_greville_cvs() {
    let (c, frame) = Cage::r#box([4, 7, 2], [2, 4, 1], [10., -4., 3.], [14., 8., 7.]).unwrap();
    assert_eq!(c.knots[0], vec![0., 0., 0., 0.5, 1., 1., 1.]);
    assert_eq!(
        c.knots[1],
        vec![0., 0., 0., 0., 0., 1. / 3., 2. / 3., 1., 1., 1., 1., 1.]
    );
    close(c.points[(1 * 4) + 1], [11., -3., 3.]);
    close(c.evaluate([0.5; 3]).unwrap(), [12., 2., 5.]);
    let d = Deformation::new(c, frame, Region::Global).unwrap();
    close(d.deform([12., 2., 5.]).unwrap(), [12., 2., 5.]);
    assert_eq!(d.exact_affine_transform().unwrap(), Some(IDENTITY));
}
#[test]
fn box_parameters_reject_unsafe_conversions_and_stage_outputs() {
    let mut counts = [99usize; 3];
    let mut degrees = [99usize; 3];
    unsafe {
        for bad in [
            [1e300, 4., 4.],
            [-1., 4., 4.],
            [4.5, 4., 4.],
            [f64::NAN, 4., 4.],
            [128.; 3],
        ] {
            assert!(
                om9_cage_box_parameters(
                    bad.as_ptr(),
                    [1.; 3].as_ptr(),
                    counts.as_mut_ptr(),
                    degrees.as_mut_ptr()
                ) < 0
            );
            assert_eq!(counts, [99; 3]);
            assert_eq!(degrees, [99; 3]);
        }
        assert!(
            om9_cage_box_parameters(
                [4.; 3].as_ptr(),
                [4., 1., 1.].as_ptr(),
                counts.as_mut_ptr(),
                degrees.as_mut_ptr()
            ) < 0
        );
        assert_eq!(counts, [99; 3]);
        assert_eq!(degrees, [99; 3]);
        assert_eq!(
            om9_cage_box_parameters(
                [127., 127., 62.].as_ptr(),
                [1.; 3].as_ptr(),
                counts.as_mut_ptr(),
                degrees.as_mut_ptr()
            ),
            0
        );
        assert_eq!(counts, [127, 127, 62]);
        assert_eq!(degrees, [1; 3]);
    }
}
#[test]
fn box_fill_checks_all_capacities_before_any_output_write() {
    let counts = [3usize, 2, 2];
    let degrees = [2usize, 1, 1];
    let lo = [0.; 3];
    let hi = [2., 4., 6.];
    let mut points = [77.; 36];
    let mut u = [77.; 6];
    let mut v = [77.; 4];
    let mut w = [77.; 4];
    let mut frame = [77.; 16];
    unsafe {
        assert!(
            om9_cage_box_fill(
                counts.as_ptr(),
                degrees.as_ptr(),
                lo.as_ptr(),
                hi.as_ptr(),
                points.as_mut_ptr(),
                12,
                u.as_mut_ptr(),
                6,
                v.as_mut_ptr(),
                4,
                w.as_mut_ptr(),
                3,
                frame.as_mut_ptr()
            ) < 0
        );
        assert_eq!(points, [77.; 36]);
        assert_eq!(u, [77.; 6]);
        assert_eq!(v, [77.; 4]);
        assert_eq!(w, [77.; 4]);
        assert_eq!(frame, [77.; 16]);
        assert!(
            om9_cage_box_fill(
                counts.as_ptr(),
                degrees.as_ptr(),
                lo.as_ptr(),
                [0., 4., 6.].as_ptr(),
                points.as_mut_ptr(),
                12,
                u.as_mut_ptr(),
                6,
                v.as_mut_ptr(),
                4,
                w.as_mut_ptr(),
                4,
                frame.as_mut_ptr()
            ) < 0
        );
        assert_eq!(points, [77.; 36]);
        assert_eq!(frame, [77.; 16]);
        assert_eq!(
            om9_cage_box_fill(
                counts.as_ptr(),
                degrees.as_ptr(),
                lo.as_ptr(),
                hi.as_ptr(),
                points.as_mut_ptr(),
                12,
                u.as_mut_ptr(),
                6,
                v.as_mut_ptr(),
                4,
                w.as_mut_ptr(),
                4,
                frame.as_mut_ptr()
            ),
            0
        );
        assert_eq!(u, [0., 0., 0., 1., 1., 1.]);
        assert_eq!(v, [0., 0., 1., 1.]);
        assert_eq!(w, [0., 0., 1., 1.]);
        close([points[3], points[4], points[5]], [1., 0., 0.]);
        close([points[33], points[34], points[35]], [2., 4., 6.]);
        assert_eq!(
            frame,
            [
                0.5,
                0.,
                0.,
                0.,
                0.,
                0.25,
                0.,
                0.,
                0.,
                0.,
                1. / 6.,
                0.,
                0.,
                0.,
                0.,
                1.
            ]
        );
    }
}
#[test]
fn proven_affine_high_degree_extrapolation_preserves_identity_and_edits() {
    for (degree, point) in [(4, [1e6, 0., 0.]), (16, [10., 0., 0.])] {
        let (mut c, frame) =
            Cage::r#box([degree + 1, 2, 2], [degree, 1, 1], [0.; 3], [1.; 3]).unwrap();
        close(c.evaluate(point).unwrap(), point);
        close(
            Deformation::new(c.clone(), frame, Region::Global)
                .unwrap()
                .deform(point)
                .unwrap(),
            point,
        );
        for p in &mut c.points {
            *p = [2. * p[0] + 3., -p[1] + 2., p[2] + 0.25];
        }
        let expected = [2. * point[0] + 3., 2., 0.25];
        let edited = Deformation::new(c, frame, Region::Global).unwrap();
        close(edited.deform(point).unwrap(), expected);
        close(edited.evaluate_normalized(point).unwrap(), expected);
    }
}
#[test]
fn ffi_affine_far_extrapolation_is_repeatable_for_deform_and_apply() {
    for (degree, point) in [(4, [1e6, 0., 0.]), (16, [10., 0., 0.])] {
        let (mut c, _) = Cage::r#box([degree + 1, 2, 2], [degree, 1, 1], [0.; 3], [1.; 3]).unwrap();
        for p in &mut c.points {
            p[0] = 2. * p[0] + 3.;
        }
        let desc = descriptor(&c);
        let mut handle = std::ptr::null_mut();
        let mut out = [0.; 3];
        unsafe {
            assert_eq!(om9_cage_create(&desc, &mut handle), 0);
            for _ in 0..2 {
                assert_eq!(
                    om9_cage_deform(handle, point.as_ptr(), 1, out.as_mut_ptr(), 1),
                    0
                );
                close(out, [2. * point[0] + 3., 0., 0.]);
                assert_eq!(
                    om9_cage_apply(
                        handle,
                        point.as_ptr(),
                        point.as_ptr(),
                        1,
                        out.as_mut_ptr(),
                        1
                    ),
                    0
                );
                close(out, [2. * point[0] + 3., 0., 0.]);
            }
            om9_cage_destroy(handle);
        }
    }
}
#[test]
fn affine_extrapolation_keeps_local_blending_and_non_affine_basis_behavior() {
    let (mut c, _) = Cage::r#box([5, 2, 2], [4, 1, 1], [0.; 3], [1.; 3]).unwrap();
    for p in &mut c.points {
        *p = [2. * p[0] + 3., -p[1] + 2., p[2] + 0.25];
    }
    let local = Deformation::new(
        c,
        IDENTITY,
        Region::Local {
            min: [0.; 3],
            max: [0.; 3],
            falloff: 2e6,
        },
    )
    .unwrap();
    close(
        local.deform([1e6, 0., 0.]).unwrap(),
        [1.5e6 + 1.5, 1., 0.125],
    );
    let (mut nonlinear, _) = Cage::r#box([5, 2, 2], [4, 1, 1], [0.; 3], [1.; 3]).unwrap();
    for (i, p) in nonlinear.points.iter_mut().enumerate() {
        if i % 5 == 2 {
            p[0] += 0.125;
        }
    }
    let d = Deformation::new(nonlinear, IDENTITY, Region::Global).unwrap();
    assert!(d.exact_affine_transform().unwrap().is_none());
    close(d.deform([1.5, 0., 0.]).unwrap(), [1.921875, 0., 0.]);
}
#[test]
fn rational_tensor_path_retains_high_degree_and_nonuniform_knot_interpolation() {
    // Bernstein weights 2^i give the independent rational mean 2u/(1+u),
    // at every degree. Nonuniform weights prevent the affine shortcut.
    for degree in 1..=16 {
        let (mut c, _) = Cage::r#box([degree + 1, 2, 2], [degree, 1, 1], [0.; 3], [1.; 3]).unwrap();
        for (i, w) in c.weights.iter_mut().enumerate() {
            *w = 2f64.powi((i % (degree + 1)) as i32);
        }
        let d = Deformation::new(c, IDENTITY, Region::Global).unwrap();
        assert!(d.exact_affine_transform().unwrap().is_none());
        close(d.deform([0.25, 0.2, 0.8]).unwrap(), [0.4, 0.2, 0.8]);
    }
    let (mut c, _) = Cage::r#box([4, 2, 2], [2, 1, 1], [0.; 3], [1.; 3]).unwrap();
    c.knots[0] = vec![0., 0., 0., 0.25, 1., 1., 1.];
    for (i, p) in c.points.iter_mut().enumerate() {
        p[0] = [0., 0.125, 0.625, 1.][i % 4];
    }
    for (i, w) in c.weights.iter_mut().enumerate() {
        *w = if i % 4 == 1 { 2. } else { 1. };
    }
    // At u=.4 the nonzero quadratic bases are .48,.48,.04;
    // weighted numerator=.46 and denominator=1.48.
    close(c.evaluate([0.4, 0.2, 0.8]).unwrap(), [23. / 74., 0.2, 0.8]);
}
fn thirty_degree_pose() -> [f64; 16] {
    let cosine = 3f64.sqrt() / 2.;
    [
        cosine, -0.5, 0., 10., 0.5, cosine, 0., -4., 0., 0., 1., 3., 0., 0., 0., 1.,
    ]
}
#[test]
fn local_affine_proof_survives_rotated_translated_pose_at_high_degree() {
    let pose = thirty_degree_pose();
    for degree in [1, 4, 16] {
        let (mut c, _) = Cage::r#box([degree + 1, 2, 2], [degree, 1, 1], [0.; 3], [1.; 3]).unwrap();
        let d = Deformation::with_pose(c.clone(), IDENTITY, Region::Global, pose).unwrap();
        assert_eq!(d.exact_affine_transform().unwrap(), Some(pose));
        close(
            d.evaluate_normalized([0.5; 3]).unwrap(),
            [10.183012701892219, -3.316987298107781, 3.5],
        );
        close(
            d.inverse_point([10.183012701892219, -3.316987298107781, 3.5])
                .unwrap(),
            [0.5; 3],
        );
        for p in &mut c.points {
            p[0] = 2. * p[0] + 3.;
        }
        let edited = Deformation::with_pose(c, IDENTITY, Region::Global, pose).unwrap();
        assert!(edited.exact_affine_transform().unwrap().is_some());
        close(
            edited.evaluate_normalized([0.5; 3]).unwrap(),
            [13.214101615137755, -1.5669872981077807, 3.5],
        );
    }
}
#[test]
fn placed_nonlinear_inverse_and_local_binding_use_world_coordinates() {
    let mut c = box_cage();
    c.points[7][2] = 2.;
    let pose = thirty_degree_pose();
    let source = [10.183012701892219, -3.316987298107781, 3.625];
    let current = Deformation::with_pose(c.clone(), IDENTITY, Region::Global, pose).unwrap();
    assert!(current.exact_affine_transform().unwrap().is_none());
    close(current.evaluate_normalized([0.5; 3]).unwrap(), source);
    close(current.inverse_point(source).unwrap(), [0.5; 3]);
    let binding = CageBinding::capture(&current, &[source]).unwrap();
    c.points[7][2] = 3.;
    let edited = Deformation::with_pose(
        c,
        IDENTITY,
        Region::Local {
            min: [source[0] - 1., source[1], source[2]],
            max: [source[0] - 1., source[1], source[2]],
            falloff: 2.,
        },
        pose,
    )
    .unwrap();
    close(
        binding.apply(&edited).unwrap()[0],
        [source[0], source[1], 3.6875],
    );
}
#[test]
fn ffi_placed_affine_cage_keeps_proof_and_applies_pose_once() {
    let (c, _) = Cage::r#box([5, 2, 2], [4, 1, 1], [0.; 3], [1.; 3]).unwrap();
    let desc = descriptor(&c);
    let pose = thirty_degree_pose();
    let mut handle = std::ptr::null_mut();
    unsafe {
        assert_eq!(om9_cage_create_placed(&desc, pose.as_ptr(), &mut handle), 0);
        let mut affine = [0.; 16];
        assert_eq!(om9_cage_affine(handle, affine.as_mut_ptr()), 0);
        assert_eq!(affine, pose);
        let parameters = [0.5; 3];
        let world = [10.183012701892219, -3.316987298107781, 3.5];
        let mut out = [0.; 3];
        assert_eq!(
            om9_cage_evaluate(handle, parameters.as_ptr(), 1, out.as_mut_ptr(), 1),
            0
        );
        close(out, world);
        assert_eq!(
            om9_cage_inverse(handle, world.as_ptr(), 1, out.as_mut_ptr(), 1),
            0
        );
        close(out, parameters);
        assert_eq!(
            om9_cage_apply(
                handle,
                world.as_ptr(),
                parameters.as_ptr(),
                1,
                out.as_mut_ptr(),
                1
            ),
            0
        );
        close(out, world);
        assert_eq!(
            om9_cage_deform(handle, parameters.as_ptr(), 1, out.as_mut_ptr(), 1),
            0
        );
        close(out, world);
        om9_cage_destroy(handle);
        handle = std::ptr::null_mut();
        assert!(om9_cage_create_placed(&desc, std::ptr::null(), &mut handle) < 0);
        assert!(handle.is_null());
        assert!(om9_cage_create_placed(&desc, [0.; 16].as_ptr(), &mut handle) < 0);
        assert!(handle.is_null());
    }
}
#[test]
fn posed_cage_preserves_original_world_reference_for_deformation() {
    let (c, reference) = Cage::r#box([5, 2, 2], [4, 1, 1], [2., 4., 6.], [4., 8., 10.]).unwrap();
    let d = Deformation::with_pose(c, reference, Region::Global, thirty_degree_pose()).unwrap();
    assert!(d.exact_affine_transform().unwrap().is_some());
    let current_world = [9.598076211353316, 2.696152422706632, 11.];
    close(d.deform([3., 6., 8.]).unwrap(), current_world);
    close(d.evaluate_normalized([0.5; 3]).unwrap(), current_world);
    close(d.inverse_point(current_world).unwrap(), [0.5; 3]);
}
