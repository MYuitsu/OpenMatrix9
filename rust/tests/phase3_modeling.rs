use openmatrix9_rust::phase3_modeling::{
    InputFacts, Kind, Operation, Reason, Request, Snapshot, capability,
};

#[test]
fn empty_kernel_output_is_never_committable() {
    use openmatrix9_rust::phase3_modeling::validate_result_count;
    assert_eq!(validate_result_count(0), Err(Reason::InvalidTopology));
    for count in [1, 2, 4096, usize::MAX] {
        assert_eq!(validate_result_count(count), Ok(()));
    }
}

fn facts(kind: Kind) -> InputFacts {
    InputFacts {
        kind,
        valid: true,
        closed: kind == Kind::Solid,
        protected: false,
        components: 2,
    }
}
fn snapshot(name: &str, signature: &str) -> Snapshot {
    Snapshot {
        identity: name.into(),
        signature: signature.into(),
    }
}
#[test]
fn closed_solid_boolean_and_open_shell_rejection() {
    for op in [
        Operation::Difference,
        Operation::Union,
        Operation::Intersection,
        Operation::TwoObjects,
    ] {
        assert_eq!(
            capability(op, &[facts(Kind::Solid), facts(Kind::Solid)]),
            Ok(())
        );
        assert_eq!(
            capability(op, &[facts(Kind::Solid), facts(Kind::Shell)]),
            Err(Reason::ClosedSolidRequired)
        );
        let mut open = facts(Kind::Solid);
        open.closed = false;
        assert_eq!(
            capability(op, &[open, facts(Kind::Solid)]),
            Err(Reason::ClosedSolidRequired)
        );
    }
}
#[test]
fn curve_and_face_join_have_separate_type_policies() {
    assert_eq!(
        capability(Operation::Join, &[facts(Kind::Curve), facts(Kind::Curve)]),
        Ok(())
    );
    assert_eq!(
        capability(Operation::Join, &[facts(Kind::Face), facts(Kind::Shell)]),
        Ok(())
    );
    assert_eq!(
        capability(Operation::Join, &[facts(Kind::Curve), facts(Kind::Face)]),
        Err(Reason::MixedInputs)
    );
    assert_eq!(
        capability(Operation::Join, &[facts(Kind::Solid), facts(Kind::Solid)]),
        Err(Reason::Unsupported)
    );
}
#[test]
fn surface_input_order_cardinality_and_heavy_types() {
    assert_eq!(
        capability(Operation::Loft, &[facts(Kind::Curve), facts(Kind::Curve)]),
        Ok(())
    );
    assert_eq!(
        capability(Operation::Sweep1, &[facts(Kind::Curve), facts(Kind::Curve)]),
        Ok(())
    );
    assert_eq!(
        capability(Operation::Sweep2, &[facts(Kind::Curve), facts(Kind::Curve)]),
        Err(Reason::InputCount)
    );
    assert_eq!(
        capability(Operation::Sweep2, &[facts(Kind::Curve); 3]),
        Ok(())
    );
    for kind in [
        Kind::Mesh,
        Kind::Cloud,
        Kind::Retained,
        Kind::Point,
        Kind::Unknown,
    ] {
        assert_eq!(
            capability(Operation::Loft, &[facts(kind), facts(kind)]),
            Err(Reason::Unsupported)
        );
    }
}
#[test]
fn invalid_or_preservation_protected_inputs_fail_before_kernel() {
    let mut bad = facts(Kind::Solid);
    bad.valid = false;
    assert_eq!(
        capability(Operation::Difference, &[bad, facts(Kind::Solid)]),
        Err(Reason::InvalidTopology)
    );
    bad.valid = true;
    bad.protected = true;
    assert_eq!(
        capability(Operation::Difference, &[bad, facts(Kind::Solid)]),
        Err(Reason::Protected)
    );
}
#[test]
fn trim_faces_explode_components_and_numeric_validation() {
    assert_eq!(
        capability(Operation::Trim, &[facts(Kind::Face), facts(Kind::Face)]),
        Ok(())
    );
    assert_eq!(
        capability(Operation::Explode, &[facts(Kind::Solid)]),
        Ok(())
    );
    let mut one = facts(Kind::Shell);
    one.components = 1;
    assert_eq!(
        capability(Operation::Explode, &[one]),
        Err(Reason::InputCount)
    );
    assert_eq!(
        capability(Operation::Difference, &[]),
        Err(Reason::InputCount)
    );
}
#[test]
fn request_owns_snapshots_and_rejects_changed_placement_shape_or_identity() {
    let mut initial = vec![snapshot("profile.Edge1", "shape-and-world-placement")];
    let request = Request::new("Doc", initial.clone()).unwrap();
    initial[0].signature = "changed".into();
    assert_eq!(
        request.validate_current(
            "Doc",
            &[snapshot("profile.Edge1", "shape-and-world-placement")],
            true
        ),
        Ok(())
    );
    assert_eq!(
        request.validate_current("Doc", &initial, true),
        Err(Reason::Stale)
    );
    assert_eq!(
        request.validate_current(
            "Doc",
            &[snapshot("profile.Edge2", "shape-and-world-placement")],
            true
        ),
        Err(Reason::Stale)
    );
    assert_eq!(
        request.validate_current("OtherDoc", &[], true),
        Err(Reason::Stale)
    );
    assert_eq!(
        request.validate_current("Doc", &[], true),
        Err(Reason::Stale)
    );
    assert_eq!(
        request.validate_current("Doc", &initial, false),
        Err(Reason::Unavailable)
    );
}
#[test]
fn cancelled_request_never_becomes_committable_again() {
    let values = vec![snapshot("A", "digest")];
    let mut request = Request::new("Doc", values.clone()).unwrap();
    request.cancel();
    assert_eq!(
        request.validate_current("Doc", &values, true),
        Err(Reason::Cancelled)
    );
}
#[test]
fn requests_reject_duplicate_empty_or_oversized_identity() {
    assert!(Request::new("Doc", vec![snapshot("A", "1"), snapshot("A", "2")]).is_err());
    assert!(Request::new("Doc", vec![]).is_err());
    assert!(Request::new("", vec![snapshot("A", "1")]).is_err());
    assert!(Request::new("Doc", vec![snapshot(&"A".repeat(4097), "1")]).is_err());
}
#[test]
fn numeric_surface_options_have_rust_policy() {
    use openmatrix9_rust::phase3_modeling::surface_options;
    assert!(surface_options(3, 0, false, 0, 16, 0.01, 2).is_ok());
    assert!(surface_options(3, 6, false, 0, 16, 0.01, 2).is_err());
    assert!(surface_options(3, 5, true, 0, 16, 0.01, 3).is_err());
    assert!(surface_options(2, 0, false, 2, 256, 0.01, 3).is_ok());
    for count in [0, 1, 257] {
        assert!(surface_options(1, 0, false, 1, count, 0.01, 2).is_err());
    }
    for tolerance in [f64::NAN, f64::INFINITY, 0., -1., 1e7] {
        assert!(surface_options(1, 0, false, 2, 16, tolerance, 2).is_err());
    }
    assert!(surface_options(1, 0, true, 0, 16, 0.01, 2).is_err());
}
#[test]
fn join_routes_by_native_kind_in_rust_without_geometry_arrays() {
    use openmatrix9_rust::phase3_modeling::{Kind, join_uses_surface_adapter};
    assert!(!join_uses_surface_adapter(Kind::Curve));
    for kind in [
        Kind::Face,
        Kind::Shell,
        Kind::Solid,
        Kind::Mesh,
        Kind::Cloud,
        Kind::Unknown,
        Kind::Retained,
    ] {
        assert!(join_uses_surface_adapter(kind));
    }
}
