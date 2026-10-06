//! Matrix-specific Ctrl-left zoom and documented Rhino 5 viewport gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Action {
    None = 0,
    Select = 1,
    Pan = 2,
    Orbit = 3,
    Zoom = 4,
    Dolly = 5,
    Tilt = 6,
    Look = 7,
    Popup = 8,
}
pub fn action(button: u32, modifiers: u32, perspective: bool) -> Action {
    match (button, modifiers) {
        (1, 2) => Action::Zoom,
        (1, 0..=7) => Action::Select,
        (2, 0) => {
            if perspective {
                Action::Orbit
            } else {
                Action::Pan
            }
        }
        (2, 1) => Action::Pan,
        (2, 2 | 7) => Action::Zoom,
        (2, 3) => {
            if perspective {
                Action::Zoom
            } else {
                Action::Orbit
            }
        }
        (2, 4) => Action::Dolly,
        (2, 5) => Action::Tilt,
        (2, 6) => Action::Look,
        (4, 0) => Action::Popup,
        _ => Action::None,
    }
}
pub fn moved(a: [f64; 2], b: [f64; 2], threshold: f64) -> bool {
    a.iter().chain(b.iter()).all(|v| v.is_finite())
        && threshold.is_finite()
        && threshold >= 0.
        && (a[0] - b[0]).hypot(a[1] - b[1]) >= threshold
}
pub fn selection_hit(rect: [f64; 4], bounds: [f64; 4], crossing: bool) -> bool {
    if !rect.iter().chain(bounds.iter()).all(|v| v.is_finite())
        || rect[0] > rect[2]
        || rect[1] > rect[3]
        || bounds[0] > bounds[2]
        || bounds[1] > bounds[3]
    {
        return false;
    }
    if crossing {
        bounds[0] <= rect[2] && bounds[2] >= rect[0] && bounds[1] <= rect[3] && bounds[3] >= rect[1]
    } else {
        bounds[0] >= rect[0] && bounds[2] <= rect[2] && bounds[1] >= rect[1] && bounds[3] <= rect[3]
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_mouse_action(button: u32, modifiers: u32, perspective: bool) -> u32 {
    action(button, modifiers, perspective) as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_mouse_moved(ax: f64, ay: f64, bx: f64, by: f64, threshold: f64) -> bool {
    moved([ax, ay], [bx, by], threshold)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_mouse_window_contains(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    b3: f64,
) -> bool {
    selection_hit([x0, y0, x1, y1], [b0, b1, b2, b3], false)
}
