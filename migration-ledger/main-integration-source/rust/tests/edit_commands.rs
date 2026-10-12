use openmatrix9_rust::ffi::*;
use std::ffi::CStr;
#[test]
fn edit_commands_require_document_write_permission() {
    for (icon, caption) in [
        ("TopIconJoin", "Join"),
        ("TopIconExplode", "Explode"),
        ("TopIconTrim", "Trim"),
        ("SolidUnion", "BooleanUnion"),
        ("SolidDifference", "BooleanDifference"),
        ("SolidIntersection", "BooleanIntersection"),
        ("SolidBooleanTwoObjects", "Boolean2Objects"),
    ] {
        let i = (0..om9_command_count())
            .find(|i| {
                unsafe { CStr::from_ptr(om9_command_icon(*i)) }
                    .to_str()
                    .unwrap()
                    == icon
            })
            .unwrap();
        assert_eq!(
            om9_command_permissions(i),
            1,
            "{icon} must enforce write permission"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(om9_command_menu_text(i)) }
                .to_str()
                .unwrap(),
            caption
        );
    }
}
