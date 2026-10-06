use openmatrix9_rust::core_3dm_archive::*;
#[test]
fn modes_and_capabilities_reject_unknown_values() {
    assert!(om9_3dm_archive_mode_valid(0));
    assert!(om9_3dm_archive_mode_valid(1));
    assert!(!om9_3dm_archive_mode_valid(2));
    for value in 1..=4 {
        assert!(om9_3dm_archive_capability_valid(value));
    }
    assert!(!om9_3dm_archive_capability_valid(0));
    assert!(!om9_3dm_archive_capability_valid(5));
    assert!(om9_3dm_archive_legacy_export_allowed(0));
    assert!(!om9_3dm_archive_legacy_export_allowed(1));
    assert!(!om9_3dm_archive_legacy_export_allowed(u32::MAX));
}
#[test]
fn identities_are_scoped_and_canonical() {
    let source = "12345678-1234-1234-1234-123456789abc";
    let a = ArchiveIdentity::new(source, source).unwrap();
    let b = ArchiveIdentity::new("12345678-1234-1234-1234-123456789abd", source).unwrap();
    assert_ne!(a, b);
    for invalid in [
        "",
        "00000000-0000-0000-0000-000000000000",
        "12345678-1234-1234-1234-123456789ABC",
        "12345678123412341234123456789abc",
        "é",
    ] {
        assert!(ArchiveIdentity::new(invalid, source).is_err());
        assert!(ArchiveIdentity::new(source, invalid).is_err());
    }
}
