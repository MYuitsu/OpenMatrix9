use openmatrix9_rust::core_3dm_archive::validate_export_manifest;

const MESH: &str = r#"{"items":[{"name":"mesh","layer":"Default","visible":true,"locked":false,"color":[128,0,255],"vertices":[[0,0,0],[1,0,0],[0,1,0]],"faces":[[0,1,2,2]]}],"tolerance":0.000001}"#;

#[test]
fn export_manifest_preserves_exact_mesh_and_brep_schema() {
    assert!(validate_export_manifest(MESH).is_ok());
    assert!(validate_export_manifest(r#"{"items":[{"brep":"C:/staged/shape.brep","color":[0,1,255]}],"tolerance":0.001}"#).is_ok());
}

#[test]
fn malformed_mesh_tuples_cannot_be_coerced_to_zero_or_truncated() {
    for (from, to) in [
        ("[0,0,0]", "[0,0]"), ("[0,0,0]", "[0,0,0,7]"),
        ("[0,0,0]", "[0,\"1\",0]"), ("[0,1,2,2]", "[0,1,2]"),
        ("[0,1,2,2]", "[0,1,3,3]"), ("[0,1,2,2]", "[0,1,2.1,2]"),
        ("[0,1,2,2]", "[0,-1,2,2]"), ("[128,0,255]", "[128,0]"),
        ("[128,0,255]", "[128,0,256]"), ("true", "null"),
    ] {
        assert!(validate_export_manifest(&MESH.replace(from, to)).is_err(), "{to}");
    }
}

#[test]
fn invalid_schema_or_retained_data_is_rejected_before_native_conversion() {
    for raw in [
        "{}", r#"{"items":[],"tolerance":1}"#,
        r#"{"items":[{"brep":"","color":[0,0,0]}],"tolerance":1}"#,
        r#"{"items":[{"brep":"x\u0000y","color":[0,0,0]}],"tolerance":1}"#,
    ] {assert!(validate_export_manifest(raw).is_err(), "{raw}");}
    for (from, to) in [
        ("0.000001", "0"), ("0.000001", "\"0.1\""),
        ("\"name\":\"mesh\"", "\"retained\":true"),
        ("\"name\":\"mesh\"", "\"brep\":\"shape.brep\""),
        ("\"name\":\"mesh\"", "\"name\":\"mesh\",\"name\":\"duplicate\""),
    ] {assert!(validate_export_manifest(&MESH.replace(from, to)).is_err(), "{to}");}
}

#[test]
fn export_manifest_ffi_rejects_bad_utf8_and_oversized_lengths_before_borrowing() {
    use openmatrix9_rust::core_3dm_archive::om9_3dm_export_manifest_valid;
    unsafe {
        assert!(om9_3dm_export_manifest_valid(MESH.as_ptr(), MESH.len()));
        assert!(!om9_3dm_export_manifest_valid(std::ptr::null(), 1));
        assert!(!om9_3dm_export_manifest_valid(MESH.as_ptr(), usize::MAX));
        assert!(!om9_3dm_export_manifest_valid([0xff].as_ptr(), 1));
    }
}
