use openmatrix9_rust::ffi::{
    om9_command_count, om9_command_icon, om9_command_repeat_candidate, om9_command_repeatable,
    om9_sidebar_history_command, om9_sidebar_record_execution,
};
use std::ffi::CStr;

fn command(name: &str) -> usize {
    (0..om9_command_count())
        .find(|&i| unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == name.as_bytes())
        .unwrap_or_else(|| panic!("Missing test command {name}"))
}

#[test]
fn repeat_keeps_successful_supported_command_separate_from_transcript_and_nonrepeatable_success() {
    let line = command("CurveLineSingleLine");
    let circle = command("CurveCircleCenterRadius");
    let undo = command("Undo");
    assert_eq!(om9_command_repeat_candidate(), usize::MAX);
    om9_sidebar_record_execution(line, false);
    assert_eq!(om9_command_repeat_candidate(), usize::MAX);
    om9_sidebar_record_execution(line, true);
    assert_eq!(om9_command_repeat_candidate(), line);
    om9_sidebar_record_execution(circle, false);
    assert_eq!(om9_command_repeat_candidate(), line);
    om9_sidebar_record_execution(undo, true);
    assert_eq!(om9_sidebar_history_command(0), undo);
    assert_eq!(om9_command_repeat_candidate(), line);
    for name in ["Undo", "Redo", "Delete", "ToolsObjectSnapEnd"] {
        assert!(!om9_command_repeatable(command(name)), "{name}");
    }
    om9_sidebar_record_execution(circle, true);
    assert_eq!(om9_command_repeat_candidate(), circle);
    assert!(!om9_command_repeatable(usize::MAX));
}
