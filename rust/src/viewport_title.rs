//! Matrix viewport label actions. Native host applies validated rendering adapters.
use std::ffi::{CString, c_char};
use std::sync::OnceLock;
pub const MODES: &[&str] = &[
    "Wireframe",
    "Shaded",
    "Rendered",
    "Ghosted",
    "X-Ray",
    "Technical",
    "Art Color",
    "Art Color Wires",
    "ClayooSculpt",
    "Detect All",
    "Detect Backfaces",
    "Detect Naked Edges",
    "Detect UV Normals",
    "Floorplan",
    "Ice",
    "Legacy",
    "Machine",
    "Pastel",
    "Plastic",
    "Presentation",
    "Shiny Plastic",
    "Simple Shade",
    "Tech White",
    "Vivid",
    "Wire Render",
    "Working Render",
    "Working Shade",
];
pub fn native_mode(name: &str) -> Option<u32> {
    match name {
        "Shaded" => Some(0),
        "Wireframe" => Some(1),
        _ => None,
    }
}
pub fn next_single(current: Option<usize>, clicked: usize) -> Option<Option<usize>> {
    if clicked >= 4 {
        None
    } else {
        Some(if current == Some(clicked) {
            None
        } else {
            Some(clicked)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_viewport_mode_count() -> usize {
    MODES.len()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_viewport_mode_name(index: usize) -> *const c_char {
    static STRINGS: OnceLock<Vec<CString>> = OnceLock::new();
    STRINGS
        .get_or_init(|| {
            MODES
                .iter()
                .map(|s| CString::new(*s).expect("static mode name"))
                .collect()
        })
        .get(index)
        .map_or(std::ptr::null(), |s| s.as_ptr())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_viewport_native_mode(index: usize) -> i32 {
    MODES
        .get(index)
        .and_then(|s| native_mode(s))
        .map_or(-1, |n| n as i32)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_viewport_next_single(current: i32, clicked: usize) -> i32 {
    let current = if current < 0 {
        None
    } else {
        Some(current as usize)
    };
    next_single(current, clicked).map_or(-2, |s| s.map_or(-1, |n| n as i32))
}
