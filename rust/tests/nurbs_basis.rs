use openmatrix9_rust::nurbs_basis::{
    Axis, BasisError, BasisPolicy, CurveBasis, SurfaceBasis, preflight_curve_counts,
    preflight_surface_counts,
};
use openmatrix9_rust::spline;

fn policy() -> BasisPolicy {
    BasisPolicy {
        max_degree: 25,
        max_poles: 4096,
        max_knots: 4096,
        max_bytes: 1_048_576,
        max_coordinate: 1e12,
    }
}

fn axis<'a>(degree: usize, knots: &'a [f64], multiplicities: &'a [usize]) -> Axis<'a> {
    Axis {
        degree,
        knots,
        multiplicities,
        periodic: false,
        domain: [0., 1.],
    }
}

fn line<'a>(poles: &'a [[f64; 3]]) -> CurveBasis<'a> {
    CurveBasis {
        poles,
        weights: None,
        axis: axis(1, &[0., 1.], &[2, 2]),
    }
}

#[test]
fn native_rational_circle_basis_accepts_positive_weights() {
    // Four rational quadratic arcs: nine poles, endpoint multiplicity three.
    let poles = [
        [10., 0., 0.],
        [10., 10., 0.],
        [0., 10., 0.],
        [-10., 10., 0.],
        [-10., 0., 0.],
        [-10., -10., 0.],
        [0., -10., 0.],
        [10., -10., 0.],
        [10., 0., 0.],
    ];
    let q = std::f64::consts::FRAC_1_SQRT_2;
    let weights = [1., q, 1., q, 1., q, 1., q, 1.];
    let basis = CurveBasis {
        poles: &poles,
        weights: Some(&weights),
        axis: axis(2, &[0., 1., 2., 3., 4.], &[3, 2, 2, 2, 3]),
    };
    assert_eq!(basis.validate(policy()), Ok(()));
}

#[test]
fn explicit_native_policy_accepts_more_than_256_poles() {
    let poles = vec![[1., 0., 0.]; 300];
    let knots: Vec<_> = (0..300).map(f64::from).collect();
    let mut mults = vec![1; 300];
    mults[0] = 2;
    mults[299] = 2;
    let basis = CurveBasis {
        poles: &poles,
        weights: None,
        axis: Axis {
            domain: [0., 299.],
            ..axis(1, &knots, &mults)
        },
    };
    assert_eq!(basis.validate(policy()), Ok(()));
    let generated = spline::Basis {
        poles: &poles,
        weights: None,
        knots: &knots,
        multiplicities: &mults,
        degree: 1,
        periodic: false,
        domain: [0., 299.],
    };
    assert!(generated.validate().is_err());
}

#[test]
fn explicit_native_policy_accepts_degree_and_coordinates_beyond_generated_limits() {
    let poles = [[2e9, 0., 0.]; 13];
    let basis = CurveBasis {
        poles: &poles,
        weights: None,
        axis: axis(12, &[0., 1.], &[13, 13]),
    };
    assert_eq!(basis.validate(policy()), Ok(()));
    let generated = spline::Basis {
        poles: &poles,
        weights: None,
        knots: &[0., 1.],
        multiplicities: &[13, 13],
        degree: 12,
        periodic: false,
        domain: [0., 1.],
    };
    assert!(generated.validate().is_err());
}

#[test]
fn periodic_poles_follow_native_seam_count_without_generated_degree_plus_one_limit() {
    let poles = [[0., 0., 0.], [1., 1., 0.]];
    let basis = CurveBasis {
        poles: &poles,
        weights: None,
        axis: Axis {
            periodic: true,
            ..axis(3, &[0., 0.5, 1.], &[1, 1, 1])
        },
    };
    assert_eq!(basis.validate(policy()), Ok(()));
}

#[test]
fn unclamped_curve_domain_uses_active_expanded_knot_interval() {
    let poles = [[0., 0., 0.]; 4];
    let mut basis = CurveBasis {
        poles: &poles,
        weights: None,
        axis: axis(3, &[-3., -2., -1., 0., 1., 2., 3., 4.], &[1; 8]),
    };
    assert_eq!(basis.validate(policy()), Ok(()));
    basis.axis.domain = [-1., 1.];
    assert_eq!(basis.validate(policy()), Err(BasisError::Domain));
}

#[test]
fn rational_surface_grid_accepts_distinct_u_v_axes() {
    // U-major: each U row contains three V poles; a quadratic V / linear U patch.
    let poles = [
        [0., 0., 0.],
        [0., 1., 1.],
        [0., 2., 0.],
        [1., 0., 0.],
        [1., 1., 1.],
        [1., 2., 0.],
    ];
    let basis = SurfaceBasis {
        poles: &poles,
        weights: Some(&[1., 0.5, 1., 1., 0.5, 1.]),
        u_count: 2,
        v_count: 3,
        axes: [axis(1, &[0., 1.], &[2, 2]), axis(2, &[0., 1.], &[3, 3])],
    };
    assert_eq!(basis.validate(policy()), Ok(()));
}

#[test]
fn surface_can_mix_periodic_and_unclamped_axes() {
    let poles = [[0., 0., 0.]; 8];
    let basis = SurfaceBasis {
        poles: &poles,
        weights: None,
        u_count: 2,
        v_count: 4,
        axes: [
            Axis {
                periodic: true,
                ..axis(3, &[0., 0.5, 1.], &[1, 1, 1])
            },
            axis(3, &[-3., -2., -1., 0., 1., 2., 3., 4.], &[1; 8]),
        ],
    };
    assert_eq!(basis.validate(policy()), Ok(()));
}

#[test]
fn invalid_policies_reject_before_count_or_data_inspection() {
    let mut cases = vec![];
    for max_degree in [0, usize::MAX] {
        cases.push(BasisPolicy {
            max_degree,
            ..policy()
        });
    }
    for max_poles in [0, 1] {
        cases.push(BasisPolicy {
            max_poles,
            ..policy()
        });
    }
    for max_knots in [0, 1] {
        cases.push(BasisPolicy {
            max_knots,
            ..policy()
        });
    }
    cases.push(BasisPolicy {
        max_bytes: 0,
        ..policy()
    });
    for max_coordinate in [0., -1., f64::NAN, f64::INFINITY] {
        cases.push(BasisPolicy {
            max_coordinate,
            ..policy()
        });
    }
    for p in cases {
        assert_eq!(p.validate(), Err(BasisError::InvalidPolicy));
        assert_eq!(
            preflight_curve_counts(usize::MAX, None, 2, 2, p),
            Err(BasisError::InvalidPolicy)
        );
    }
}

#[test]
fn degree_preflight_rejects_zero_above_policy_and_overflow() {
    for degree in [0, 26, usize::MAX] {
        assert_eq!(policy().validate_degree(degree), Err(BasisError::Degree));
    }
    assert_eq!(policy().validate_degree(25), Ok(()));
}

#[test]
fn curve_preflight_checks_counts_and_exact_payload_byte_limit() {
    // Two XYZ poles=48 bytes, two weights=16, two knots=16, two native usize mults.
    let bytes = 80 + 2 * std::mem::size_of::<usize>();
    assert_eq!(
        preflight_curve_counts(
            2,
            Some(2),
            2,
            2,
            BasisPolicy {
                max_bytes: bytes,
                ..policy()
            }
        ),
        Ok(bytes)
    );
    assert_eq!(
        preflight_curve_counts(
            2,
            Some(2),
            2,
            2,
            BasisPolicy {
                max_bytes: bytes - 1,
                ..policy()
            }
        ),
        Err(BasisError::ByteLimit)
    );
    assert_eq!(
        preflight_curve_counts(1, None, 2, 2, policy()),
        Err(BasisError::PoleCount)
    );
    assert_eq!(
        preflight_curve_counts(4097, None, 2, 2, policy()),
        Err(BasisError::PoleLimit)
    );
    assert_eq!(
        preflight_curve_counts(2, Some(1), 2, 2, policy()),
        Err(BasisError::Weights)
    );
    assert_eq!(
        preflight_curve_counts(2, None, 1, 1, policy()),
        Err(BasisError::Knots)
    );
    assert_eq!(
        preflight_curve_counts(2, None, 2, 1, policy()),
        Err(BasisError::Knots)
    );
    assert_eq!(
        preflight_curve_counts(2, None, 4097, 4097, policy()),
        Err(BasisError::KnotLimit)
    );
}

#[test]
fn surface_preflight_checks_grid_product_truncation_and_total_pole_policy() {
    let bytes = 4 * 24 + 4 * (8 + std::mem::size_of::<usize>());
    assert_eq!(
        preflight_surface_counts(2, 2, 4, None, [2, 2], [2, 2], policy()),
        Ok(bytes)
    );
    assert_eq!(
        preflight_surface_counts(2, 2, 3, None, [2, 2], [2, 2], policy()),
        Err(BasisError::GridCountMismatch)
    );
    assert_eq!(
        preflight_surface_counts(1, 4, 4, None, [2, 2], [2, 2], policy()),
        Err(BasisError::GridDimensions)
    );
    assert_eq!(
        preflight_surface_counts(64, 65, 4160, None, [2, 2], [2, 2], policy()),
        Err(BasisError::PoleLimit)
    );
    assert_eq!(
        preflight_surface_counts(2, 2, 4, Some(3), [2, 2], [2, 2], policy()),
        Err(BasisError::Weights)
    );
    assert_eq!(
        preflight_surface_counts(2, 2, 4, None, [2, 3], [2, 2], policy()),
        Err(BasisError::Knots)
    );
}

#[test]
fn preflight_rejects_usize_overflow_without_allocating() {
    let p = BasisPolicy {
        max_poles: usize::MAX,
        max_knots: usize::MAX,
        max_bytes: usize::MAX,
        ..policy()
    };
    assert_eq!(
        preflight_surface_counts(usize::MAX, 2, 0, None, [2, 2], [2, 2], p),
        Err(BasisError::CountOverflow)
    );
    assert_eq!(
        preflight_curve_counts(usize::MAX, None, 2, 2, p),
        Err(BasisError::CountOverflow)
    );
    assert_eq!(
        preflight_curve_counts(2, None, usize::MAX, usize::MAX, p),
        Err(BasisError::CountOverflow)
    );
}

#[test]
fn curve_rejects_nonfinite_or_out_of_policy_poles() {
    for coordinate in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e12 + 1.] {
        assert_eq!(
            line(&[[coordinate, 0., 0.], [1., 0., 0.]]).validate(policy()),
            Err(BasisError::Poles)
        );
    }
    assert_eq!(
        line(&[[1e12, -1e12, 0.], [1., 0., 0.]]).validate(policy()),
        Ok(())
    );
}

#[test]
fn curve_rejects_wrong_nonpositive_nonfinite_and_overflowing_weights() {
    let poles = [[2., 0., 0.], [1., 0., 0.]];
    for weights in [
        &[1.][..],
        &[0., 1.],
        &[-1., 1.],
        &[f64::NAN, 1.],
        &[f64::INFINITY, 1.],
        &[f64::MAX, 1.],
    ] {
        assert_eq!(
            CurveBasis {
                weights: Some(weights),
                ..line(&poles)
            }
            .validate(policy()),
            Err(BasisError::Weights)
        );
    }
    assert_eq!(
        CurveBasis {
            weights: Some(&[f64::MIN_POSITIVE, 1.]),
            ..line(&poles)
        }
        .validate(policy()),
        Ok(())
    );
}

#[test]
fn knots_reject_mismatch_nonfinite_nonincreasing_and_interval_overflow() {
    let poles = [[0., 0., 0.]; 2];
    for knots in [
        &[0.][..],
        &[0., 0.],
        &[1., 0.],
        &[0., f64::NAN],
        &[0., f64::INFINITY],
        &[-f64::MAX, f64::MAX],
    ] {
        assert_eq!(
            CurveBasis {
                axis: axis(1, knots, &[2, 2]),
                ..line(&poles)
            }
            .validate(policy()),
            Err(BasisError::Knots)
        );
    }
}

#[test]
fn multiplicities_reject_zero_internal_excess_and_checked_sum_overflow() {
    let poles = [[0., 0., 0.]; 3];
    for multiplicities in [&[2, 0, 2][..], &[2, 2, 2], &[3, 1, 2]] {
        let basis = CurveBasis {
            poles: &poles,
            weights: None,
            axis: axis(1, &[0., 0.5, 1.], multiplicities),
        };
        assert_eq!(basis.validate(policy()), Err(BasisError::Multiplicity));
    }
    let p = BasisPolicy {
        max_degree: usize::MAX - 1,
        ..policy()
    };
    let basis = CurveBasis {
        poles: &poles,
        weights: None,
        axis: axis(usize::MAX - 1, &[0., 1.], &[usize::MAX, usize::MAX]),
    };
    assert_eq!(basis.validate(p), Err(BasisError::CountOverflow));
}

#[test]
fn periodic_axes_reject_seam_mismatch_and_wrong_pole_cardinality() {
    let poles = [[0., 0., 0.]; 4];
    for multiplicities in [&[1, 1, 1, 1, 2][..], &[1, 1, 1, 2, 1]] {
        let basis = CurveBasis {
            poles: &poles,
            weights: None,
            axis: Axis {
                periodic: true,
                ..axis(3, &[0., 0.25, 0.5, 0.75, 1.], multiplicities)
            },
        };
        assert_eq!(
            basis.validate(policy()),
            Err(BasisError::PeriodicCardinality)
        );
    }
}

#[test]
fn nonperiodic_axes_reject_wrong_cardinality_and_empty_active_interval() {
    let poles = [[0., 0., 0.]; 2];
    assert_eq!(
        CurveBasis {
            axis: axis(1, &[0., 1.], &[2, 1]),
            ..line(&poles)
        }
        .validate(policy()),
        Err(BasisError::Cardinality)
    );
    // Native minimum two poles does not imply an active interval for degree three.
    assert_eq!(
        CurveBasis {
            axis: axis(3, &[0., 1.], &[3, 3]),
            ..line(&poles)
        }
        .validate(policy()),
        Err(BasisError::Domain)
    );
}

#[test]
fn domains_reject_nonfinite_reversed_zero_outside_and_overflow() {
    let poles = [[0., 0., 0.]; 2];
    for domain in [
        [0., f64::NAN],
        [0., f64::INFINITY],
        [1., 0.],
        [0., 0.],
        [-1., 1.],
        [0., 2.],
        [-f64::MAX, f64::MAX],
    ] {
        let basis = CurveBasis {
            axis: Axis {
                domain,
                ..axis(1, &[0., 1.], &[2, 2])
            },
            ..line(&poles)
        };
        assert_eq!(basis.validate(policy()), Err(BasisError::Domain));
    }
}

#[test]
fn surface_validation_rejects_bad_grid_weights_and_each_axis_domain() {
    let poles = [[0., 0., 0.]; 4];
    let basis = SurfaceBasis {
        poles: &poles,
        weights: None,
        u_count: 2,
        v_count: 2,
        axes: [axis(1, &[0., 1.], &[2, 2]); 2],
    };
    assert_eq!(
        SurfaceBasis {
            poles: &poles[..3],
            ..basis
        }
        .validate(policy()),
        Err(BasisError::GridCountMismatch)
    );
    assert_eq!(
        SurfaceBasis {
            weights: Some(&[1., 1., 0., 1.]),
            ..basis
        }
        .validate(policy()),
        Err(BasisError::Weights)
    );
    for index in [0, 1] {
        let mut invalid = basis;
        invalid.axes[index].domain = [0., 2.];
        assert_eq!(invalid.validate(policy()), Err(BasisError::Domain));
    }
}
