//! OM9-INFO-008: viewport tab visibility/alignment; host owns native widgets.
use std::sync::{Mutex, OnceLock};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub visible: bool,
    /// 0 top, 1 bottom, 2 left, 3 right. Bottom/visible are OM9 defaults.
    pub alignment: u32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            visible: true,
            alignment: 1,
        }
    }
}
impl State {
    pub fn encoded(self) -> u32 {
        (self.alignment << 1) | u32::from(self.visible)
    }
    pub fn decode(value: u32) -> Option<Self> {
        (value < 8).then_some(Self {
            visible: value & 1 != 0,
            alignment: value >> 1,
        })
    }
    pub fn apply(&mut self, text: &str) -> Result<bool, &'static str> {
        let words: Vec<_> = text
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect();
        let before = *self;
        match words.as_slice() {
            [] => self.visible = !self.visible,
            [value] if value == "toggle" => self.visible = !self.visible,
            [value] if value == "show" => self.visible = true,
            [value] if value == "hide" => self.visible = false,
            [align, value] if align == "align" => {
                self.alignment = match value.as_str() {
                    "top" => 0,
                    "bottom" => 1,
                    "left" => 2,
                    "right" => 3,
                    _ => return Err("Unknown alignment"),
                };
            }
            _ => return Err("Use Show, Hide, Toggle or Align Top/Bottom/Left/Right"),
        }
        Ok(before != *self)
    }
}
static STATE: OnceLock<Mutex<State>> = OnceLock::new();
fn state() -> &'static Mutex<State> {
    STATE.get_or_init(|| Mutex::new(State::default()))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_tabs_state() -> u32 {
    state().lock().map_or(3, |s| s.encoded())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_tabs_load(value: u32) -> bool {
    let Some(value) = State::decode(value) else {
        return false;
    };
    let Ok(mut current) = state().lock() else {
        return false;
    };
    *current = value;
    true
}
/// Returns 0 invalid, 1 changed, 2 unchanged. No mutation on invalid input.
/// # Safety
/// `text` must address `length` readable bytes, at most 256.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_core_tabs_apply(text: *const u8, length: usize) -> u32 {
    if text.is_null() || length > 256 {
        return 0;
    }
    let bytes = unsafe { std::slice::from_raw_parts(text, length) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return 0;
    };
    let Ok(mut current) = state().lock() else {
        return 0;
    };
    match current.apply(text) {
        Ok(true) => 1,
        Ok(false) => 2,
        Err(_) => 0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visibility_is_idempotent_and_toggle_changes_it() {
        let mut s = State::default();
        assert_eq!(s.apply("Show"), Ok(false));
        assert_eq!(s.apply("hide"), Ok(true));
        assert_eq!(s.apply(" HIDE "), Ok(false));
        assert_eq!(s.apply(""), Ok(true));
        assert_eq!(s.apply("Toggle"), Ok(true));
        assert!(!s.visible);
    }
    #[test]
    fn alignment_preserves_visibility_and_rejects_invalid_options_atomically() {
        let mut s = State {
            visible: false,
            alignment: 1,
        };
        for (name, value) in [("Top", 0), ("Bottom", 1), ("Left", 2), ("Right", 3)] {
            s.apply(&format!("Align {name}")).unwrap();
            assert_eq!(s.alignment, value);
            assert!(!s.visible);
            assert_eq!(State::decode(s.encoded()), Some(s));
        }
        let before = s;
        for text in ["Align", "Align Unknown", "Show Hide", "Toggle\0", "Left"] {
            assert!(s.apply(text).is_err());
            assert_eq!(s, before);
        }
        assert_eq!(State::decode(8), None);
    }
}
