//! Native Edit workflow: OM9-TOP11-005/008/010, OM9-SOLID-001..004.
//! OpenMatrix9 decisions: curve Trim with true 3D intersections; solid Booleans.
use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Kind {
    Join = 1,
    Explode = 2,
    Trim = 3,
    Difference = 4,
    Intersection = 5,
    Union = 6,
    TwoObjects = 7,
}
impl Kind {
    pub fn from_name(s: &str) -> Option<Self> {
        match s
            .trim()
            .trim_start_matches('_')
            .to_ascii_lowercase()
            .as_str()
        {
            "join" | "topiconjoin" | "om9_topiconjoin" => Some(Self::Join),
            "explode" | "topiconexplode" | "om9_topiconexplode" => Some(Self::Explode),
            "trim" | "topicontrim" | "om9_topicontrim" => Some(Self::Trim),
            "booleandifference" | "soliddifference" | "om9_soliddifference" => {
                Some(Self::Difference)
            }
            "booleanintersection" | "solidintersection" | "om9_solidintersection" => {
                Some(Self::Intersection)
            }
            "booleanunion" | "solidunion" | "om9_solidunion" => Some(Self::Union),
            "boolean2objects" | "solidbooleantwoobjects" | "om9_solidbooleantwoobjects" => {
                Some(Self::TwoObjects)
            }
            _ => None,
        }
    }
    pub fn caption(self) -> &'static str {
        match self {
            Self::Join => "Join",
            Self::Explode => "Explode",
            Self::Trim => "Trim",
            Self::Difference => "BooleanDifference",
            Self::Intersection => "BooleanIntersection",
            Self::Union => "BooleanUnion",
            Self::TwoObjects => "Boolean2Objects",
        }
    }
    pub fn feature(self) -> &'static str {
        match self {
            Self::Join => "OM9-TOP11-008",
            Self::Explode => "OM9-TOP11-005",
            Self::Trim => "OM9-TOP11-010",
            Self::Difference => "OM9-SOLID-001",
            Self::Intersection => "OM9-SOLID-002",
            Self::Union => "OM9-SOLID-003",
            Self::TwoObjects => "OM9-SOLID-004",
        }
    }
}
pub struct Session {
    pub kind: Kind,
    pub phase: u32,
    pub first: Vec<String>,
    pub second: Vec<String>,
    pub cycle: u32,
    pub delete_input: bool,
    pub extend_lines: bool,
    pub apparent_intersections: bool,
    pub tolerance: f64,
}
impl Session {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            phase: 1,
            first: vec![],
            second: vec![],
            cycle: 0,
            delete_input: (kind as u32) <= 3,
            extend_lines: false,
            apparent_intersections: false,
            tolerance: 1e-7,
        }
    }
    pub fn set_option(&mut self, option: u32, value: bool) -> bool {
        if self.kind != Kind::Trim {
            return false;
        }
        match option {
            1 => self.extend_lines = value,
            2 => self.apparent_intersections = value,
            _ => return false,
        }
        true
    }
    pub fn set_tolerance(&mut self, value: f64) -> bool {
        if self.kind != Kind::Join || !value.is_finite() || !(1e-9..=1e6).contains(&value) {
            return false;
        }
        self.tolerance = value;
        true
    }
    pub fn add(&mut self, name: &str) -> bool {
        if !(1..=2).contains(&self.phase)
            || name.is_empty()
            || self
                .first
                .iter()
                .chain(self.second.iter())
                .any(|s| s == name)
            || (self.kind == Kind::TwoObjects && self.first.len() == 2)
        {
            return false;
        }
        if self.phase == 1 {
            self.first.push(name.into());
        } else {
            self.second.push(name.into());
        }
        true
    }
    pub fn finish(&mut self) -> bool {
        if self.phase == 2 {
            if self.second.is_empty() {
                return false;
            }
            self.phase = 3;
            return true;
        }
        if self.phase != 1 {
            return false;
        }
        let min = if self.kind == Kind::Explode
            || self.kind == Kind::Difference
            || self.kind == Kind::Intersection
        {
            1
        } else {
            2
        };
        if self.first.len() < min {
            return false;
        }
        self.phase = match self.kind {
            Kind::Difference | Kind::Intersection => 2,
            Kind::Trim => 4,
            _ => 3,
        };
        true
    }
    pub fn undo_selection(&mut self) -> bool {
        if self.phase == 2 && self.second.is_empty() {
            self.phase = 1;
            return true;
        }
        match self.phase {
            1 => self.first.pop().is_some(),
            2 => self.second.pop().is_some(),
            _ => false,
        }
    }
    pub fn cycle(&mut self) -> bool {
        if self.kind != Kind::TwoObjects || self.phase != 3 {
            return false;
        }
        self.cycle = (self.cycle + 1) % 5;
        true
    }
}
static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();
fn state() -> &'static Mutex<Option<Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
}
unsafe fn name<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(p) }.to_str().ok()
    }
}
/// # Safety
/// A non-null input must point to a readable NUL-terminated string for this call. No caller storage is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_edit_kind(p: *const c_char) -> u32 {
    unsafe { name(p) }
        .and_then(Kind::from_name)
        .map_or(0, |k| k as u32)
}
/// # Safety
/// A non-null input must point to a readable NUL-terminated string for this call. No caller storage is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_edit_start(p: *const c_char) -> bool {
    let Some(k) = unsafe { name(p) }.and_then(Kind::from_name) else {
        return false;
    };
    *state().lock().unwrap() = Some(Session::new(k));
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_cancel() {
    *state().lock().unwrap() = None;
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_phase() -> u32 {
    state().lock().unwrap().as_ref().map_or(0, |s| s.phase)
}
/// # Safety
/// A non-null input must point to a readable NUL-terminated string for this call. No caller storage is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_edit_add(p: *const c_char) -> bool {
    let Some(n) = (unsafe { name(p) }) else {
        return false;
    };
    state().lock().unwrap().as_mut().is_some_and(|s| s.add(n))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_finish() -> bool {
    state()
        .lock()
        .unwrap()
        .as_mut()
        .is_some_and(Session::finish)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_back() {
    if let Some(s) = state().lock().unwrap().as_mut() {
        s.phase = if s.kind == Kind::Difference || s.kind == Kind::Intersection {
            2
        } else {
            1
        };
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_undo() -> bool {
    state()
        .lock()
        .unwrap()
        .as_mut()
        .is_some_and(Session::undo_selection)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_cycle() -> bool {
    state().lock().unwrap().as_mut().is_some_and(Session::cycle)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_mode() -> u32 {
    state().lock().unwrap().as_ref().map_or(0, |s| s.cycle)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_count(group: u32) -> usize {
    state().lock().unwrap().as_ref().map_or(0, |s| {
        if group == 1 {
            s.first.len()
        } else {
            s.second.len()
        }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_delete_input(value: i32) -> bool {
    let mut state = state().lock().unwrap();
    let Some(s) = state.as_mut() else {
        return false;
    };
    if value >= 0 {
        s.delete_input = value != 0;
    }
    s.delete_input
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_set_option(option: u32, value: bool) -> bool {
    state()
        .lock()
        .unwrap()
        .as_mut()
        .is_some_and(|s| s.set_option(option, value))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_option(option: u32) -> bool {
    state()
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|s| match option {
            1 => s.extend_lines,
            2 => s.apparent_intersections,
            _ => false,
        })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_set_tolerance(value: f64) -> bool {
    state()
        .lock()
        .unwrap()
        .as_mut()
        .is_some_and(|s| s.set_tolerance(value))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edit_tolerance() -> f64 {
    state()
        .lock()
        .unwrap()
        .as_ref()
        .map_or(1e-7, |s| s.tolerance)
}
