use openmatrix9_rust::phase2_curve::{Basis, join_options, validate_wire};
use openmatrix9_rust::phase2_session::{
    CurveSession, RebuildInput, RebuildOptions, RebuildSession, Witness,
};
fn basis() -> Basis {
    Basis {
        degree: 2,
        periodic: false,
        poles: vec![[0., 0., 0.], [1., 2., 0.], [2., 0., 0.]],
        weights: vec![1., 0.7, 1.],
        knots: vec![0., 1.],
        multiplicities: vec![3, 3],
        first: 0.,
        last: 1.,
    }
}
fn witness() -> Witness {
    Witness {
        document: 1,
        object: 2,
        generation: 3,
        signature: "rational-local-basis".into(),
    }
}
#[test]
fn rational_owned_basis_preserves_exact_fields() {
    let b = basis();
    assert!(b.validate().is_ok());
    let session = CurveSession::new(witness(), b.clone()).unwrap();
    assert_eq!(session.commit_request(&witness()).unwrap(), b);
}
#[test]
fn invalid_numeric_basis_is_rejected() {
    let mut cases = Vec::new();
    for degree in [0, 26] {
        let mut b = basis();
        b.degree = degree;
        cases.push(b);
    }
    for weight in [0., -1., f64::NAN, 1e10] {
        let mut b = basis();
        b.weights[1] = weight;
        cases.push(b);
    }
    for value in [f64::INFINITY, 1e10] {
        let mut b = basis();
        b.poles[1][0] = value;
        cases.push(b);
    }
    for knots in [vec![1., 0.], vec![0., 0.]] {
        let mut b = basis();
        b.knots = knots;
        cases.push(b);
    }
    let mut b = basis();
    b.multiplicities[0] = 2;
    cases.push(b);
    let mut b = basis();
    b.last = 2.;
    cases.push(b);
    let mut b = basis();
    b.weights.pop();
    cases.push(b);
    let mut b = basis();
    b.poles.resize(4097, [0.; 3]);
    b.weights.resize(4097, 1.);
    cases.push(b);
    for b in cases {
        assert!(b.validate().is_err(), "accepted {b:?}");
    }
}
#[test]
fn exact_budget_and_periodic_relation_are_checked() {
    let n = 4096;
    let mut b = Basis {
        degree: 1,
        periodic: false,
        poles: vec![[0.; 3]; n],
        weights: vec![1.; n],
        knots: (0..n).map(|x| x as f64).collect(),
        multiplicities: vec![1; n],
        first: 0.,
        last: (n - 1) as f64,
    };
    b.multiplicities[0] = 2;
    b.multiplicities[n - 1] = 2;
    assert!(b.validate().is_ok());
    let p = Basis {
        degree: 2,
        periodic: true,
        poles: vec![[0.; 3]; 3],
        weights: vec![1.; 3],
        knots: vec![0., 1., 2., 3.],
        multiplicities: vec![1; 4],
        first: 0.,
        last: 3.,
    };
    assert!(p.validate().is_ok());
    let mut bad = p;
    bad.multiplicities[3] = 2;
    assert!(bad.validate().is_err());
}
#[test]
fn complete_nonbranching_traversal_is_required() {
    assert!(validate_wire(&[1, 2, 1], &[9, 10], 2).is_ok());
    assert!(validate_wire(&[3, 1, 1, 1], &[9, 10], 3).is_err());
    assert!(validate_wire(&[1, 2, 1], &[9, 9], 2).is_err());
    assert!(validate_wire(&[1, 1, 1, 1], &[9], 2).is_err());
    assert!(join_options(2, 64).is_ok());
    assert!(join_options(17, 2).is_err());
    assert!(join_options(2, 65).is_err());
}
#[test]
fn draft_is_atomic_and_stale_cancelled_requests_fail() {
    let mut s = CurveSession::new(witness(), basis()).unwrap();
    let mut b = basis();
    b.weights[1] = -1.;
    assert!(s.replace_draft(b).is_err());
    assert_eq!(s.commit_request(&witness()).unwrap(), basis());
    let mut changed = witness();
    changed.generation += 1;
    assert!(s.commit_request(&changed).is_err());
    s.cancel();
    assert!(s.commit_request(&witness()).is_err());
    assert!(s.replace_draft(basis()).is_err());
}
#[test]
fn rebuild_options_dependencies_and_all_witnesses_are_rust_owned() {
    let input = RebuildInput {
        witness: witness(),
        has_dependents: false,
    };
    let options = RebuildOptions {
        degree: 3,
        point_count: 12,
        delete_input: true,
    };
    let mut s = RebuildSession::new(vec![input.clone()], options.clone()).unwrap();
    assert_eq!(
        s.commit_request(std::slice::from_ref(&input)).unwrap(),
        options
    );
    let mut dependent = input.clone();
    dependent.has_dependents = true;
    assert!(s.commit_request(&[dependent]).is_err());
    let mut invalid = options.clone();
    invalid.point_count = 3;
    assert!(s.replace_options(invalid).is_err());
    assert_eq!(
        s.commit_request(std::slice::from_ref(&input)).unwrap(),
        options
    );
    assert!(RebuildSession::new(vec![input.clone(); 17], options).is_err());
    s.cancel();
    assert!(s.commit_request(&[input]).is_err());
}
