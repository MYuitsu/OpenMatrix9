use openmatrix9_rust::nurbs_basis_ffi::*;
fn limits() -> Limits {
    Limits {
        max_degree: 25,
        max_poles: 1_000_000,
        max_knots: 1_000_000,
        max_bytes: 64 * 1024 * 1024,
        max_coordinate: 1e150,
    }
}
fn axis() -> NativeAxis {
    NativeAxis {
        degree: 1,
        knots: [0., 1.].as_ptr(),
        knot_count: 2,
        multiplicities: [2, 2].as_ptr(),
        multiplicity_count: 2,
        periodic: 0,
        domain: [0., 1.],
    }
}
fn curve(p: &[[f64; 3]]) -> NativeCurve {
    NativeCurve {
        poles: p.as_ptr(),
        pole_count: p.len(),
        weights: std::ptr::null(),
        weight_count: 0,
        axis: axis(),
    }
}
#[test]
fn valid_curve() {
    let p = [[0., 0., 0.], [1., 0., 0.]];
    assert_eq!(unsafe { om9_basis_curve_validate(curve(&p), limits()) }, 0);
}
#[test]
fn count_preflight_never_reads_buffers() {
    assert_eq!(om9_basis_curve_counts(1, 2, 0, 2, limits()), 0);
    assert_ne!(om9_basis_curve_counts(1, usize::MAX, 0, 2, limits()), 0);
}
#[test]
fn invalid_degree_rejects_before_pointer_read() {
    let mut c = curve(&[]);
    c.poles = std::ptr::null();
    c.pole_count = 2;
    c.axis.degree = 0;
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
}
#[test]
fn null_nonempty_rejected() {
    let p = [[0., 0., 0.], [1., 0., 0.]];
    let mut c = curve(&p);
    c.poles = std::ptr::null();
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
}
#[test]
fn misaligned_rejected() {
    let p = [[0., 0., 0.], [1., 0., 0.]];
    let mut c = curve(&p);
    c.poles = std::ptr::without_provenance(1);
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
}
#[test]
fn address_overflow_rejected() {
    let p = [[0., 0., 0.], [1., 0., 0.]];
    let mut c = curve(&p);
    c.poles = (usize::MAX - 7) as *const [f64; 3];
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
}
#[test]
fn invalid_periodic_flag_rejected() {
    let p = [[0., 0., 0.], [1., 0., 0.]];
    let mut c = curve(&p);
    c.axis.periodic = 2;
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
}
#[test]
fn malformed_basis_rejected() {
    let p = [[0., 0., f64::NAN], [1., 0., 0.]];
    assert_ne!(unsafe { om9_basis_curve_validate(curve(&p), limits()) }, 0);
}
#[test]
fn weight_count_and_null_rejected() {
    let p = [[0., 0., 0.], [1., 0., 0.]];
    let mut c = curve(&p);
    c.weight_count = 2;
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
    c.weight_count = 1;
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
}
#[test]
fn truncation_and_invalid_multiplicity_rejected() {
    let p = [[0., 0., 0.], [1., 0., 0.]];
    let mut c = curve(&p);
    c.axis.multiplicity_count = 1;
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
    c.axis = axis();
    c.axis.multiplicities = [1, 1].as_ptr();
    assert_ne!(unsafe { om9_basis_curve_validate(c, limits()) }, 0);
}
#[test]
fn valid_surface() {
    let p = [[0., 0., 0.], [0., 1., 0.], [1., 0., 0.], [1., 1., 0.]];
    let s = NativeSurface {
        poles: p.as_ptr(),
        pole_count: 4,
        weights: std::ptr::null(),
        weight_count: 0,
        u_count: 2,
        v_count: 2,
        axes: [axis(); 2],
    };
    assert_eq!(unsafe { om9_basis_surface_validate(s, limits()) }, 0);
}
#[test]
fn surface_product_and_payload_limit() {
    let c = SurfaceCounts {
        degrees: [1, 1],
        u_count: 2,
        v_count: 2,
        pole_count: 4,
        weight_count: 0,
        knot_counts: [2, 2],
    };
    assert_eq!(om9_basis_surface_counts(c, limits()), 0);
    assert_ne!(
        om9_basis_surface_counts(
            SurfaceCounts {
                u_count: usize::MAX,
                ..c
            },
            limits()
        ),
        0
    );
    let mut l = limits();
    l.max_bytes = 1;
    assert_ne!(om9_basis_surface_counts(c, l), 0);
}
