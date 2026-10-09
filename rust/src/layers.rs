// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded OM9 layer policy. Native owns durable document/view properties;
//! this module owns identifiers, defaults, typed selection and output validation.
//! No process-global active layer or native pointers are retained.
use std::ffi::c_char;

pub const COUNT: usize = 32;
pub const ROOT_COLOR: [f64; 3] = [0.0, 130.0 / 255.0, 85.0 / 255.0];
const NAMES: [&str; COUNT] = [
    "Metal 01", "Metal 02", "Metal 03", "Metal 04", "Gem 01", "Gem 02", "Gem 03", "Gem 04",
    "User 01", "User 02", "User 03", "User 04", "Heads", "Finger", "Cutting", "Creation",
    "User 17", "User 18", "User 19", "User 20", "User 25", "User 26", "User 27", "User 28",
    "User 21", "User 22", "User 23", "User 24", "User 29", "User 30", "User 31", "User 32",
];
const C_NAMES: [&[u8]; COUNT] = [
    b"Metal 01\0",
    b"Metal 02\0",
    b"Metal 03\0",
    b"Metal 04\0",
    b"Gem 01\0",
    b"Gem 02\0",
    b"Gem 03\0",
    b"Gem 04\0",
    b"User 01\0",
    b"User 02\0",
    b"User 03\0",
    b"User 04\0",
    b"Heads\0",
    b"Finger\0",
    b"Cutting\0",
    b"Creation\0",
    b"User 17\0",
    b"User 18\0",
    b"User 19\0",
    b"User 20\0",
    b"User 25\0",
    b"User 26\0",
    b"User 27\0",
    b"User 28\0",
    b"User 21\0",
    b"User 22\0",
    b"User 23\0",
    b"User 24\0",
    b"User 29\0",
    b"User 30\0",
    b"User 31\0",
    b"User 32\0",
];
const COLORS: [u32; COUNT] = [
    0x229987, 0x319f49, 0x70c64b, 0xa0cf78, 0x598bc2, 0x4f80cb, 0x7aa7ce, 0xaccae4, 0xe23434,
    0x70c133, 0x3468ff, 0x777777, 0x851ca1, 0xad423c, 0xca7135, 0xdfb126, 0xdf9098, 0xdabb7c,
    0xbd8735, 0xa5c14b, 0x47c7aa, 0x72c6cb, 0x607cab, 0x4d626a, 0x57b397, 0x586076, 0xc6bbb7,
    0x6c5578, 0x57536b, 0x1e555b, 0x0c343f, 0x1b3555,
];

pub fn name(index: i32) -> Option<&'static str> {
    index
        .checked_sub(1)
        .and_then(|i| usize::try_from(i).ok())
        .and_then(|i| NAMES.get(i).copied())
}
pub fn default_color(index: i32) -> Option<[f64; 3]> {
    let i = usize::try_from(index.checked_sub(1)?).ok()?;
    let rgb = *COLORS.get(i)?;
    Some([
        ((rgb >> 16) & 255) as f64 / 255.,
        ((rgb >> 8) & 255) as f64 / 255.,
        (rgb & 255) as f64 / 255.,
    ])
}
pub fn valid_color(color: [f64; 3]) -> bool {
    color
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerSnapshot {
    /// 0 means legacy root output; 1..=32 are explicit stable OM9 indexes.
    pub index: i32,
    pub exists: bool,
    pub locked: bool,
    pub visible: bool,
    pub color: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LayerError {
    InvalidIndex = 1,
    MissingLayer = 2,
    Locked = 3,
    InvalidColor = 4,
}
impl LayerSnapshot {
    pub fn output_plan(self) -> Result<Self, LayerError> {
        if self.index == 0 {
            return Ok(Self {
                index: 0,
                exists: false,
                locked: false,
                visible: true,
                color: ROOT_COLOR,
            });
        }
        if !(1..=COUNT as i32).contains(&self.index) {
            return Err(LayerError::InvalidIndex);
        }
        if !self.exists {
            return Err(LayerError::MissingLayer);
        }
        if self.locked {
            return Err(LayerError::Locked);
        }
        if !valid_color(self.color) {
            return Err(LayerError::InvalidColor);
        }
        // Hidden layers deliberately accept new outputs, with Visibility=false.
        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Unhandled,
    Invalid,
    Index(i32),
}
pub fn parse_selection(text: &str) -> Selection {
    let text = text.trim();
    let Some(prefix) = text.get(..5) else {
        return Selection::Unhandled;
    };
    if !prefix.eq_ignore_ascii_case("Layer") {
        return Selection::Unhandled;
    }
    let tail = &text[5..];
    if !tail.is_empty() && !tail.starts_with('=') && !tail.starts_with(char::is_whitespace) {
        return Selection::Unhandled;
    }
    if text.len() > 256 || text.contains('\0') {
        return Selection::Invalid;
    }
    let value = tail.trim().strip_prefix('=').unwrap_or(tail.trim()).trim();
    let value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else {
        value
    };
    if value.eq_ignore_ascii_case("None") || value == "0" {
        return Selection::Index(0);
    }
    if let Ok(index) = value.parse::<i32>() {
        return if (1..=COUNT as i32).contains(&index) {
            Selection::Index(index)
        } else {
            Selection::Invalid
        };
    }
    NAMES
        .iter()
        .position(|name| name.eq_ignore_ascii_case(value))
        .map_or(Selection::Invalid, |i| Selection::Index(i as i32 + 1))
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_name(index: i32) -> *const c_char {
    index
        .checked_sub(1)
        .and_then(|i| usize::try_from(i).ok())
        .and_then(|i| C_NAMES.get(i))
        .map_or(std::ptr::null(), |s| s.as_ptr().cast())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_default_color(index: i32, channel: usize) -> f64 {
    default_color(index)
        .and_then(|v| v.get(channel).copied())
        .unwrap_or(f64::NAN)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_root_color(channel: usize) -> f64 {
    ROOT_COLOR.get(channel).copied().unwrap_or(f64::NAN)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_layer_output_status(
    index: i32,
    exists: bool,
    locked: bool,
    visible: bool,
    r: f64,
    g: f64,
    b: f64,
) -> i32 {
    LayerSnapshot {
        index,
        exists,
        locked,
        visible,
        color: [r, g, b],
    }
    .output_plan()
    .map_or_else(|e| e as i32, |_| 0)
}
/// # Safety
/// Caller supplies readable `len` UTF-8 bytes valid for this call. Bounded to
/// 256 bytes before constructing the borrowed slice; nothing crosses ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_layer_parse_selection(text: *const u8, len: usize) -> i32 {
    if text.is_null() {
        return -1;
    }
    if len > 256 {
        // Only the bounded ASCII command prefix is inspected for oversized
        // input, so unrelated commands can continue through their own router.
        let prefix = unsafe { std::slice::from_raw_parts(text, 5) };
        return if prefix.eq_ignore_ascii_case(b"Layer") {
            -1
        } else {
            -2
        };
    }
    let Ok(text) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(text, len) }) else {
        return -1;
    };
    match parse_selection(text) {
        Selection::Unhandled => -2,
        Selection::Invalid => -1,
        Selection::Index(i) => i,
    }
}
