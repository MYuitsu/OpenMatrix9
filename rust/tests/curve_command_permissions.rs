use openmatrix9_rust::ffi::*;
use std::ffi::CStr;
#[test]
fn om9_curve_003_009_require_document_write_permission() {
    for name in ["InterpCrv", "Rebuild", "Rectangle", "Circle", "Ellipse"] {
        let index = (0..om9_command_count())
            .find(|&i| {
                unsafe { CStr::from_ptr(om9_command_id(i)) }
                    .to_str()
                    .unwrap()
                    == name
            })
            .unwrap();
        assert_eq!(
            om9_command_permissions(index) & 1,
            1,
            "{name} alters the document"
        );
    }
}
