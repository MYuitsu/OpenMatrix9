use openmatrix9_rust::{layer_clipboard::*, layer_ffi::*};
#[test]
fn rollback_only_restores_this_publication_and_never_a_newer_owner() {
    for (before, published, current, wanted) in [
        (8, 9, 9, true),
        (8, 9, 10, false),
        (8, 8, 8, false),
        (8, 0, 0, false),
        (u32::MAX, 1, 1, true),
        (0, 1, 1, true),
    ] {
        assert_eq!(
            rollback_owned_publication(before, published, current),
            wanted
        );
        assert_eq!(
            om9_layer_clipboard_rollback_owned(before, published, current),
            u8::from(wanted)
        );
    }
}
#[test]
fn backup_budget_checks_cumulative_bytes_and_format_count() {
    assert_eq!(check_clipboard_backup(512 * 1024 * 1024, 256), Ok(()));
    assert!(check_clipboard_backup(512 * 1024 * 1024 + 1, 1).is_err());
    assert!(check_clipboard_backup(0, 257).is_err());
    assert_eq!(
        om9_layer_clipboard_check_backup(512 * 1024 * 1024 + 1, 1),
        8
    );
    assert_eq!(om9_layer_clipboard_check_backup(0, 257), 8);
}
