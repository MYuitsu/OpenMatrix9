// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rust owns command identity, immutable preview snapshots, phases and options.
//! Qt/document objects never cross this boundary; snapshots are independent copies.
use std::ffi::{CStr, CString, c_char};
pub const ICONS: [&str; 3] = [
    "TransformCageEditingCreateCage",
    "TransformCageEditingCageEdit",
    "ReleaseFromCage",
];
pub fn command(icon: &str) -> Option<&'static str> {
    match icon {
        "TransformCageEditingCreateCage" => Some("OM9_TransformCageEditingCreateCage"),
        "TransformCageEditingCageEdit" => Some("OM9_TransformCageEditingCageEdit"),
        "ReleaseFromCage" => Some("OM9_ReleaseFromCage"),
        _ => None,
    }
}
pub fn caption(icon: &str) -> Option<&'static str> {
    match icon {
        "TransformCageEditingCreateCage" => Some("Create Cage"),
        "TransformCageEditingCageEdit" => Some("Cage Edit"),
        "ReleaseFromCage" => Some("Release From Cage"),
        _ => None,
    }
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    None = 0,
    Create = 1,
    Capture = 2,
    Release = 3,
}
pub fn kind(name: &str) -> Kind {
    let n = name.to_ascii_lowercase();
    match n.strip_prefix("om9_").unwrap_or(&n) {
        "cage" | "om9-transform-041" | "transformcageeditingcreatecage" => Kind::Create,
        "cageedit" | "om9-transform-020" | "transformcageeditingcageedit" => Kind::Capture,
        "releasefromcage" | "om9-transform-022" => Kind::Release,
        _ => Kind::None,
    }
}
pub fn available(document: bool, gui: bool, editable: bool, allowed: bool) -> bool {
    document && gui && editable && allowed
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Inputs = 1,
    Control = 2,
    Options = 3,
    Ready = 4,
}
#[derive(Clone, Debug)]
pub struct Input {
    pub name: CString,
    pub signature: CString,
    pub lo: [f64; 3],
    pub hi: [f64; 3],
}
impl Input {
    pub fn new(
        name: &str,
        signature: &str,
        lo: [f64; 3],
        hi: [f64; 3],
    ) -> Result<Self, &'static str> {
        if name.is_empty()
            || signature.is_empty()
            || lo.iter().chain(&hi).any(|x| !x.is_finite())
            || (0..3).any(|a| lo[a] > hi[a])
        {
            return Err("Invalid native geometry snapshot");
        }
        Ok(Self {
            name: CString::new(name).map_err(|_| "Invalid name")?,
            signature: CString::new(signature).map_err(|_| "Invalid snapshot")?,
            lo,
            hi,
        })
    }
}
pub struct Session {
    pub kind: Kind,
    pub phase: Phase,
    pub inputs: Vec<Input>,
    pub control: Option<Input>,
    pub retained: bool,
    pub counts: [usize; 3],
    pub degrees: [usize; 3],
    pub local: bool,
    pub falloff: f64,
    pub lo: [f64; 3],
    pub hi: [f64; 3],
    error: CString,
}
impl Session {
    pub fn validate_commit(&self) -> Result<(), &'static str> {
        if !matches!(self.phase, Phase::Options | Phase::Ready) {
            return Err("Complete the Cage selection before committing");
        }
        Ok(())
    }
    fn parameters(&mut self, counts: [usize; 3], degrees: [usize; 3]) -> Result<(), &'static str> {
        let mut total = 1usize;
        for a in 0..3 {
            if !(2..=128).contains(&counts[a])
                || !(1..=16).contains(&degrees[a])
                || counts[a] <= degrees[a]
            {
                return Err(
                    "Counts must exceed degree; counts must be 2..128; degree must be 1..16",
                );
            }
            total = total
                .checked_mul(counts[a])
                .filter(|x| *x <= 1_000_000)
                .ok_or("Cage exceeds control point budget")?;
        }
        self.counts = counts;
        self.degrees = degrees;
        Ok(())
    }
    pub fn new(kind: Kind) -> Result<Self, &'static str> {
        if kind == Kind::None {
            return Err("Unsupported Cage command");
        }
        Ok(Self {
            kind,
            phase: Phase::Inputs,
            inputs: vec![],
            control: None,
            retained: false,
            counts: [2; 3],
            degrees: [1; 3],
            local: false,
            falloff: 0.,
            lo: [0.; 3],
            hi: [0.; 3],
            error: CString::default(),
        })
    }
    pub fn add(&mut self, input: Input) -> Result<(), &'static str> {
        if self.phase != Phase::Inputs {
            return Err("Inputs are frozen; cancel and restart to change captives");
        }
        if !self.inputs.iter().any(|i| i.name == input.name) {
            self.inputs.push(input);
        }
        Ok(())
    }
    pub fn set_control(&mut self, input: Input, retained: bool) -> Result<(), &'static str> {
        if self.phase != Phase::Control {
            return Err("Select captives and press Enter before choosing a control");
        }
        if self.inputs.iter().any(|i| i.name == input.name) {
            return Err("The control cannot also be a captive");
        }
        self.control = Some(input);
        self.retained = retained;
        self.phase = Phase::Options;
        Ok(())
    }
    pub fn advance(&mut self) -> Result<(), &'static str> {
        match self.phase {
            Phase::Inputs => {
                if self.inputs.is_empty() {
                    return Err("Select whole objects or type their names");
                }
                self.lo = std::array::from_fn(|a| {
                    self.inputs
                        .iter()
                        .map(|i| i.lo[a])
                        .fold(f64::INFINITY, f64::min)
                });
                self.hi = std::array::from_fn(|a| {
                    self.inputs
                        .iter()
                        .map(|i| i.hi[a])
                        .fold(f64::NEG_INFINITY, f64::max)
                });
                if self.kind == Kind::Create
                    && (0..3)
                        .any(|a| self.hi[a] <= self.lo[a] || !(self.hi[a] - self.lo[a]).is_finite())
                {
                    return Err(
                        "World bounding box requires positive extents in X, Y and Z; no automatic padding",
                    );
                }
                self.phase = match self.kind {
                    Kind::Create => Phase::Options,
                    Kind::Capture => Phase::Control,
                    _ => Phase::Ready,
                };
            }
            Phase::Control => {
                return Err(
                    "Choose one existing native 3D cage; retained 3DM controls require explicit Restore",
                );
            }
            Phase::Options => self.phase = Phase::Ready,
            Phase::Ready => {}
        }
        Ok(())
    }
    pub fn verify(&self, name: &str, signature: &str) -> Result<(), &'static str> {
        let i = self
            .inputs
            .iter()
            .chain(self.control.iter())
            .find(|i| i.name.to_bytes() == name.as_bytes())
            .ok_or("Unknown preview input")?;
        if i.signature.to_bytes() != signature.as_bytes() {
            Err("Cage input geometry or global placement changed; cancel and reselect")
        } else {
            Ok(())
        }
    }
    pub fn option(&mut self, key: &str, value: &str) -> Result<(), &'static str> {
        if self.phase != Phase::Options {
            return Err("Options require completed selection");
        }
        let key = key.to_ascii_lowercase();
        match key.as_str() {
            "parameters" if self.kind == Kind::Create => {
                let values = value
                    .split(',')
                    .map(|v| {
                        v.trim()
                            .parse::<usize>()
                            .map_err(|_| "Count/degree must be a finite integer")
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() != 6 {
                    return Err("Parameters require three counts and three degrees");
                }
                self.parameters(
                    [values[0], values[1], values[2]],
                    [values[3], values[4], values[5]],
                )?;
            }
            "coordinates" | "coordinatesystem" if self.kind == Kind::Create => {
                if !value.eq_ignore_ascii_case("World") {
                    return Err("Only World coordinates are supported");
                }
            }
            "shape" if self.kind == Kind::Create => {
                if !value.eq_ignore_ascii_case("BoundingBox") {
                    return Err("Only BoundingBox cage creation is supported");
                }
            }
            "region" if self.kind == Kind::Capture => {
                self.local = match value.to_ascii_lowercase().as_str() {
                    "global" => false,
                    "local" => true,
                    _ => return Err("Region requires Global or Local"),
                };
            }
            "falloff" if self.kind == Kind::Capture => {
                let v = value
                    .parse::<f64>()
                    .map_err(|_| "Falloff must be finite and nonnegative")?;
                if !v.is_finite() || v < 0. {
                    return Err("Falloff must be finite and nonnegative");
                }
                self.falloff = v;
            }
            _ if self.kind == Kind::Create => {
                let (axis, degree) = match key.as_str() {
                    "ucount" => (0, false),
                    "vcount" => (1, false),
                    "wcount" => (2, false),
                    "udegree" => (0, true),
                    "vdegree" => (1, true),
                    "wdegree" => (2, true),
                    _ => {
                        return Err(
                            "Supported Cage options: World, BoundingBox, U/V/WCount and U/V/WDegree",
                        );
                    }
                };
                let v = value
                    .parse::<usize>()
                    .map_err(|_| "Count/degree must be a finite integer")?;
                let mut counts = self.counts;
                let mut degrees = self.degrees;
                if degree {
                    degrees[axis] = v
                } else {
                    counts[axis] = v
                }
                self.parameters(counts, degrees)?;
            }
            _ => {
                return Err("Only existing native 3D cage, Global/Local and Falloff are supported");
            }
        };
        Ok(())
    }
}
unsafe fn string<'a>(p: *const c_char) -> Result<&'a str, &'static str> {
    if p.is_null() {
        Err("Null text")
    } else {
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .map_err(|_| "Invalid UTF-8")
    }
}
// All handles originate here and are freed here. Native caller supplies live,
// aligned buffers and serializes GUI-thread access/free. Returned strings remain
// borrowed until the next mutation/free. Every fallible mutation contains panic.
fn mutate(h: *mut Session, f: impl FnOnce(&mut Session) -> Result<(), &'static str>) -> i32 {
    if h.is_null() {
        return -1;
    }
    let s = unsafe { &mut *h };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(s))) {
        Ok(Ok(())) => 0,
        result => {
            s.error = CString::new(match result {
                Ok(Err(e)) => e,
                _ => "Cage session panic",
            })
            .unwrap();
            -1
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_command_kind(p: *const c_char) -> u32 {
    kind(unsafe { string(p) }.unwrap_or("")) as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_cage_command_caption(k: u32) -> *const c_char {
    match k {
        1 => c"Cage".as_ptr(),
        2 => c"CageEdit".as_ptr(),
        3 => c"ReleaseFromCage".as_ptr(),
        _ => c"".as_ptr(),
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_cage_command_available(d: bool, g: bool, e: bool, a: bool) -> bool {
    available(d, g, e, a)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_cage_session_new(k: u32) -> *mut Session {
    let kind = match k {
        1 => Kind::Create,
        2 => Kind::Capture,
        3 => Kind::Release,
        _ => return std::ptr::null_mut(),
    };
    Box::into_raw(Box::new(Session::new(kind).unwrap()))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_free(h: *mut Session) {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_add(
    h: *mut Session,
    n: *const c_char,
    s: *const c_char,
    lo: *const f64,
    hi: *const f64,
    role: u32,
) -> i32 {
    mutate(h, |h| {
        if lo.is_null() || hi.is_null() {
            return Err("Null bounds");
        }
        let input = Input::new(
            unsafe { string(n) }?,
            unsafe { string(s) }?,
            unsafe { *(lo as *const [f64; 3]) },
            unsafe { *(hi as *const [f64; 3]) },
        )?;
        if role == 0 {
            h.add(input)
        } else {
            h.set_control(input, role == 2)
        }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_cage_session_advance(h: *mut Session) -> i32 {
    mutate(h, Session::advance)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_cage_session_prepare(h: *mut Session) -> i32 {
    mutate(h, |h| h.validate_commit())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_option(
    h: *mut Session,
    k: *const c_char,
    v: *const c_char,
) -> i32 {
    mutate(h, |h| {
        h.option(unsafe { string(k) }?, unsafe { string(v) }?)
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_verify(
    h: *mut Session,
    n: *const c_char,
    s: *const c_char,
) -> i32 {
    mutate(h, |h| {
        h.verify(unsafe { string(n) }?, unsafe { string(s) }?)
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_error(h: *const Session) -> *const c_char {
    if h.is_null() {
        c"Invalid Cage session".as_ptr()
    } else {
        unsafe { (*h).error.as_ptr() }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_phase(h: *const Session) -> u32 {
    if h.is_null() {
        0
    } else {
        unsafe { (*h).phase as u32 }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_count(h: *const Session) -> usize {
    if h.is_null() {
        0
    } else {
        unsafe { (*h).inputs.len() }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_name(h: *const Session, i: usize) -> *const c_char {
    if h.is_null() {
        return std::ptr::null();
    }
    unsafe {
        (&(*h).inputs)
            .get(i)
            .map(|i| i.name.as_ptr())
            .unwrap_or(std::ptr::null())
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_control(h: *const Session) -> *const c_char {
    if h.is_null() {
        return std::ptr::null();
    }
    unsafe {
        (*h).control
            .as_ref()
            .map(|i| i.name.as_ptr())
            .unwrap_or(std::ptr::null())
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_values(h: *const Session, out: *mut f64) -> u32 {
    if h.is_null() || out.is_null() {
        return 0;
    }
    let h = unsafe { &*h };
    let data: [f64; 13] = std::array::from_fn(|i| match i {
        0..=2 => h.lo[i],
        3..=5 => h.hi[i - 3],
        6..=8 => h.counts[i - 6] as f64,
        9..=11 => h.degrees[i - 9] as f64,
        _ => h.falloff,
    });
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), out, 13) };
    u32::from(h.local) | u32::from(h.retained) << 1
}
