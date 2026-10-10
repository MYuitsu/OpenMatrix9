use crate::curve::{CurveSession, Effect};
use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};
#[derive(Default)]
struct State {
    session: CurveSession,
    message: String,
}
fn state() -> &'static Mutex<State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(State::default()))
}
fn effect(s: &mut State, result: Result<Effect, String>) -> u32 {
    match result {
        Ok(e) => {
            s.message = s.session.prompt();
            match e {
                Effect::Waiting => 1,
                Effect::Commit => 2,
                Effect::Cancelled => 3,
            }
        }
        Err(message) => {
            s.message = format!("{message}\n{}", s.session.prompt());
            0
        }
    }
}
/// # Safety
/// `text` must point to a valid NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_start(text: *const c_char) -> bool {
    if text.is_null() {
        return false;
    }
    let Ok(text) = unsafe { CStr::from_ptr(text) }.to_str() else {
        return false;
    };
    let Ok(mut s) = state().lock() else {
        return false;
    };
    match s.session.start(text) {
        Ok(()) => {
            s.message = s.session.prompt();
            true
        }
        Err(e) => {
            s.message = e;
            false
        }
    }
}
/// # Safety
/// `text` must point to a valid NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_input(text: *const c_char) -> u32 {
    if text.is_null() {
        return 0;
    }
    let Ok(text) = unsafe { CStr::from_ptr(text) }.to_str() else {
        return 0;
    };
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    if text.len() > 1024 {
        s.message = "Input is too long".into();
        return 0;
    }
    let result = s.session.input(text);
    effect(&mut s, result)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_point(x: f64, y: f64, z: f64) -> u32 {
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    let result = s.session.point([x, y, z]);
    effect(&mut s, result)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_mouse_point(x: f64, y: f64, z: f64, shift: bool) -> u32 {
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    let active = crate::core_keyboard::om9_ortho_active(shift);
    let result = s.session.mouse_point([x, y, z], active);
    effect(&mut s, result)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_frame(
    ox: f64,
    oy: f64,
    oz: f64,
    xx: f64,
    xy: f64,
    xz: f64,
    yx: f64,
    yy: f64,
    yz: f64,
    zx: f64,
    zy: f64,
    zz: f64,
) -> bool {
    let Ok(mut s) = state().lock() else {
        return false;
    };
    s.session
        .set_frame([ox, oy, oz], [[xx, xy, xz], [yx, yy, yz], [zx, zy, zz]])
        .is_ok()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_cancel() {
    if let Ok(mut s) = state().lock() {
        s.session.cancel();
        s.message = s.session.prompt();
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_active() -> bool {
    state().lock().ok().is_some_and(|s| s.session.active())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_count() -> usize {
    state().lock().ok().map_or(0, |s| s.session.output().len())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_preview_count() -> usize {
    state().lock().ok().map_or(0, |s| {
        if s.session.active() {
            s.session.points().len()
        } else {
            0
        }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_preview_coordinate(point: usize, axis: usize) -> f64 {
    state()
        .lock()
        .ok()
        .and_then(|s| {
            s.session
                .points()
                .get(point)
                .and_then(|p| p.get(axis))
                .copied()
        })
        .unwrap_or(f64::NAN)
}
/// # Safety
/// `output` must point to three writable doubles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_preview_point(
    x: f64,
    y: f64,
    z: f64,
    shift: bool,
    output: *mut f64,
) -> bool {
    if output.is_null() {
        return false;
    }
    let Ok(s) = state().lock() else {
        return false;
    };
    let Ok(p) = s
        .session
        .preview_point([x, y, z], crate::core_keyboard::om9_ortho_active(shift))
    else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(p.as_ptr(), output, 3);
    }
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_coordinate(point: usize, axis: usize) -> f64 {
    state()
        .lock()
        .ok()
        .and_then(|s| {
            s.session
                .output()
                .get(point)
                .and_then(|p| p.get(axis))
                .copied()
        })
        .unwrap_or(f64::NAN)
}
/// # Safety
/// `buffer` must be writable for `capacity` bytes, or NULL when capacity is zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_message(buffer: *mut c_char, capacity: usize) -> usize {
    let Ok(s) = state().lock() else {
        return 0;
    };
    let bytes = s.message.as_bytes();
    if capacity > 0 && !buffer.is_null() {
        let count = bytes.len().min(capacity - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.cast::<u8>(), count);
            *buffer.add(count) = 0;
        }
    }
    bytes.len()
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_spline_publish() -> bool {
    let Ok(s) = state().lock() else {
        return false;
    };
    crate::spline_ffi::publish(
        s.session
            .spline()
            .cloned()
            .ok_or("No committed spline".into()),
    )
}
/// # Safety
/// `hover` is null or points to three readable world coordinates.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_preview_spline(hover: *const f64) -> bool {
    unsafe { om9_curve_preview_spline_closed(hover, false) }
}
/// # Safety
/// `hover` is null or points to three readable world coordinates.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_preview_spline_closed(hover: *const f64, close: bool) -> bool {
    let Ok(s) = state().lock() else {
        return false;
    };
    let point = if hover.is_null() {
        None
    } else {
        let p = unsafe { std::slice::from_raw_parts(hover, 3) };
        Some([p[0], p[1], p[2]])
    };
    crate::spline_ffi::publish(s.session.preview_spline_closed(point, close))
}
