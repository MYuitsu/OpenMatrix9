//! OM9-SOLID-012 / OM9-SOLID-014: native solid creation sessions.
//! Host decisions: mm, right-handed CPlane, standalone BRep, 1e-7 mm minimum.
use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};
type Point = [f64; 3];
const MIN: f64 = 1e-7;
fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: Point, s: f64) -> Point {
    a.map(|x| x * s)
}
fn dot(a: Point, b: Point) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn finite(p: Point) -> bool {
    p.iter().all(|x| x.is_finite() && x.abs() <= 1e9)
}
fn number(text: &str) -> Result<f64, String> {
    let text = text.trim().to_ascii_lowercase();
    let (text, k) = if let Some(s) = text.strip_suffix("mm") {
        (s, 1.)
    } else if let Some(s) = text.strip_suffix("cm") {
        (s, 10.)
    } else if let Some(s) = text.strip_suffix("in") {
        (s, 25.4)
    } else {
        (text.as_str(), 1.)
    };
    let n = text
        .trim()
        .parse::<f64>()
        .map_err(|_| "Enter a point or a supported dimension".to_owned())?
        * k;
    if !n.is_finite() || n.abs() > 1e9 {
        return Err("Dimension/coordinate must be finite and within 1e9 mm".into());
    }
    Ok(n)
}
fn dimension(n: f64) -> Result<f64, String> {
    if !n.is_finite() || n.abs() < MIN || n.abs() > 1e9 {
        Err("Dimension must be nonzero and within 1e9 mm".into())
    } else {
        Ok(n)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Kind {
    Box = 1,
    Sphere = 2,
}
impl Kind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name
            .trim()
            .trim_start_matches('_')
            .to_ascii_lowercase()
            .as_str()
        {
            "box" | "solidboxcornertocornerheight" | "om9_solidboxcornertocornerheight" => {
                Some(Self::Box)
            }
            "sphere" | "solidspherecenterradius" | "om9_solidspherecenterradius" => {
                Some(Self::Sphere)
            }
            _ => None,
        }
    }
    pub fn caption(self) -> &'static str {
        if self == Self::Box { "Box" } else { "Sphere" }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Waiting,
    Commit,
    Cancelled,
}
#[derive(Clone)]
struct Data {
    phase: u32,
    first: Point,
    last: Point,
    axes: [Point; 3],
    length: f64,
    width: f64,
    three: bool,
    output: Option<[f64; 15]>,
}
#[derive(Clone)]
pub struct Session {
    kind: Kind,
    data: Data,
    frame: (Point, [Point; 3]),
    history: Vec<Data>,
}
impl Session {
    pub fn new(kind: Kind) -> Self {
        let axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        Self {
            kind,
            data: Data {
                phase: 1,
                first: [0.; 3],
                last: [0.; 3],
                axes,
                length: 0.,
                width: 0.,
                three: false,
                output: None,
            },
            frame: ([0.; 3], axes),
            history: Vec::new(),
        }
    }
    pub fn phase(&self) -> u32 {
        self.data.phase
    }
    pub fn output(&self) -> Option<[f64; 15]> {
        self.data.output
    }
    pub fn set_frame(&mut self, origin: Point, axes: [Point; 3]) -> Result<(), String> {
        if !finite(origin) || axes.iter().flatten().any(|x| !x.is_finite()) {
            return Err("Invalid CPlane".into());
        }
        for i in 0..3 {
            for j in 0..3 {
                if (dot(axes[i], axes[j]) - if i == j { 1. } else { 0. }).abs() > 1e-6 {
                    return Err("CPlane must be orthonormal".into());
                }
            }
        }
        if dot(cross(axes[0], axes[1]), axes[2]) < 0.999999 {
            return Err("CPlane must be right handed".into());
        }
        self.frame = (origin, axes);
        Ok(())
    }
    pub fn prompt(&self) -> String {
        let action = match self.phase() {
            1 => {
                if self.kind == Kind::Box {
                    "First corner (3Point / Esc)"
                } else {
                    "Center (Esc)"
                }
            }
            2 => {
                if self.data.three {
                    "End of first edge or Length"
                } else {
                    "Other base corner or Length"
                }
            }
            3 => "Width (Enter = Length)",
            4 => "Height (Enter = Width)",
            5 => "Radius (pick a point or enter a positive value)",
            6 => "Ready to create solid",
            _ => "Command:",
        };
        format!("{}: {}", self.kind.caption(), action)
    }
    pub fn input(&mut self, text: &str) -> Result<Effect, String> {
        if self.phase() == 0 || self.phase() == 6 {
            return Err("No editable solid command".into());
        }
        let before = self.data.clone();
        match self.input_inner(text.trim()) {
            Ok(effect) => {
                if effect == Effect::Waiting
                    && self.data.phase != 0
                    && !text.trim().eq_ignore_ascii_case("undo")
                {
                    self.history.push(before);
                }
                Ok(effect)
            }
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    fn input_inner(&mut self, text: &str) -> Result<Effect, String> {
        if text.eq_ignore_ascii_case("cancel") || text.eq_ignore_ascii_case("esc") {
            self.cancel();
            return Ok(Effect::Cancelled);
        }
        if text.eq_ignore_ascii_case("undo") {
            self.data = self.history.pop().ok_or("No input to undo")?;
            return Ok(Effect::Waiting);
        }
        if text.eq_ignore_ascii_case("3point") || text.eq_ignore_ascii_case("3p") {
            if self.kind != Kind::Box || self.phase() != 1 {
                return Err("3Point is available before the first Box point".into());
            }
            self.data.three = true;
            return Ok(Effect::Waiting);
        }
        if text.is_empty() {
            return match self.phase() {
                3 => self.numeric(self.data.length),
                4 => self.numeric(self.data.width.abs()),
                _ => Err("Enter a point or dimension before confirming".into()),
            };
        }
        if let Some((option, value)) = text.split_once('=') {
            let allowed = match self.phase() {
                2 => "Length",
                3 => "Width",
                4 => "Height",
                5 => "Radius",
                _ => "",
            };
            if allowed.is_empty() || !option.eq_ignore_ascii_case(allowed) {
                return Err("Option is unavailable at this step".into());
            }
            return self.numeric(number(value)?);
        }
        if !text.contains(',') {
            return self.numeric(number(text)?);
        }
        let (relative, text) = if text.starts_with('r') || text.starts_with('R') {
            (true, &text[1..])
        } else {
            (false, text)
        };
        let values = text.split(',').map(number).collect::<Result<Vec<_>, _>>()?;
        if !(2..=3).contains(&values.len()) {
            return Err("Enter x,y or x,y,z".into());
        }
        if relative && self.phase() == 1 {
            return Err("Relative input requires a previous point".into());
        }
        let mut p = if relative {
            self.data.last
        } else {
            self.frame.0
        };
        for (i, v) in values.iter().enumerate() {
            p = add(p, scale(self.frame.1[i], *v));
        }
        self.point_inner(p)
    }
    fn numeric(&mut self, n: f64) -> Result<Effect, String> {
        let n = dimension(n)?;
        match self.phase() {
            2 => {
                if n < 0. {
                    return Err("Length must be positive".into());
                }
                self.data.length = n;
                self.data.last = add(self.data.first, scale(self.data.axes[0], n));
                self.data.phase = 3;
                Ok(Effect::Waiting)
            }
            3 => {
                self.data.width = n;
                self.data.phase = 4;
                Ok(Effect::Waiting)
            }
            4 => self.finish_box(n),
            5 => {
                if n < 0. {
                    return Err("Radius must be positive".into());
                }
                self.finish_sphere(n)
            }
            _ => Err("Pick or enter the first point".into()),
        }
    }
    pub fn point(&mut self, p: Point) -> Result<Effect, String> {
        let before = self.data.clone();
        match self.point_inner(p) {
            Ok(e) => {
                if e == Effect::Waiting {
                    self.history.push(before);
                }
                Ok(e)
            }
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    fn point_inner(&mut self, p: Point) -> Result<Effect, String> {
        if !finite(p) {
            return Err("Invalid point coordinates".into());
        }
        match self.phase() {
            1 => {
                self.data.first = p;
                self.data.last = p;
                self.data.axes = self.frame.1;
                self.data.phase = if self.kind == Kind::Sphere { 5 } else { 2 };
                Ok(Effect::Waiting)
            }
            2 => {
                let d = sub(p, self.data.first);
                let z = self.data.axes[2];
                if self.data.three {
                    let edge = sub(d, scale(z, dot(d, z)));
                    let length = dimension(dot(edge, edge).sqrt())?;
                    self.data.axes[0] = scale(edge, 1. / length);
                    self.data.axes[1] = cross(z, self.data.axes[0]);
                    self.data.length = length;
                    self.data.phase = 3;
                } else {
                    self.data.length = dimension(dot(d, self.data.axes[0]))?;
                    self.data.width = dimension(dot(d, self.data.axes[1]))?;
                    self.data.phase = 4;
                }
                self.data.last = p;
                Ok(Effect::Waiting)
            }
            3 => {
                let effect = self.numeric(dot(sub(p, self.data.first), self.data.axes[1]))?;
                self.data.last = p;
                Ok(effect)
            }
            4 => self.finish_box(dot(sub(p, self.data.first), self.data.axes[2])),
            5 => self.finish_sphere(dot(sub(p, self.data.first), sub(p, self.data.first)).sqrt()),
            _ => Err("No editable solid command".into()),
        }
    }
    fn finish_box(&mut self, height: f64) -> Result<Effect, String> {
        let dims = [
            dimension(self.data.length)?,
            dimension(self.data.width)?,
            dimension(height)?,
        ];
        let mut origin = self.data.first;
        for (i, &n) in dims.iter().enumerate() {
            if n < 0. {
                origin = add(origin, scale(self.data.axes[i], n));
            }
        }
        let mut b = [0.; 15];
        b[..3].copy_from_slice(&origin);
        for i in 0..3 {
            b[3 + i * 3..6 + i * 3].copy_from_slice(&self.data.axes[i]);
            b[12 + i] = dims[i].abs();
        }
        // Check all extrema, not only the nominal origin.
        for bits in 0..8 {
            let mut p = origin;
            for i in 0..3 {
                if bits & (1 << i) != 0 {
                    p = add(p, scale(self.data.axes[i], b[12 + i]));
                }
            }
            if !finite(p) {
                return Err("Box exceeds supported coordinate range".into());
            }
        }
        self.data.output = Some(b);
        self.data.phase = 6;
        Ok(Effect::Commit)
    }
    fn finish_sphere(&mut self, radius: f64) -> Result<Effect, String> {
        let radius = dimension(radius)?;
        if radius < 0. || self.data.first.iter().any(|x| x.abs() + radius > 1e9) {
            return Err("Invalid sphere radius or coordinate range".into());
        }
        let mut b = [0.; 15];
        b[..3].copy_from_slice(&self.data.first);
        for i in 0..3 {
            b[3 + i * 3..6 + i * 3].copy_from_slice(&self.data.axes[i]);
        }
        b[12] = radius;
        self.data.output = Some(b);
        self.data.phase = 6;
        Ok(Effect::Commit)
    }
    pub fn preview(&self, p: Point, ortho: bool) -> Option<[f64; 15]> {
        let mut candidate = self.clone();
        candidate.point(candidate.constrain(p, ortho)).ok()?;
        candidate.output()
    }
    pub fn constrain(&self, p: Point, ortho: bool) -> Point {
        if !ortho || self.phase() != 2 || !self.data.three {
            return p;
        }
        let delta = sub(p, self.data.first);
        let x = dot(delta, self.data.axes[0]);
        let y = dot(delta, self.data.axes[1]);
        add(
            self.data.first,
            if x.abs() >= y.abs() {
                scale(self.data.axes[0], x)
            } else {
                scale(self.data.axes[1], y)
            },
        )
    }
    pub fn cancel(&mut self) {
        self.data.phase = 0;
        self.data.output = None;
        self.history.clear();
    }
}
#[derive(Default)]
struct State {
    session: Option<Session>,
    message: String,
}
fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::default()))
}
fn string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .ok()
            .filter(|s| s.len() <= 1024)
            .map(str::to_owned)
    }
}
fn change(f: impl FnOnce(&mut Session) -> Result<Effect, String>) -> u32 {
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    let result = state
        .session
        .as_mut()
        .ok_or("No active solid command".to_owned())
        .and_then(f);
    state.message = match &result {
        Ok(_) => state
            .session
            .as_ref()
            .map_or(String::new(), Session::prompt),
        Err(e) => format!(
            "{e}. {}",
            state
                .session
                .as_ref()
                .map_or(String::new(), Session::prompt)
        ),
    };
    match result {
        Ok(Effect::Waiting) => 1,
        Ok(Effect::Commit) => 2,
        Ok(Effect::Cancelled) => 3,
        Err(_) => 0,
    }
}
/// # Safety
/// Non-null input is a live NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_kind(p: *const c_char) -> u32 {
    string(p)
        .and_then(|s| Kind::from_name(&s))
        .map_or(0, |k| k as u32)
}
/// # Safety
/// Non-null input is a live NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_start(p: *const c_char) -> bool {
    let Some(kind) = string(p).and_then(|s| Kind::from_name(&s)) else {
        return false;
    };
    let mut state = state().lock().unwrap_or_else(|p| p.into_inner());
    let session = Session::new(kind);
    state.message = session.prompt();
    state.session = Some(session);
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_cancel() {
    *state().lock().unwrap_or_else(|p| p.into_inner()) = State::default();
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_phase() -> u32 {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, Session::phase)
}
/// # Safety
/// Non-null input is a live NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_input(p: *const c_char) -> u32 {
    let Some(text) = string(p) else {
        return 0;
    };
    change(|s| s.input(&text))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_point(x: f64, y: f64, z: f64, ortho: bool) -> u32 {
    change(|s| s.point(s.constrain([x, y, z], ortho)))
}
/// # Safety
/// Non-null frame addresses 12 doubles: origin, X, Y, Z.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_frame(p: *const f64) -> bool {
    if p.is_null() {
        return false;
    }
    let p = unsafe { std::slice::from_raw_parts(p, 12) };
    let v = |i| [p[i], p[i + 1], p[i + 2]];
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_mut()
        .is_some_and(|s| s.set_frame(v(0), [v(3), v(6), v(9)]).is_ok())
}
/// # Safety
/// Non-null output addresses 15 writable doubles. Preview uses a copied session.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_geometry(
    p: *mut f64,
    preview: bool,
    x: f64,
    y: f64,
    z: f64,
    ortho: bool,
) -> bool {
    if p.is_null() {
        return false;
    }
    let state = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &state.session else {
        return false;
    };
    let b = if preview {
        s.preview([x, y, z], ortho)
    } else {
        s.output()
    };
    let Some(b) = b else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(b.as_ptr(), p, 15);
    }
    true
}
/// # Safety
/// Non-null output addresses 6 writable doubles: height anchor and normal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_height_axis(p: *mut f64) -> bool {
    if p.is_null() {
        return false;
    }
    let state = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &state.session else {
        return false;
    };
    if s.phase() != 4 {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.data.first.as_ptr(), p, 3);
        std::ptr::copy_nonoverlapping(s.data.axes[2].as_ptr(), p.add(3), 3);
    }
    true
}
/// # Safety
/// Non-null buffer must be writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_message(p: *mut c_char, capacity: usize) -> usize {
    let state = state().lock().unwrap_or_else(|p| p.into_inner());
    let b = state.message.as_bytes();
    if !p.is_null() && capacity > 0 {
        let n = b.len().min(capacity - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(b.as_ptr(), p.cast(), n);
            *p.add(n) = 0;
        }
    }
    b.len()
}
