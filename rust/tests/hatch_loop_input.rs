use openmatrix9_rust::hatch_loop_input::{MAX_JSON_BYTES, om9_hatch_loop_input, preflight_loops};
fn nurbs(rational: bool, cvs: &str, knots: &str, domain: &str) -> String {
    format!(
        r#"{{"class_name":"ON_NurbsCurve","dimension":2,"source_index":null,"domain":{domain},"order":2,"rational":{rational},"cvs":{cvs},"knots":{knots}}}"#
    )
}
fn payload(curve: &str) -> String {
    format!(r#"{{"schema_version":1,"loops":[{{"type":0,"source_index":null,"curve":{curve}}}]}}"#)
}
const CVS: &str = "[[0,0],[10,0],[10,10],[0,10],[0,0]]";
const RCVS: &str = "[[0,0,1],[20,0,2],[10,10,1],[0,10,1],[0,0,1]]";
const KNOTS: &str = "[0,1,2,3,4]";
fn valid() -> String {
    payload(&nurbs(false, CVS, KNOTS, "[0,4]"))
}
#[test]
fn nonrational_closed_basis_is_admitted() {
    assert!(preflight_loops(&valid()).is_ok());
}
#[test]
fn rational_homogeneous_basis_is_admitted() {
    assert!(preflight_loops(&payload(&nurbs(true, RCVS, KNOTS, "[0,4]"))).is_ok());
}
#[test]
fn true_requires_weight_row() {
    assert!(preflight_loops(&payload(&nurbs(true, CVS, KNOTS, "[0,4]"))).is_err());
}
#[test]
fn false_rejects_hidden_weight_row() {
    assert!(preflight_loops(&payload(&nurbs(false, RCVS, KNOTS, "[0,4]"))).is_err());
}
#[test]
fn malformed_cv_rejects() {
    assert!(preflight_loops(&valid().replace("[10,10]", "[null,10]")).is_err());
}
#[test]
fn zero_and_negative_weights_reject() {
    for w in ["0", "-1"] {
        assert!(
            preflight_loops(&payload(&nurbs(
                true,
                &RCVS.replace("[10,10,1]", &format!("[10,10,{w}]")),
                KNOTS,
                "[0,4]"
            )))
            .is_err()
        );
    }
}
#[test]
fn finite_tiny_weight_cannot_overflow_dehomogeneous_poles() {
    let cvs = RCVS.replace("[10,10,1]", "[10,10,1e-320]");
    assert!(preflight_loops(&payload(&nurbs(true, &cvs, KNOTS, "[0,4]"))).is_err());
}
#[test]
fn raw_knots_have_native_cardinality() {
    assert!(preflight_loops(&payload(&nurbs(false, CVS, "[0,1,2,3]", "[0,4]"))).is_err());
}
#[test]
fn unordered_and_degenerate_knots_reject() {
    for k in ["[0,1,-1,3,4]", "[0,0,0,0,0]"] {
        assert!(preflight_loops(&payload(&nurbs(false, CVS, k, "[0,4]"))).is_err());
    }
}
#[test]
fn domain_matches_native_active_interval_exactly() {
    for d in ["[1,4]", "[0,5]", "[4,0]"] {
        assert!(preflight_loops(&payload(&nurbs(false, CVS, KNOTS, d))).is_err());
    }
}
#[test]
fn excess_multiplicity_rejects() {
    assert!(preflight_loops(&payload(&nurbs(false, CVS, "[0,0,0,1,1]", "[0,1]"))).is_err());
}
#[test]
fn degree_and_dimensions_reject() {
    for bad in [
        valid().replace("\"order\":2", "\"order\":27"),
        valid().replace("\"dimension\":2", "\"dimension\":3"),
    ] {
        assert!(preflight_loops(&bad).is_err());
    }
}
#[test]
fn nonfinite_json_and_oversize_input_reject() {
    assert!(preflight_loops(&valid().replace("[10,10]", "[1e309,10]")).is_err());
    assert!(preflight_loops(&" ".repeat(MAX_JSON_BYTES + 1)).is_err());
}
#[test]
fn later_nested_invalid_nurbs_rejects_whole_input() {
    let good = nurbs(false, CVS, KNOTS, "[0,4]");
    let bad = nurbs(false, CVS, "[0,1,2,3]", "[0,4]");
    let poly = format!(r#"{{"class_name":"ON_PolyCurve","segments":[{good},{bad}]}}"#);
    assert!(preflight_loops(&payload(&poly)).is_err());
}
#[test]
fn parse_schema_and_depth_limits_reject() {
    for s in ["{}", "null", "{\"schema_version\":2,\"loops\":[]}"] {
        assert!(preflight_loops(s).is_err());
    }
    let s = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    assert!(preflight_loops(&s).is_err());
}
#[test]
fn native_supported_curve_depth_retains_json_headroom() {
    let wrap = |c: String| {
        format!(
            r#"{{"class_name":"ON_PolyCurve","dimension":2,"source_index":null,"domain":[0,4],"parameters":[0,4],"segments":[{c}]}}"#
        )
    };
    let mut c = nurbs(false, CVS, KNOTS, "[0,4]");
    for _ in 0..63 {
        c = wrap(c);
    }
    assert!(preflight_loops(&payload(&c)).is_ok());
    c = wrap(c);
    assert!(preflight_loops(&payload(&c)).is_err());
}
#[test]
fn native_byte_abi_rejects_before_invalid_slice() {
    unsafe {
        assert_eq!(om9_hatch_loop_input(std::ptr::null(), 1), 1);
        assert_eq!(om9_hatch_loop_input(1usize as *const u8, usize::MAX), 1);
        assert_eq!(om9_hatch_loop_input(b"".as_ptr(), 0), 1);
        assert_eq!(om9_hatch_loop_input([255u8].as_ptr(), 1), 1);
    }
}
#[test]
fn native_byte_abi_returns_basis_error_and_success() {
    let good = valid();
    let bad = good.replace("[10,10]", "[null,10]");
    unsafe {
        assert_eq!(om9_hatch_loop_input(good.as_ptr(), good.len()), 0);
        assert_ne!(om9_hatch_loop_input(bad.as_ptr(), bad.len()), 0);
    }
}
