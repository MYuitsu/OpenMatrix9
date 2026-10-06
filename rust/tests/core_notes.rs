use openmatrix9_rust::core_notes::{Edit, decide_edit};

#[test]
fn om9_file_008_and_info_006_preserve_unicode_and_multiline_text() {
    assert_eq!(
        decide_edit("", "Ghi chú nhẫn\nSize 12 — 金"),
        Ok(Edit::Changed)
    );
    assert_eq!(decide_edit("Ghi chú\n", "Ghi chú\n"), Ok(Edit::Unchanged));
    assert_eq!(decide_edit("old notes", ""), Ok(Edit::Changed));
}
#[test]
fn invalid_notes_are_rejected_before_commit() {
    assert!(decide_edit("saved", "a\0b").is_err());
    assert!(decide_edit("saved", &"x".repeat(1_048_577)).is_err());
    assert_eq!(
        decide_edit("saved", &"x".repeat(1_048_576)),
        Ok(Edit::Changed)
    );
}
