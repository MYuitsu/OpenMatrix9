use openmatrix9_rust::circle_tangent::{Constraint, Options, solve};
#[test]
fn two_nonplanar_curves_respect_fixed_radius() {
    let cs = [
        Constraint::Curve {
            index: 0,
            seed: 0.55,
        },
        Constraint::Curve {
            index: 1,
            seed: 0.55,
        },
    ];
    let r = solve(
        &cs,
        &Options {
            radius: Some(5.),
            ..Options::default()
        },
        curve,
    )
    .unwrap();
    assert!((r.plan.radius - 5.).abs() < 1e-6);
    assert!(r.plan.center.iter().all(|v| v.abs() < 1e-5));
}
#[test]
fn spatial_solver_is_rigid_and_scale_invariant() {
    let cs = (0..3)
        .map(|i| Constraint::Curve {
            index: i,
            seed: 0.54,
        })
        .collect::<Vec<_>>();
    for scale in [0.01, 1000.] {
        let r = solve(
            &cs,
            &Options {
                vertical: true,
                ..Options::default()
            },
            |i, u| {
                let (p, t) = curve(i, u)?;
                Ok((
                    [
                        1000. + scale * p[2],
                        2000. + scale * p[1],
                        3000. - scale * p[0],
                    ],
                    [t[2], t[1], -t[0]],
                ))
            },
        )
        .unwrap();
        assert!((r.plan.radius - 5. * scale).abs() < 1e-5);
        assert!(
            r.plan
                .center
                .iter()
                .zip([1000., 2000., 3000.])
                .all(|(a, b)| (a - b).abs() < 1e-5)
        );
        assert!((r.plan.normal[0].abs() - 1.).abs() < 1e-8);
    }
}
#[test]
fn spatial_ffi_failure_preserves_plan_and_returns_bounded_error() {
    use openmatrix9_rust::circle_tangent::om9_circle_spatial_solve;
    unsafe extern "C" fn fail(_: *mut std::ffi::c_void, _: usize, _: f64, _: *mut f64) -> bool {
        false
    }
    let b = [0., 0., 0., 0., 1., 0., 0., 0.];
    let seeds = [0.5; 2];
    let f = [0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1., 1.];
    let mut out = [123.; 10];
    let mut error = [9i8; 8];
    assert!(!unsafe {
        om9_circle_spatial_solve(
            b.as_ptr(),
            seeds.as_ptr(),
            2,
            f.as_ptr(),
            false,
            false,
            -1,
            Some(fail),
            std::ptr::null_mut(),
            out.as_mut_ptr(),
            error.as_mut_ptr(),
            error.len(),
        )
    });
    assert_eq!(out, [123.; 10]);
    assert_eq!(error[7], 0);
}
fn curve(i: usize, u: f64) -> Result<([f64; 3], [f64; 3]), String> {
    let a = i as f64 * std::f64::consts::TAU / 3.;
    let (s, c) = a.sin_cos();
    let v = 2. * u - 1.;
    Ok((
        [
            5. * c - s * v + c * 0.3 * v * v,
            5. * s + c * v + s * 0.3 * v * v,
            v * v + 0.4 * v * v * v,
        ],
        [-s + 0.6 * c * v, c + 0.6 * s * v, 2. * v + 1.2 * v * v],
    ))
}
#[test]
fn nonplanar_contacts_move_to_exact_circle() {
    let cs = (0..3)
        .map(|i| Constraint::Curve {
            index: i,
            seed: 0.55,
        })
        .collect::<Vec<_>>();
    let result = solve(&cs, &Options::default(), curve).unwrap();
    assert!(result.plan.center.iter().all(|v| v.abs() < 1e-5));
    assert!((result.plan.radius - 5.).abs() < 1e-5);
    assert!(result.fractions.iter().all(|u| (u - 0.5).abs() < 1e-5));
}
#[test]
fn nonplanar_from_first_keeps_exact_parameter() {
    let cs = (0..3)
        .map(|i| Constraint::Curve {
            index: i,
            seed: if i == 0 { 0.5 } else { 0.54 },
        })
        .collect::<Vec<_>>();
    let r = solve(
        &cs,
        &Options {
            from_first: true,
            ..Options::default()
        },
        curve,
    )
    .unwrap();
    assert_eq!(r.fractions[0], 0.5);
    assert!((r.plan.radius - 5.).abs() < 1e-5);
}
#[test]
fn spatial_radius_and_freepoint_are_verified() {
    let cs = [
        Constraint::Curve {
            index: 0,
            seed: 0.53,
        },
        Constraint::Point([-5., 0., 0.]),
    ];
    let r = solve(
        &cs,
        &Options {
            radius: Some(5.),
            ..Options::default()
        },
        curve,
    )
    .unwrap();
    assert!((r.plan.radius - 5.).abs() < 1e-6);
    let d: [f64; 3] = std::array::from_fn(|i| [-5., 0., 0.][i] - r.plan.center[i]);
    assert!((d.iter().map(|v| v * v).sum::<f64>().sqrt() - 5.).abs() < 1e-6);
    let (p, t) = curve(0, r.fractions[0]).unwrap();
    let d: [f64; 3] = std::array::from_fn(|i| p[i] - r.plan.center[i]);
    assert!(d.iter().zip(t).map(|(a, b)| a * b).sum::<f64>().abs() < 1e-6);
}
#[test]
fn skew_lines_and_callback_failure_reject() {
    let cs = [
        Constraint::Curve {
            index: 0,
            seed: 0.5,
        },
        Constraint::Curve {
            index: 1,
            seed: 0.5,
        },
    ];
    let o = Options {
        radius: Some(1.),
        ..Options::default()
    };
    assert!(
        solve(&cs, &o, |i, u| Ok((
            if i == 0 { [u, 0., 0.] } else { [0., u, 1.] },
            if i == 0 { [1., 0., 0.] } else { [0., 1., 0.] }
        )))
        .is_err()
    );
    assert!(solve(&cs, &o, |_, _| Err("stale source".into())).is_err());
}
#[test]
fn spatial_inputs_and_vertical_fail_atomically() {
    let cs = (0..3)
        .map(|i| Constraint::Curve {
            index: i,
            seed: 0.5,
        })
        .collect::<Vec<_>>();
    assert!(
        solve(
            &cs,
            &Options {
                vertical: true,
                from_first: true,
                ..Options::default()
            },
            curve
        )
        .is_err()
    );
    assert!(
        solve(
            &cs,
            &Options {
                radius: Some(f64::NAN),
                ..Options::default()
            },
            curve
        )
        .is_err()
    );
    assert!(
        solve(
            &[Constraint::Curve { index: 0, seed: 2. }],
            &Options::default(),
            curve
        )
        .is_err()
    );
}
