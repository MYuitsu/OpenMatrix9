//! Defaults explicitly stated by the local Matrix specifications.
use std::{
    ffi::c_char,
    sync::atomic::{AtomicBool, Ordering},
};
pub const SHORTCUTS: &[(&str, &str)] = &[
    ("F2", "CommandHistory"),
    ("F4", "@origin"),
    ("F5", "CenterViewport"),
    ("F6", "F6"),
    ("F7", "@grid"),
    ("F8", "Ortho"),
    ("F10", "PointsOn"),
    ("Ctrl+T", "Properties"),
    ("Ctrl+Q", "Group"),
    ("Ctrl+W", "UnGroup"),
    ("Ctrl+Alt+C", "gvCenterObjects"),
];
pub const ICONS: [&str; 4] = ["CommandHistory", "ObjectProperties", "OrthoSnapON", "F6"];
pub fn command(icon: &str) -> Option<&'static str> {
    match icon {
        "CommandHistory" => Some("CommandHistory"),
        "ObjectProperties" => Some("Properties"),
        "OrthoSnapON" => Some("Ortho"),
        "F6" => Some("F6"),
        "TopIconControlPointsOn" => Some("PointsOn"),
        "EditGroupsGroup" => Some("Group"),
        "EditGroupsUnGroup" => Some("UnGroup"),
        "AnalyzeCenterPoint" => Some("gvCenterObjects"),
        _ => None,
    }
}
pub fn ortho_active(saved: bool, shift: bool, master: bool) -> bool {
    master && (saved ^ shift)
}
static ORTHO: AtomicBool = AtomicBool::new(false);
#[unsafe(no_mangle)]
pub extern "C" fn om9_ortho_enabled() -> bool {
    ORTHO.load(Ordering::SeqCst)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ortho_load(value: bool) {
    ORTHO.store(value, Ordering::SeqCst);
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ortho_toggle() -> bool {
    !ORTHO.fetch_xor(true, Ordering::SeqCst)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ortho_active(shift: bool) -> bool {
    ortho_active(
        om9_ortho_enabled(),
        shift,
        crate::core_snaps::om9_snap_state() & 1 != 0,
    )
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_keyboard_count() -> usize {
    SHORTCUTS.len()
}
// Static NUL-terminated storage: safe for the complete native module lifetime.
const KEYS: [&[u8]; 11] = [
    b"F2\0",
    b"F4\0",
    b"F5\0",
    b"F6\0",
    b"F7\0",
    b"F8\0",
    b"F10\0",
    b"Ctrl+T\0",
    b"Ctrl+Q\0",
    b"Ctrl+W\0",
    b"Ctrl+Alt+C\0",
];
const TARGETS: [&[u8]; 11] = [
    b"CommandHistory\0",
    b"@origin\0",
    b"CenterViewport\0",
    b"F6\0",
    b"@grid\0",
    b"Ortho\0",
    b"PointsOn\0",
    b"Properties\0",
    b"Group\0",
    b"UnGroup\0",
    b"gvCenterObjects\0",
];
#[unsafe(no_mangle)]
pub extern "C" fn om9_keyboard_key(i: usize) -> *const c_char {
    KEYS.get(i).map_or(std::ptr::null(), |s| s.as_ptr().cast())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_keyboard_target(i: usize) -> *const c_char {
    TARGETS
        .get(i)
        .map_or(std::ptr::null(), |s| s.as_ptr().cast())
}
