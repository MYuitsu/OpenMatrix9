use openmatrix9_rust::tolerances::{Error, Operation, Source, resolve};
use openmatrix9_rust::units::Context;

fn context() -> Context {
    Context {
        revision: 7,
        model_mm: 25.4,
        page_mm: 1.0,
        absolute_mm: 0.01,
        relative_ratio: 0.00001,
        angular_rad: 0.02,
    }
}

#[test]
fn precedence_and_surface_rule_are_explicit() {
    let c = context();
    let legacy = resolve(Operation::JoinCurves, None, None, 100.0).unwrap();
    assert_eq!(legacy.effective_mm, 1e-7);
    assert_eq!(legacy.source, Source::Legacy);
    assert_eq!(legacy.context_revision, 0);
    assert_eq!(
        resolve(Operation::JoinSurfaces, None, None, 100.0)
            .unwrap()
            .effective_mm,
        1e-6
    );
    let curve = resolve(Operation::JoinCurves, Some(&c), None, 100.0).unwrap();
    assert_eq!(curve.effective_mm, 0.01);
    assert_eq!(curve.source, Source::Document);
    assert_eq!(curve.context_revision, 7);
    let surface = resolve(Operation::JoinSurfaces, Some(&c), None, 100.0).unwrap();
    assert_eq!(surface.requested_mm, 0.01);
    assert_eq!(surface.effective_mm, 0.02);
    let explicit = resolve(Operation::JoinSurfaces, Some(&c), Some(0.003), 100.0).unwrap();
    assert_eq!(explicit.source, Source::Override);
    assert_eq!(explicit.effective_mm, 0.003);
    assert_eq!(explicit.used_fields, 1); // Only absolute length; no invented relative rule.
    let mut other = c.clone();
    other.relative_ratio = 0.7;
    other.angular_rad = 1.2;
    assert_eq!(
        resolve(Operation::JoinCurves, Some(&other), None, 100.0).unwrap(),
        curve
    );
}

#[test]
fn numerical_resolution_rejects_without_relaxing_requested_tolerance() {
    let c = context();
    assert_eq!(
        resolve(Operation::JoinCurves, Some(&c), Some(1e-3), 1e12),
        Err(Error::BelowResolution)
    );
    assert!(resolve(Operation::JoinCurves, Some(&c), Some(0.01), 1e12).is_ok());
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        assert_eq!(
            resolve(Operation::JoinCurves, Some(&c), None, bad),
            Err(Error::InvalidBound)
        );
    }
    for bad in [f64::NAN, f64::INFINITY, -1.0, 0.0, 1e-12, 1e7] {
        assert_eq!(
            resolve(Operation::JoinCurves, Some(&c), Some(bad), 1.0),
            Err(Error::InvalidTolerance)
        );
    }
    let mut large = c.clone();
    large.absolute_mm = 600_000.0;
    assert!(resolve(Operation::JoinCurves, Some(&large), None, 1.0).is_ok());
    assert_eq!(
        resolve(Operation::JoinSurfaces, Some(&large), None, 1.0),
        Err(Error::InvalidTolerance)
    );
    large.absolute_mm = f64::NAN;
    assert_eq!(
        resolve(Operation::JoinCurves, Some(&large), Some(0.01), 1.0),
        Err(Error::InvalidContext)
    );
}

#[test]
fn report_preserves_actual_rule_and_does_not_depend_on_display_precision() {
    let r = resolve(Operation::JoinSurfaces, Some(&context()), None, 10.0).unwrap();
    let report = r.report();
    assert!(report.contains("\"operation\":\"join_surfaces\""));
    assert!(report.contains("\"requested_mm\":1e-2"));
    assert!(report.contains("\"effective_mm\":2e-2"));
    assert!(report.contains("\"relative_used\":false"));
    assert!(report.contains("\"angular_used\":false"));
    assert_eq!(report, r.report());
}

#[test]
fn abi_fails_closed_and_leaves_output_untouched() {
    use openmatrix9_rust::tolerances::{om9_tolerance_report, om9_tolerance_resolve};
    use std::ffi::CString;
    let c = CString::new(context().encode().unwrap()).unwrap();
    let before = resolve(Operation::JoinCurves, None, None, 1.0).unwrap();
    let mut out = before;
    unsafe {
        assert_eq!(
            om9_tolerance_resolve(c.as_ptr(), 99, 0.0, false, 1.0, &mut out),
            Error::UnsupportedOperation as u32
        );
        assert_eq!(out, before);
        assert_eq!(
            om9_tolerance_resolve(c.as_ptr(), 1, 1e-3, true, 1e12, &mut out),
            Error::BelowResolution as u32
        );
        assert_eq!(out, before);
        let future = CString::new("2|1|1|1|0.01|0.01|0.01").unwrap();
        assert_eq!(
            om9_tolerance_resolve(future.as_ptr(), 1, 0.1, true, 1.0, &mut out),
            Error::InvalidContext as u32
        );
        assert_eq!(out, before);
        assert_eq!(
            om9_tolerance_resolve(c.as_ptr(), 1, 0.0, false, 1.0, std::ptr::null_mut()),
            Error::InvalidPointer as u32
        );
        assert_eq!(
            om9_tolerance_resolve(c.as_ptr(), 2, 0.0, false, 1.0, &mut out),
            0
        );
        assert_eq!(
            out,
            resolve(Operation::JoinSurfaces, Some(&context()), None, 1.0).unwrap()
        );
        let count = om9_tolerance_report(&out, std::ptr::null_mut(), 0);
        assert_eq!(count, out.report().len() + 1);
        let mut buffer = vec![77_u8; count];
        assert_eq!(
            om9_tolerance_report(&out, buffer.as_mut_ptr().cast(), count - 1),
            count
        );
        assert!(buffer.iter().all(|b| *b == 77));
        assert_eq!(
            om9_tolerance_report(&out, buffer.as_mut_ptr().cast(), count),
            count
        );
        assert_eq!(buffer[count - 1], 0);
        assert_eq!(&buffer[..count - 1], out.report().as_bytes());
        assert_eq!(
            om9_tolerance_report(std::ptr::null(), buffer.as_mut_ptr().cast(), count),
            0
        );
    }
}
