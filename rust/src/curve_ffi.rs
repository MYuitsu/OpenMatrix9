use crate::curve::{CurveSession, Effect};
use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};
#[derive(Default)]
struct State {
    session: CurveSession,
    message: String,
    outline: Vec<[f64; 3]>,
}
fn state() -> &'static Mutex<State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(State::default()))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_input_units(mm_per_unit: f64) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|mut s| s.session.set_input_scale(mm_per_unit).is_ok())
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
/// output points to eleven writable doubles: center, major direction, normal,
/// major radius, minor radius. Returns false without writing when unavailable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_ellipse_plan(output: *mut f64) -> bool {
    if output.is_null() {
        return false;
    }
    let Ok(s) = state().lock() else {
        return false;
    };
    let Some(p) = s.session.ellipse() else {
        return false;
    };
    let flat = [
        p.center[0],
        p.center[1],
        p.center[2],
        p.major_direction[0],
        p.major_direction[1],
        p.major_direction[2],
        p.normal[0],
        p.normal[1],
        p.normal[2],
        p.major,
        p.minor,
    ];
    unsafe {
        std::ptr::copy_nonoverlapping(flat.as_ptr(), output, 11);
    }
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ellipse_deformable() -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|s| s.session.ellipse_deformable())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ellipse_mark_foci() -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|s| s.session.ellipse_mark_foci())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ellipse_deviation() -> f64 {
    state()
        .lock()
        .ok()
        .map_or(f64::NAN, |s| s.session.ellipse_deviation())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ellipse_reference_mode() -> u32 {
    state()
        .lock()
        .ok()
        .map_or(0, |s| s.session.ellipse_reference_mode())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_ellipse_reference(
    x: f64,
    y: f64,
    z: f64,
    nx: f64,
    ny: f64,
    nz: f64,
    kind: u32,
) -> u32 {
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    let r = s
        .session
        .ellipse_native_reference([x, y, z], [nx, ny, nz], kind);
    effect(&mut s, r)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_reference_mode() -> u32 {
    state()
        .lock()
        .ok()
        .map_or(0, |s| s.session.circle_reference_mode())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_reference(
    x: f64,
    y: f64,
    z: f64,
    nx: f64,
    ny: f64,
    nz: f64,
    kind: u32,
) -> u32 {
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    let r = s
        .session
        .circle_native_reference([x, y, z], [nx, ny, nz], kind);
    effect(&mut s, r)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_solution_accept(x: f64, y: f64, z: f64, radius: f64) -> u32 {
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    let r = s.session.circle_accept_solution([x, y, z], radius);
    effect(&mut s, r)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_solution_accept_normal(
    x: f64,
    y: f64,
    z: f64,
    nx: f64,
    ny: f64,
    nz: f64,
    radius: f64,
) -> u32 {
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    let r = s
        .session
        .circle_accept_spatial_solution([x, y, z], [nx, ny, nz], radius);
    effect(&mut s, r)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_history() -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|s| s.session.circle_history())
}
/// # Safety
/// config is null or 24 writable doubles; points is null or capacity*3 writable
/// doubles. Capacity <=1024. Null buffers query the count. Buffers do not alias.
/// Returns usize::MAX on unavailable data/insufficient capacity. Snapshot includes
/// the committed session until explicit cancel/start; no borrowed state escapes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_history_snapshot(
    config: *mut f64,
    points: *mut f64,
    capacity: usize,
) -> usize {
    let Ok(s) = state().lock() else {
        return usize::MAX;
    };
    if s.session.name() != "Circle" || capacity > 1024 {
        return usize::MAX;
    }
    let picks = s.session.points();
    if !points.is_null() && capacity < picks.len() {
        return usize::MAX;
    }
    let numeric = if s.session.circle_options.center_mode() && picks.len() == 1 {
        s.session.circle().map(|p| p.radius)
    } else {
        None
    };
    let recipe = crate::circle_history::Recipe::new(&s.session.circle_options, numeric).encode();
    unsafe {
        if !config.is_null() {
            std::ptr::copy_nonoverlapping(recipe.as_ptr(), config, 24);
        }
        if !points.is_null() {
            for (i, p) in picks.iter().enumerate() {
                std::ptr::copy_nonoverlapping(p.as_ptr(), points.add(i * 3), 3);
            }
        }
    }
    picks.len()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_tangent_vertical() -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|s| s.session.circle_options.tangent_vertical)
}
/// # Safety
/// `points` has count*3 readable doubles (1..30003), `frame` has 12 readable
/// doubles (origin/axes); `output` has 12 writable doubles. Buffers do not alias.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_tangent_frame(
    points: *const f64,
    count: usize,
    frame: *const f64,
    output: *mut f64,
) -> bool {
    if points.is_null() || frame.is_null() || output.is_null() || count == 0 || count > 30003 {
        return false;
    }
    let p = unsafe { std::slice::from_raw_parts(points, count * 3) }
        .as_chunks::<3>()
        .0;
    let f = unsafe { std::slice::from_raw_parts(frame, 12) }
        .as_chunks::<3>()
        .0;
    match crate::circle::tangent_frame(p, f[0], [f[1], f[2], f[3]]) {
        Ok(axes) => {
            unsafe {
                std::ptr::copy_nonoverlapping(frame, output, 3);
                for (i, a) in axes.iter().enumerate() {
                    std::ptr::copy_nonoverlapping(a.as_ptr(), output.add(3 + i * 3), 3);
                }
            };
            true
        }
        Err(e) => {
            if let Ok(mut s) = state().lock() {
                s.message = format!("{e}\n{}", s.session.prompt());
            }
            false
        }
    }
}
/// # Safety
/// `input` has seven readable doubles center/normal/radius; `output` has seven
/// writable doubles. No UI state changes; suitable for native History recompute.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_validate_plan(input: *const f64, output: *mut f64) -> bool {
    if input.is_null() || output.is_null() {
        return false;
    }
    let p = unsafe { std::slice::from_raw_parts(input, 7) };
    let Ok(plan) = crate::circle::Plan::new([p[0], p[1], p[2]], [p[3], p[4], p[5]], p[6]) else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(plan.center.as_ptr(), output, 3);
        std::ptr::copy_nonoverlapping(plan.normal.as_ptr(), output.add(3), 3);
        *output.add(6) = plan.radius;
    }
    true
}
/// # Safety
/// `points` addresses `count * 3` readable doubles, count <= 1024.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_fit_batch(points: *const f64, count: usize) -> u32 {
    if points.is_null() || count == 0 || count > 1024 {
        return 0;
    }
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    let input: Vec<_> = unsafe { std::slice::from_raw_parts(points, count * 3) }
        .as_chunks::<3>()
        .0
        .to_vec();
    let r = s.session.circle_fit_batch(&input);
    effect(&mut s, r)
}
/// # Safety
/// `output` is null or addresses twelve writable doubles (three XYZ/free records).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_constraints(output: *mut f64) -> usize {
    let Ok(s) = state().lock() else {
        return 0;
    };
    let c = &s.session.circle_options.constraints;
    if !output.is_null() {
        for (i, (p, free)) in c.iter().enumerate() {
            unsafe {
                std::ptr::copy_nonoverlapping(p.as_ptr(), output.add(i * 4), 3);
                *output.add(i * 4 + 3) = f64::from(*free);
            }
        }
    }
    c.len()
}
/// # Safety
/// `output` addresses thirteen writable doubles: origin, axes X/Y/Z, radius constraint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_frame(output: *mut f64) -> bool {
    if output.is_null() {
        return false;
    }
    let Ok(s) = state().lock() else {
        return false;
    };
    let (origin, axes) = s.session.circle_solver_frame();
    unsafe {
        std::ptr::copy_nonoverlapping(origin.as_ptr(), output, 3);
        for (i, axis) in axes.iter().enumerate() {
            std::ptr::copy_nonoverlapping(axis.as_ptr(), output.add(3 + i * 3), 3);
        }
        *output.add(12) = s.session.circle_options.radius_constraint.unwrap_or(0.);
    }
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_from_first() -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|s| s.session.circle_options.from_first)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_solution_index() -> i32 {
    state()
        .lock()
        .ok()
        .and_then(|s| s.session.circle_options.solution)
        .map_or(-1, |n| n as i32)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_deformable() -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|s| s.session.circle_options.deformable)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_deviation(approx: bool) -> f64 {
    state().lock().ok().map_or(f64::NAN, |s| {
        let (fit, ap) = s.session.circle_deviations();
        if approx { ap } else { fit }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_circle_construction() -> u32 {
    state()
        .lock()
        .ok()
        .map_or(0, |s| match s.session.circle_options.mode {
            crate::circle::Mode::AroundCurve => 5,
            crate::circle::Mode::FitPoints => 6,
            crate::circle::Mode::Tangent => 7,
            _ => 0,
        })
}
/// # Safety
/// `hover` addresses three readable XYZ doubles; `output` addresses four
/// writable doubles: radius/diameter/circumference in mm, area in mm squared.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_hover_measure(hover: *const f64, output: *mut f64) -> bool {
    if hover.is_null() || output.is_null() {
        return false;
    }
    let Ok(s) = state().lock() else {
        return false;
    };
    let p = unsafe { std::slice::from_raw_parts(hover, 3) };
    let Some(plan) = s.session.circle_hover_plan([p[0], p[1], p[2]]) else {
        return false;
    };
    let r = plan.radius;
    let values = [
        r,
        2. * r,
        std::f64::consts::TAU * r,
        std::f64::consts::PI * r * r,
    ];
    unsafe {
        std::ptr::copy_nonoverlapping(values.as_ptr(), output, 4);
    }
    true
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
    let active = if s.session.name() == "Rectangle" {
        shift
    } else {
        crate::core_keyboard::om9_ortho_active(shift)
    };
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
/// # Safety
/// `output` points to seven writable doubles: center XYZ, normal XYZ, radius.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_circle_plan(output: *mut f64) -> bool {
    if output.is_null() {
        return false;
    }
    let Ok(s) = state().lock() else {
        return false;
    };
    let Some(plan) = s.session.circle() else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(plan.center.as_ptr(), output, 3);
        std::ptr::copy_nonoverlapping(plan.normal.as_ptr(), output.add(3), 3);
        *output.add(6) = plan.radius;
    }
    true
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
/// `hover` is null or points to three readable world coordinates.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_outline(hover: *const f64, square: bool) -> usize {
    let Ok(mut s) = state().lock() else {
        return 0;
    };
    s.outline.clear();
    if !s.session.active() {
        return 0;
    }
    s.outline = if hover.is_null() {
        s.session.points().to_vec()
    } else {
        let p = unsafe { std::slice::from_raw_parts(hover, 3) };
        s.session
            .preview_geometry([p[0], p[1], p[2]], square)
            .unwrap_or_default()
    };
    s.outline.len()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_outline_coordinate(point: usize, axis: usize) -> f64 {
    state()
        .lock()
        .ok()
        .and_then(|s| s.outline.get(point).and_then(|p| p.get(axis)).copied())
        .unwrap_or(f64::NAN)
}
/// # Safety
/// `output` points to six writable doubles: origin XYZ, normal XYZ.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_curve_pick_plane(output: *mut f64) -> bool {
    if output.is_null() {
        return false;
    }
    let Ok(s) = state().lock() else {
        return false;
    };
    let Some((origin, normal)) = s.session.rectangle_pick_plane() else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(origin.as_ptr(), output, 3);
        std::ptr::copy_nonoverlapping(normal.as_ptr(), output.add(3), 3);
    }
    true
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
