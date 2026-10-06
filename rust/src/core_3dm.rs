//! OM9-FILE-012: policy for CAD-preserving Rhino 5 archive exchange.
use std::ffi::CStr;
pub const ICONS: [&str; 2] = ["FileImport3dm", "FileExport3dm"];
pub fn command(icon: &str) -> Option<&'static str> {
    match icon {
        "FileImport3dm" => Some("Import3dm"),
        "FileExport3dm" => Some("Export3dm"),
        _ => None,
    }
}
pub fn caption(icon: &str) -> Option<&'static str> {
    match icon {
        "FileImport3dm" => Some("Import Rhino 5 (.3dm)"),
        "FileExport3dm" => Some("Export Selected Rhino 5 (.3dm)"),
        _ => None,
    }
}
pub fn scale(unit_mm: f64, override_mm: f64) -> f64 {
    let value = if unit_mm.is_finite() && unit_mm > 0.0 {
        unit_mm
    } else {
        override_mm
    };
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_scale(unit_mm: f64, override_mm: f64) -> f64 {
    scale(unit_mm, override_mm)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_operation(index: usize) -> u32 {
    let p = crate::ffi::om9_command_id(index);
    if p.is_null() {
        return 0;
    }
    match unsafe { CStr::from_ptr(p) }.to_bytes() {
        b"Import3dm" => 1,
        b"Export3dm" => 2,
        _ => 0,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_3dm_type_supported(kind: u32) -> bool {
    (1..=5).contains(&kind)
}
