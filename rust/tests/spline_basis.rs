use openmatrix9_rust::spline::{self, Basis, Knots};

#[test]
fn nurbs_basis_rejects_invalid_weights_multiplicities_domain_and_extreme_degree() {
    let poles = [[0., 0., 0.], [1., 2., 0.], [3., 0., 1.], [4., 1., 0.]];
    let mut basis = Basis {
        poles: &poles, weights: None, knots: &[0., 1.], multiplicities: &[4, 4],
        degree: 3, periodic: false, domain: [0., 1.],
    };
    assert!(basis.validate().is_ok());
    for weights in [&[1., 0., 1., 1.][..], &[1., -1., 1., 1.], &[1., f64::NAN, 1., 1.],
        &[1., 1., 1.], &[1., 1., 1., f64::MAX]] {
        basis.weights = Some(weights);
        assert!(basis.validate().is_err());
    }
    basis.weights = Some(&[1., 0.5, 2., 1.]);
    assert!(basis.validate().is_ok());
    basis.weights = None;
    for mults in [&[0, 8][..], &[4, 3], &[usize::MAX, 1]] {
        basis.multiplicities = mults;
        assert!(basis.validate().is_err());
    }
    basis.multiplicities = &[4, 4];
    for domain in [[0., 0.], [-1., 1.], [0., 2.], [0., f64::NAN]] {
        basis.domain = domain;
        assert!(basis.validate().is_err());
    }
    basis.domain = [0., 1.];
    basis.degree = usize::MAX;
    assert!(basis.validate().is_err());
}

#[test]
fn periodic_and_unclamped_bases_are_not_forced_to_have_clamped_end_multiplicities() {
    let poles = [[0., 0., 0.], [1., 2., 0.], [3., 0., 1.], [4., 1., 0.]];
    let mut basis = Basis {
        poles: &poles, weights: None, knots: &[0., 0.25, 0.5, 0.75, 1.],
        multiplicities: &[1, 1, 1, 1, 1], degree: 3, periodic: true, domain: [0., 1.],
    };
    assert!(basis.validate().is_ok());
    basis.multiplicities = &[1, 1, 1, 1, 2];
    assert!(basis.validate().is_err());
    basis.periodic = false;
    basis.knots = &[-3., -2., -1., 0., 1., 2., 3., 4.];
    basis.multiplicities = &[1, 1, 1, 1, 1, 1, 1, 1];
    assert!(basis.validate().is_ok());
    basis.domain = [-1., 1.];
    assert!(basis.validate().is_err());
}

#[test]
fn generated_periodic_extension_is_validated_before_host_knot_slicing() {
    let points = [[0., 0., 0.], [1., 2., 0.], [3., 0., 1.], [4., 1., 0.]];
    for periodic in [false, true] {
        for mode in [Knots::Uniform, Knots::Chord, Knots::SqrtChord] {
            let curve = spline::interpolate(&points, 3, mode, periodic).unwrap();
            assert!(curve.validate().is_ok());
        }
    }
    let mut curve = spline::interpolate(&points, 3, Knots::Uniform, true).unwrap();
    curve.knots[0] -= 0.25;
    assert!(curve.validate().is_err());
    curve.knots.clear();
    assert!(curve.validate().is_err());
}
