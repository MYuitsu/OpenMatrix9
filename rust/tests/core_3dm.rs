// Spec: OM9-FILE-012 — Rhino 5 archive exchange.
use openmatrix9_rust::{command_completion, core_3dm, ffi};
use std::ffi::CStr;

#[test]
fn units_never_guess_or_accept_invalid_scale() {
    assert_eq!(core_3dm::scale(25.4, 0.0), 25.4);
    assert_eq!(core_3dm::scale(1.0, 0.0), 1.0);
    assert_eq!(core_3dm::scale(0.0, 2.0), 2.0);
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(core_3dm::scale(value, 0.0), 0.0);
    }
    assert_eq!(core_3dm::scale(0.0, f64::NAN), 0.0);
}

#[test]
fn import_export_share_catalog_policy_and_completion() {
    for (name, operation, permissions) in [("Import3dm", 1, 1), ("Export3dm", 2, 0)] {
        let index = (0..ffi::om9_command_count())
            .find(|i| unsafe {
                CStr::from_ptr(ffi::om9_command_id(*i)).to_bytes() == name.as_bytes()
            })
            .expect("exchange command in catalog");
        assert_eq!(core_3dm::om9_3dm_operation(index), operation);
        assert_eq!(ffi::om9_command_permissions(index), permissions);
        assert!(command_completion::names().contains(&name));
    }
    assert_eq!(core_3dm::om9_3dm_operation(usize::MAX), 0);
    assert!(core_3dm::om9_3dm_type_supported(1));
    assert!(!core_3dm::om9_3dm_type_supported(99));
}
