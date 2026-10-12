use openmatrix9_rust::nurbs_basis_ffi::*;
#[test]
fn curve_payload_is_exact_and_uses_shared_limits() {
    let p = om9_basis_native_limits();
    let result = om9_basis_curve_payload(2, 3, 3, 2, p);
    assert_eq!((result.status, result.reserved, result.bytes), (0, 0, 128));
    let result = om9_basis_curve_payload(2, 3, 0, 2, p);
    assert_eq!((result.status, result.bytes), (0, 104));
    for args in [
        (26, 3, 0, 2),
        (2, usize::MAX, 0, 2),
        (2, 3, 2, 2),
        (2, 3, 0, usize::MAX),
    ] {
        let r = om9_basis_curve_payload(args.0, args.1, args.2, args.3, p);
        assert_ne!(r.status, 0);
        assert_eq!((r.reserved, r.bytes), (0, 0));
    }
}
#[test]
fn surface_payload_product_and_grid_match_precede_accounting() {
    let p = om9_basis_native_limits();
    let c = SurfaceCounts {
        degrees: [1, 1],
        u_count: 2,
        v_count: 2,
        pole_count: 4,
        weight_count: 4,
        knot_counts: [2, 2],
    };
    let result = om9_basis_surface_payload(c, p);
    assert_eq!((result.status, result.reserved, result.bytes), (0, 0, 192));
    for mode in 0..4 {
        let mut c = c;
        match mode {
            0 => c.u_count = usize::MAX,
            1 => c.pole_count = 3,
            2 => c.weight_count = 3,
            _ => c.degrees[1] = 26,
        };
        let r = om9_basis_surface_payload(c, p);
        assert_ne!(r.status, 0);
        assert_eq!((r.reserved, r.bytes), (0, 0));
    }
}
#[test]
fn payload_summary_abi_has_no_pointer_or_owner() {
    assert_eq!(std::mem::size_of::<CountSummary>(), 16);
    assert_eq!(std::mem::align_of::<CountSummary>(), 8);
    assert_eq!(std::mem::offset_of!(CountSummary, bytes), 8);
}
