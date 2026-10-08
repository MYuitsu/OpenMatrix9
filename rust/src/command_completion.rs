//! Named-command matching only; availability remains a native host decision.
use std::{
    ffi::{CStr, CString, c_char},
    sync::OnceLock,
};
pub fn names() -> &'static [&'static str] {
    static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names: Vec<_> = include_str!("../../Resources/menu/CommandNames.txt")
            .lines()
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .collect();
        names.extend(["Sweep1", "Sweep2", "Loft", "Box", "Sphere"]);
        names.extend(
            crate::core_keyboard::ICONS
                .into_iter()
                .filter_map(crate::core_keyboard::command),
        );
        names.extend(
            crate::core_views::ICONS
                .into_iter()
                .filter_map(crate::core_views::command),
        );
        names.extend(
            crate::core_snaps::ICONS
                .into_iter()
                .filter_map(crate::core_snaps::command),
        );
        names.sort_unstable_by_key(|name| name.to_ascii_lowercase());
        names.extend(
            crate::core_3dm::ICONS
                .into_iter()
                .filter_map(crate::core_3dm::command),
        );
        names.sort_unstable_by_key(|name| name.to_ascii_lowercase());
        names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        names
    })
}
pub fn rank(name: &str, input: &str) -> u32 {
    let input = input.trim().trim_start_matches('_');
    if input.is_empty()
        || input.len() > 128
        || !input.starts_with(|c: char| c.is_ascii_alphabetic())
        || !input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ' ')
    {
        return 0;
    }
    let name = name.to_ascii_lowercase();
    let input = input.to_ascii_lowercase();
    if name == input {
        1
    } else if name.starts_with(&input) {
        2
    } else if name.contains(&input) {
        3
    } else {
        0
    }
}
fn strings() -> &'static [CString] {
    static STRINGS: OnceLock<Vec<CString>> = OnceLock::new();
    STRINGS.get_or_init(|| {
        names()
            .iter()
            .filter_map(|s| CString::new(*s).ok())
            .collect()
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_console_completion_count() -> usize {
    strings().len()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_console_completion_name(index: usize) -> *const c_char {
    strings()
        .get(index)
        .map_or(std::ptr::null(), |s| s.as_ptr())
}
/// # Safety
/// Non-null input must address a live NUL-terminated string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_console_completion_rank(index: usize, input: *const c_char) -> u32 {
    if input.is_null() {
        return 0;
    }
    let Some(name) = names().get(index) else {
        return 0;
    };
    unsafe { CStr::from_ptr(input) }
        .to_str()
        .map_or(0, |s| rank(name, s))
}
