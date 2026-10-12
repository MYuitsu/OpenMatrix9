use openmatrix9_rust::ffi;
use std::ffi::CStr;

fn command(name: &str) -> usize {
    (0..ffi::om9_command_count())
        .find(|&index| unsafe { CStr::from_ptr(ffi::om9_command_id(index)) }.to_bytes() == name.as_bytes())
        .unwrap_or_else(|| panic!("native command missing: {name}"))
}

#[test]
fn points_on_menu_exposes_native_editor_dispatch_with_document_permission() {
    assert_eq!(ffi::om9_command_permissions(command("PointsOn")), 1);
}

#[test]
fn clipboard_paste_requires_document_mutation_and_copy_keeps_read_permission() {
    assert_eq!(ffi::om9_command_permissions(command("Paste3dm")), 1);
    for name in ["Copy3dm", "CopySession3dm", "ExportSession3dm", "ExportLayerSelection3dm"] {
        assert_eq!(ffi::om9_command_permissions(command(name)), 0);
    }
}
