//! Gate proof: a minimal Rust application calls the pinned OCCT 8.0.1.
//!
//! Expected values are analytic, so a wrong kernel result cannot pass.

#[test]
fn the_linked_kernel_is_the_pinned_version() {
    let version = om9_kernel::version();
    assert!(
        version.starts_with("8.0."),
        "expected the pinned OCCT 8.0.x, linked {version}"
    );
}

#[test]
fn a_box_volume_is_exact_from_rust() {
    let volume = om9_kernel::box_volume(10.0, 10.0, 4.0).expect("box");
    assert!((volume - 400.0).abs() < 1e-9, "box volume {volume} != 400.0");
}

#[test]
fn a_boolean_drilled_volume_matches_the_analytic_value_from_rust() {
    let volume = om9_kernel::drilled_volume(10.0, 10.0, 4.0, 2.0).expect("drilled");
    let expected = 400.0 - std::f64::consts::PI * 4.0 * 4.0;
    assert!(
        (volume - expected).abs() < 1e-6,
        "drilled volume {volume} != {expected}"
    );
}

#[test]
fn invalid_input_is_reported_not_fatal() {
    assert!(matches!(
        om9_kernel::box_volume(0.0, 1.0, 1.0),
        Err(om9_kernel::KernelError::InvalidInput(_))
    ));
    let error = om9_kernel::reject_invalid_cylinder(-1.0).expect_err("negative radius");
    let text = error.to_string();
    assert!(
        text.contains("null shape") || text.contains("ConstructionError"),
        "the kernel failure must name the cause, got {text:?}"
    );
}
