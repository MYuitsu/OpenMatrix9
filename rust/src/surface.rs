//! OM9-SURFACE-001/003/009: ordered surface input and command lifecycle.
//! Geometry is delegated to FreeCAD's native Part/OpenCascade adapter.
use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Kind {
    Sweep1 = 1,
    Sweep2 = 2,
    Loft = 3,
}
impl Kind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name
            .trim()
            .trim_start_matches('_')
            .to_ascii_lowercase()
            .as_str()
        {
            "sweep1" | "om9_surfacesweepsweep1rail" | "surfacesweepsweep1rail" => {
                Some(Self::Sweep1)
            }
            "sweep2" | "om9_surfacesweepsweep2rails" | "surfacesweepsweep2rails" => {
                Some(Self::Sweep2)
            }
            "loft" | "om9_surfaceloft" | "surfaceloft" => Some(Self::Loft),
            _ => None,
        }
    }
    pub fn feature_id(self) -> &'static str {
        match self {
            Self::Sweep1 => "OM9-SURFACE-001",
            Self::Sweep2 => "OM9-SURFACE-003",
            Self::Loft => "OM9-SURFACE-009",
        }
    }
    pub fn caption(self) -> &'static str {
        match self {
            Self::Sweep1 => "Sweep 1 rail",
            Self::Sweep2 => "Sweep 2 rails",
            Self::Loft => "Loft",
        }
    }
    pub fn rails(self) -> usize {
        match self {
            Self::Sweep1 => 1,
            Self::Sweep2 => 2,
            Self::Loft => 0,
        }
    }
    pub fn minimum_sections(self) -> usize {
        if self == Self::Loft { 2 } else { 1 }
    }
    // Only Normal and Straight Sections have a verified host implementation.
    pub fn options_valid(self, style: u32, closed_loft: bool) -> bool {
        match self {
            Self::Loft => style <= 1,
            _ => style == 0 && !closed_loft,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Phase {
    Idle = 0,
    Rail1 = 1,
    Rail2 = 2,
    Sections = 3,
    Options = 4,
}
// OCCT ContactOnBorder fails on even the single straight-profile SDK fixture.
// Single sections are transported/scaled by the rails; multiple use Contact.
// The host verifies both rails and the original profiles against the output.
pub fn sweep2_contact(sections: usize) -> u32 {
    if sections == 1 { 0 } else { 1 }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_sweep2_contact(sections: usize) -> u32 {
    sweep2_contact(sections)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_options_valid(kind: u32, style: u32, closed: bool) -> bool {
    let kind = match kind {
        1 => Kind::Sweep1,
        2 => Kind::Sweep2,
        3 => Kind::Loft,
        _ => return false,
    };
    kind.options_valid(style, closed)
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(a: [f64; 3]) -> Option<([f64; 3], f64)> {
    let n = dot(a, a).sqrt();
    if !n.is_finite() || n < 1e-10 {
        return None;
    }
    Some((a.map(|v| v / n), n))
}
fn frame(a: [f64; 3], b: [f64; 3], t: [f64; 3]) -> Option<([[f64; 3]; 3], f64)> {
    let (x, width) = unit(sub(b, a))?;
    let (z, _) = unit(cross(x, t))?;
    let y = cross(z, x);
    Some(([x, y, z], width))
}
/// Single-profile Sweep2 uses similar sections transported between the rails.
/// This is an OpenMatrix9 decision, not an undocumented Matrix default.
pub fn transport(
    a0: [f64; 3],
    b0: [f64; 3],
    t0: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    t: [f64; 3],
) -> Option<[f64; 16]> {
    if [a0, b0, t0, a, b, t]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite())
    {
        return None;
    }
    let (reference, w0) = frame(a0, b0, t0)?;
    let (current, w) = frame(a, b, t)?;
    let scale = [w / w0, 1., w / w0];
    let mut m = [0.; 16];
    m[15] = 1.;
    for row in 0..3 {
        for col in 0..3 {
            m[row * 4 + col] = (0..3)
                .map(|k| current[k][row] * scale[k] * reference[k][col])
                .sum();
        }
        m[row * 4 + 3] = a[row] - (0..3).map(|col| m[row * 4 + col] * a0[col]).sum::<f64>();
    }
    Some(m)
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_surface_transport(points: *const f64, output: *mut f64) -> bool {
    if points.is_null() || output.is_null() {
        return false;
    }
    let p = unsafe { std::slice::from_raw_parts(points, 18) };
    let v = |i| [p[i], p[i + 1], p[i + 2]];
    let Some(m) = transport(v(0), v(3), v(6), v(9), v(12), v(15)) else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(m.as_ptr(), output, 16);
    }
    true
}
pub struct Session {
    kind: Kind,
    inputs: Vec<(String, bool)>,
    phase: Phase,
}
impl Session {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            inputs: Vec::new(),
            phase: if kind.rails() == 0 {
                Phase::Sections
            } else {
                Phase::Rail1
            },
        }
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn count(&self) -> usize {
        self.inputs.len()
    }
    pub fn keys(&self) -> Vec<&str> {
        self.inputs.iter().map(|p| p.0.as_str()).collect()
    }
    pub fn add(&mut self, key: &str, closed: bool) -> Result<(), &'static str> {
        if matches!(self.phase, Phase::Idle | Phase::Options) {
            return Err("Finish or cancel the current operation first");
        }
        if key.is_empty() || key.len() > 1024 || self.inputs.len() >= 256 {
            return Err("Invalid curve reference or too many curves (maximum 256)");
        }
        if self.inputs.iter().any(|p| p.0 == key) {
            return Err("This curve is already selected");
        }
        if self.inputs.len() > self.kind.rails() && self.inputs[self.kind.rails()].1 != closed {
            return Err("Use either all open or all closed profiles");
        }
        self.inputs.push((key.to_owned(), closed));
        self.phase = if self.inputs.len() < self.kind.rails() {
            Phase::Rail2
        } else {
            Phase::Sections
        };
        Ok(())
    }
    pub fn finish(&mut self) -> Result<(), &'static str> {
        if self.phase != Phase::Sections
            || self.inputs.len() < self.kind.rails() + self.kind.minimum_sections()
        {
            return Err("Select the required rails and profiles before pressing Enter");
        }
        self.phase = Phase::Options;
        Ok(())
    }
    pub fn undo(&mut self) -> Result<(), &'static str> {
        if matches!(self.phase, Phase::Idle | Phase::Options) || self.inputs.pop().is_none() {
            return Err("No input to undo");
        }
        self.phase = if self.inputs.is_empty() && self.kind.rails() > 0 {
            Phase::Rail1
        } else if self.inputs.len() < self.kind.rails() {
            Phase::Rail2
        } else {
            Phase::Sections
        };
        Ok(())
    }
    pub fn back(&mut self) -> Result<(), &'static str> {
        if self.phase != Phase::Options {
            return Err("Options are not open");
        }
        self.phase = Phase::Sections;
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.inputs.clear();
        self.phase = Phase::Idle;
    }
}
struct State {
    session: Option<Session>,
    message: String,
}
fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(State {
            session: None,
            message: String::new(),
        })
    })
}
fn name(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // FFI callers supply a NUL-terminated UTF-8 string valid for the call.
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_surface_kind(text: *const c_char) -> u32 {
    name(text)
        .and_then(|s| Kind::from_name(&s))
        .map_or(0, |k| k as u32)
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_surface_start(text: *const c_char) -> bool {
    let Some(kind) = name(text).and_then(|s| Kind::from_name(&s)) else {
        return false;
    };
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    state.session = Some(Session::new(kind));
    state.message.clear();
    true
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_surface_add(key: *const c_char, closed: bool) -> bool {
    let Some(key) = name(key) else {
        return false;
    };
    change(|s| s.add(&key, closed))
}
fn change(f: impl FnOnce(&mut Session) -> Result<(), &'static str>) -> bool {
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    let result = state
        .session
        .as_mut()
        .ok_or("No active surface command")
        .and_then(f);
    state.message = result.err().unwrap_or("").to_owned();
    result.is_ok()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_finish() -> bool {
    change(Session::finish)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_undo() -> bool {
    change(Session::undo)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_back() -> bool {
    change(Session::back)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_cancel() {
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    state.session = None;
    state.message.clear();
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_phase() -> u32 {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, |s| s.phase as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_surface_count() -> usize {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, Session::count)
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_surface_message(buffer: *mut c_char, capacity: usize) -> usize {
    let state = state().lock().unwrap_or_else(|p| p.into_inner());
    let bytes = state.message.as_bytes();
    if !buffer.is_null() && capacity > 0 {
        let n = bytes.len().min(capacity - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.cast::<u8>(), n);
            *buffer.add(n) = 0;
        }
    }
    bytes.len()
}
