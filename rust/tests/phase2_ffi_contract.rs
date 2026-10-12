use openmatrix9_rust::phase2_ffi::*;
const POLES: [f64; 9] = [0.; 9];
const WEIGHTS: [f64; 3] = [1.; 3];
const KNOTS: [f64; 2] = [0., 1.];
const MULTS: [f64; 2] = [3., 3.];
fn input<'a>(p: &'a [f64], w: &'a [f64], k: &'a [f64], m: &'a [f64]) -> Om9BasisInput {
    Om9BasisInput {
        degree: 2.,
        periodic: 0,
        poles: p.as_ptr(),
        poles_len: p.len(),
        weights: w.as_ptr(),
        weights_len: w.len(),
        knots: k.as_ptr(),
        knots_len: k.len(),
        multiplicities: m.as_ptr(),
        multiplicities_len: m.len(),
        first: 0.,
        last: 1.,
    }
}
#[test]
fn owned_draft_copy_reports_capacity_and_never_writes_short_buffers() {
    unsafe {
        let sig = b"draft";
        let w = Om9WitnessInput {
            document: 1,
            object: 2,
            generation: 1,
            signature: sig.as_ptr(),
            signature_len: sig.len(),
        };
        let b = input(&POLES, &WEIGHTS, &KNOTS, &MULTS);
        let h = om9_phase2_session_create(&w, &b);
        let mut out = Om9BasisOutput::default();
        assert!(!om9_phase2_session_copy(h, &mut out));
        assert_eq!(out.poles_len, 9);
        let mut p = [42.; 9];
        let mut weights = [42.; 3];
        let mut knots = [42.; 2];
        let mut mults = [42.; 2];
        out.poles = p.as_mut_ptr();
        out.weights = weights.as_mut_ptr();
        out.knots = knots.as_mut_ptr();
        out.multiplicities = mults.as_mut_ptr();
        assert!(om9_phase2_session_copy(h, &mut out));
        assert_eq!(p, POLES);
        assert_eq!(weights, WEIGHTS);
        assert_eq!(knots, KNOTS);
        assert_eq!(mults, MULTS);
        out.poles_len = 2;
        p[0] = 42.;
        assert!(!om9_phase2_session_copy(h, &mut out));
        assert_eq!(p[0], 42.);
        om9_phase2_session_drop(h);
    }
}
#[test]
fn malformed_ffi_rejects_before_dereferencing_invalid_buffers() {
    unsafe {
        assert!(!om9_phase2_basis_validate(std::ptr::null()));
        let mut b = input(&POLES, &WEIGHTS, &KNOTS, &MULTS);
        // A pointer into valid backing storage with deliberately wrong alignment.
        // An aligned dangling pointer would violate the caller lifetime contract.
        b.poles = POLES.as_ptr().cast::<u8>().add(1).cast::<f64>();
        b.poles_len = usize::MAX;
        assert!(!om9_phase2_basis_validate(&b));
        b.poles_len = 9;
        assert!(!om9_phase2_basis_validate(&b));
        b.poles = std::ptr::null();
        assert!(!om9_phase2_basis_validate(&b));
    }
}
#[test]
fn raw_degree_multiplicity_and_xyz_shapes_are_not_truncated() {
    unsafe {
        let bad_mults = [3., 2.5];
        let mut b = input(&POLES, &WEIGHTS, &KNOTS, &MULTS);
        b.degree = 2.5;
        assert!(!om9_phase2_basis_validate(&b));
        b.degree = 2.;
        b.multiplicities = bad_mults.as_ptr();
        assert!(!om9_phase2_basis_validate(&b));
        b.poles_len = 8;
        assert!(!om9_phase2_basis_validate(&b));
    }
}
#[test]
fn ffi_session_copies_buffers_and_dropped_handles_are_not_reused() {
    unsafe {
        let mut p = [0.; 9];
        let w = [1.; 3];
        let k = [0., 1.];
        let m = [3., 3.];
        let signature = b"original";
        let witness = Om9WitnessInput {
            document: 1,
            object: 2,
            generation: 3,
            signature: signature.as_ptr(),
            signature_len: signature.len(),
        };
        let b = input(&p, &w, &k, &m);
        let h = om9_phase2_session_create(&witness, &b);
        assert_ne!(h, 0);
        p[0] = f64::NAN;
        assert!(om9_phase2_session_check(h, &witness));
        assert!(!om9_phase2_session_replace(h, &b));
        assert!(om9_phase2_session_check(h, &witness));
        om9_phase2_session_drop(h);
        assert!(!om9_phase2_session_check(h, &witness));
        p[0] = 0.;
        let b = input(&p, &w, &k, &m);
        let next = om9_phase2_session_create(&witness, &b);
        assert!(next > h);
        om9_phase2_session_drop(next);
    }
}
