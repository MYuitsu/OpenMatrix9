//! Shared O-Snap state/cursor aperture. Native layer owns geometry candidates.
use std::sync::atomic::{AtomicU32, Ordering};
use std::ffi::{CStr, c_char};
pub fn command(icon: &str) -> Option<&'static str> {
    match icon {
        "ToolsObjectSnapEnd" => Some("Osnap E"),
        "ToolsObjectSnapMidpoint" => Some("Osnap M"),
        "ToolsObjectSnapPoint" => Some("Osnap P"),
        _ => None,
    }
}
pub const ICONS: [&str; 3] = [
    "ToolsObjectSnapEnd",
    "ToolsObjectSnapMidpoint",
    "ToolsObjectSnapPoint",
];
static STATE: AtomicU32 = AtomicU32::new(0);
const PERSISTENT: u32 = 15;
const ONE_SHOT: u32 = 14 << 4;
const SUSPENDED: u32 = 1 << 8;

#[unsafe(no_mangle)]
pub extern "C" fn om9_snap_effective_state() -> u32 {
    let value = STATE.load(Ordering::SeqCst);
    if value & SUSPENDED != 0 {
        0
    } else if value & ONE_SHOT != 0 {
        1 | ((value & ONE_SHOT) >> 4)
    } else {
        value & PERSISTENT
    }
}
/// Consume an override only after a point was accepted by its owning session.
/// Hover, failed geometry and a pick while suspended leave it intact.
#[unsafe(no_mangle)]
pub extern "C" fn om9_snap_accept_point(accepted: bool) {
    if accepted {
        let _ = STATE.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
            Some(if value & SUSPENDED == 0 { value & !ONE_SHOT } else { value })
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_snap_transient_clear() {
    STATE.fetch_and(PERSISTENT, Ordering::SeqCst);
}

enum Request {
    Toggle(u32),
    Master(bool),
    Once(u32),
    Only(u32),
    Suspend(bool),
    Clear,
}

fn mode(value: &str) -> Option<u32> {
    match value {
        "e" | "end" => Some(2),
        "m" | "mid" | "midpoint" => Some(4),
        "p" | "point" => Some(8),
        _ => None,
    }
}

fn request(text: &str) -> Result<Option<Request>, ()> {
    let text = text.trim().to_ascii_lowercase();
    let words: Vec<_> = text.split_whitespace().collect();
    if words.first() != Some(&"osnap") {
        return Ok(None);
    }
    let request = match words.as_slice() {
        [_, "on"] => Request::Master(true),
        [_, "off"] => Request::Master(false),
        [_, "toggle"] => Request::Toggle(1),
        [_, "suspend"] => Request::Suspend(true),
        [_, "resume"] => Request::Suspend(false),
        [_, "clear"] => Request::Clear,
        [_, "once", name] => Request::Once(mode(name).ok_or(())?),
        [_, "only", name] => Request::Only(mode(name).ok_or(())?),
        [_, name] => Request::Toggle(mode(name).ok_or(())?),
        _ => return Err(()),
    };
    Ok(Some(request))
}

/// Returns 0 for unrelated input, 1 for a persistent update, 2 for a
/// transient update, and 3 for an unsupported/invalid Osnap request.
/// # Safety
/// Text must be a readable null-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_snap_submit(text: *const c_char) -> u32 {
    if text.is_null() { return 3; }
    let Ok(text) = (unsafe { CStr::from_ptr(text) }).to_str() else { return 3; };
    let request = match request(text) {
        Ok(Some(request)) => request,
        Ok(None) => return 0,
        Err(()) => return 3,
    };
    let transient = matches!(request, Request::Once(_) | Request::Suspend(_));
    let _ = STATE.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
        Some(match request {
            Request::Toggle(bit) => value ^ bit,
            Request::Master(true) => value | 1,
            Request::Master(false) => value & !1,
            Request::Once(mode) => (value & !ONE_SHOT) | (mode << 4),
            Request::Only(mode) => (value & SUSPENDED) | 1 | mode,
            Request::Suspend(true) => value | SUSPENDED,
            Request::Suspend(false) => value & !SUSPENDED,
            Request::Clear => value & (1 | SUSPENDED),
        })
    });
    if transient { 2 } else { 1 }
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_snap_state() -> u32 {
    STATE.load(Ordering::SeqCst) & PERSISTENT
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_snap_load(value: u32) -> bool {
    if State::decode(value).is_none() {
        return false;
    }
    STATE.store(value, Ordering::SeqCst);
    true
}
/// Bit 1 toggles master; bit 2 End; bit 4 Mid; bit 8 Point.
#[unsafe(no_mangle)]
pub extern "C" fn om9_snap_toggle(bit: u32) -> bool {
    if bit != 1 && bit != 2 && bit != 4 && bit != 8 {
        return false;
    }
    STATE.fetch_xor(bit, Ordering::SeqCst);
    true
}

/// Packed candidates: world XYZ then logical screen XY (five doubles each).
/// Returns usize::MAX when no enabled candidate qualifies or input is invalid.
/// # Safety
/// A non-null data pointer must provide count*5 readable doubles for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_snap_end_pick(
    data: *const f64,
    count: usize,
    x: f64,
    y: f64,
    radius: f64,
) -> usize {
    unsafe { om9_snap_mode_pick(data, count, x, y, radius, 2) }
}

/// Select packed candidates for End (2), Mid (4) or Point (8).
/// # Safety
/// A non-null data pointer must provide count*5 readable doubles for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_snap_mode_pick(
    data: *const f64,
    count: usize,
    x: f64,
    y: f64,
    radius: f64,
    mode: u32,
) -> usize {
    let Some(mut state) = State::decode(om9_snap_effective_state()) else {
        return usize::MAX;
    };
    state.end = match mode {
        2 => state.end,
        4 => state.mid,
        8 => state.point,
        _ => return usize::MAX,
    };
    if data.is_null() || count == 0 {
        return usize::MAX;
    }
    let Some(length) = count
        .checked_mul(5)
        .filter(|v| *v <= isize::MAX as usize / std::mem::size_of::<f64>())
    else {
        return usize::MAX;
    };
    let values = unsafe { std::slice::from_raw_parts(data, length) };
    let candidates = values.chunks_exact(5).map(|v| Candidate {
        world: [v[0], v[1], v[2]],
        screen: [v[3], v[4]],
    });
    state
        .pick_iter(candidates, [x, y], radius)
        .unwrap_or(usize::MAX)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub enabled: bool,
    pub end: bool,
    pub mid: bool,
    pub point: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub world: [f64; 3],
    pub screen: [f64; 2],
}
impl State {
    pub fn encoded(self) -> u32 {
        u32::from(self.enabled)
            | (u32::from(self.end) << 1)
            | (u32::from(self.mid) << 2)
            | (u32::from(self.point) << 3)
    }
    pub fn decode(value: u32) -> Option<Self> {
        (value < 16).then_some(Self {
            enabled: value & 1 != 0,
            end: value & 2 != 0,
            mid: value & 4 != 0,
            point: value & 8 != 0,
        })
    }
    pub fn toggle_master(&mut self) {
        self.enabled = !self.enabled;
    }
    pub fn toggle_end(&mut self) {
        self.end = !self.end;
    }
    pub fn toggle_mid(&mut self) {
        self.mid = !self.mid;
    }
    pub fn toggle_point(&mut self) {
        self.point = !self.point;
    }
    pub fn pick(self, candidates: &[Candidate], cursor: [f64; 2], radius: f64) -> Option<usize> {
        self.pick_iter(candidates.iter().copied(), cursor, radius)
    }
    fn pick_iter(
        self,
        candidates: impl IntoIterator<Item = Candidate>,
        cursor: [f64; 2],
        radius: f64,
    ) -> Option<usize> {
        if !self.enabled
            || !self.end
            || cursor.iter().any(|v| !v.is_finite())
            || !radius.is_finite()
            || radius <= 0.
            || radius > 256.
        {
            return None;
        }
        let mut best = None;
        let mut nearest = f64::INFINITY;
        for (index, candidate) in candidates.into_iter().enumerate() {
            if candidate
                .world
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1e9)
                || candidate.screen.iter().any(|v| !v.is_finite())
            {
                continue;
            }
            let distance = (candidate.screen[0] - cursor[0]).hypot(candidate.screen[1] - cursor[1]);
            if distance <= radius && distance < nearest {
                nearest = distance;
                best = Some(index);
            }
        }
        best
    }
}
