use openmatrix9_rust::ffi::*;
use std::ffi::CStr;

#[test]
fn solid_creation_requires_write_permission_and_preserves_catalog_ids() {
    for (icon, caption) in [
        ("SolidBoxCornertoCornerHeight", "Box"),
        ("SolidSphereCenterRadius", "Sphere"),
    ] {
        let i = (0..om9_command_count())
            .find(|&i| unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == icon.as_bytes())
            .unwrap();
        assert_eq!(
            om9_command_permissions(i),
            1,
            "{caption} creates document geometry"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(om9_command_menu_text(i)) }
                .to_str()
                .unwrap(),
            caption
        );
        assert_eq!(
            unsafe { CStr::from_ptr(om9_command_id(i)) }
                .to_str()
                .unwrap(),
            format!("OM9_{icon}")
        );
    }
}
