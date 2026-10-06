//! OM9-FILE-008 / OM9-INFO-006 project note commit decisions.
use std::slice;

pub const MAX_BYTES: usize = 1_048_576;
pub const ICONS: [&str; 2] = ["FileNotes", "ProjectNotes"];
pub fn caption(icon: &str) -> Option<&'static str> {
    match icon {
        "FileNotes" => Some("Notes"),
        "ProjectNotes" => Some("Project Notes"),
        _ => None,
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Edit {
    Unchanged,
    Changed,
}
pub fn decide_edit(previous: &str, proposed: &str) -> Result<Edit, &'static str> {
    if proposed.len() > MAX_BYTES || proposed.contains('\0') {
        return Err("Notes exceed the storage limit or contain NUL");
    }
    Ok(if previous == proposed {
        Edit::Unchanged
    } else {
        Edit::Changed
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_notes_metadata_key() -> *const std::ffi::c_char {
    c"OpenMatrix9.ProjectNotes".as_ptr()
}
/// 0: unchanged; 1: changed; 2: invalid. Host buffers remain caller-owned.
/// # Safety
/// Each pointer must address its stated number of readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_notes_decide(
    previous: *const u8,
    previous_len: usize,
    proposed: *const u8,
    proposed_len: usize,
) -> u32 {
    if previous.is_null()
        || proposed.is_null()
        || previous_len > MAX_BYTES
        || proposed_len > MAX_BYTES
    {
        return 2;
    }
    let previous = unsafe { slice::from_raw_parts(previous, previous_len) };
    let proposed = unsafe { slice::from_raw_parts(proposed, proposed_len) };
    let (Ok(previous), Ok(proposed)) =
        (std::str::from_utf8(previous), std::str::from_utf8(proposed))
    else {
        return 2;
    };
    match decide_edit(previous, proposed) {
        Ok(Edit::Unchanged) => 0,
        Ok(Edit::Changed) => 1,
        Err(_) => 2,
    }
}
