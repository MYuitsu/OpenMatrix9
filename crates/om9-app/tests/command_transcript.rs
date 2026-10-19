// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::transcript::Transcript;
#[test]
fn one_surface_keeps_history_and_prompt_protected_with_unicode_char_indices() {
    let mut t = Transcript::new();
    t.append_history("Kết quả: ✓");
    t.set_prompt("Command: ");
    t.set_draft("Line");
    let protected = t.protected_chars();
    let original = t.text().to_owned();
    assert_eq!(t.insert("corrupt", 0), 0);
    t.delete(0..protected);
    assert_eq!(t.text(), original);
    t.delete(0..protected + 2);
    assert_eq!(t.draft(), "ne");
    assert!(t.text().starts_with("Kết quả: ✓\nCommand: "));
    assert_eq!(t.insert("界", protected), 1);
    assert_eq!(t.draft(), "界ne");
}
#[test]
fn paste_only_changes_draft_and_submit_is_an_explicit_separate_action() {
    let mut t = Transcript::new();
    t.set_draft("Line");
    let boundary = t.protected_chars();
    t.insert("\n0,0,0\n10,0,0", boundary + 4);
    assert_eq!(t.draft(), "Line 0,0,0 10,0,0");
    assert!(t.history().is_empty());
}
#[test]
fn recall_down_restores_pending_draft_and_history_is_bounded_without_utf8_damage() {
    let mut t = Transcript::new();
    t.record_command("Line");
    t.record_command("Point");
    t.set_draft("unfinished");
    t.recall_up();
    assert_eq!(t.draft(), "Point");
    t.recall_up();
    assert_eq!(t.draft(), "Line");
    t.recall_down();
    assert_eq!(t.draft(), "Point");
    t.recall_down();
    assert_eq!(t.draft(), "unfinished");
    for _ in 0..2000 {
        t.append_history("kết quả hoàn tất");
    }
    assert!(t.text().len() <= Transcript::MAX_BYTES);
    assert!(t.text().ends_with("Command: unfinished"));
}
