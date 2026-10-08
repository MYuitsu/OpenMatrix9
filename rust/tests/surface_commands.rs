use openmatrix9_rust::ffi::*;
use std::ffi::CStr;

#[test]
fn surface_commands_have_document_permissions_and_useful_captions() {
    for (icon, caption) in [
        ("SurfaceSweepSweep1Rail", "Sweep 1 rail"),
        ("SurfaceSweepSweep2Rails", "Sweep 2 rails"),
        ("SurfaceLoft", "Loft"),
    ] {
        let index = (0..om9_command_count())
            .find(|&i| unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == icon.as_bytes())
            .unwrap();
        assert_eq!(om9_command_permissions(index), 1, "{icon} alters the document");
        assert_eq!(unsafe { CStr::from_ptr(om9_command_menu_text(index)) }.to_str().unwrap(), caption);
        // Existing public catalog identifiers remain valid.
        assert_eq!(unsafe { CStr::from_ptr(om9_command_id(index)) }.to_str().unwrap(), format!("OM9_{icon}"));
    }
}
