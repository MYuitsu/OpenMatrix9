//! OM9-VIEW-003/004/008/009 shared viewport behavior.
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Window,
    Dynamic,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Waiting,
    Window([f64; 4]),
    Scale(f64),
    Dynamic,
}
pub struct Drag {
    tool: Tool,
    active: bool,
    anchor: Option<[f64; 2]>,
    last: Option<[f64; 2]>,
}
fn valid(point: [f64; 2]) -> bool {
    point.iter().all(|v| v.is_finite() && v.abs() <= 1e6)
}
impl Drag {
    pub fn new(tool: Tool) -> Self {
        Self {
            tool,
            active: true,
            anchor: None,
            last: None,
        }
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn cancel(&mut self) {
        self.active = false;
        self.anchor = None;
        self.last = None;
    }
    pub fn press(&mut self, point: [f64; 2]) -> Result<(), &'static str> {
        if !self.active || !valid(point) {
            return Err("Invalid viewport point");
        }
        self.anchor = Some(point);
        self.last = Some(point);
        Ok(())
    }
    pub fn motion(&mut self, point: [f64; 2], height: f64) -> Result<Outcome, &'static str> {
        if !self.active || !valid(point) || !height.is_finite() || height <= 0.0 {
            return Err("Invalid viewport motion");
        }
        let Some(anchor) = self.anchor else {
            return Ok(Outcome::Waiting);
        };
        let value = match self.tool {
            Tool::Window => Outcome::Window(rect(anchor, point)),
            Tool::Dynamic => Outcome::Scale(
                ((point[1] - self.last.unwrap_or(anchor)[1]) / height)
                    .clamp(-4.0, 4.0)
                    .exp2(),
            ),
        };
        self.last = Some(point);
        Ok(value)
    }
    pub fn release(&mut self, point: [f64; 2]) -> Outcome {
        if !self.active || !valid(point) {
            return Outcome::Waiting;
        }
        let Some(anchor) = self.anchor.take() else {
            return Outcome::Waiting;
        };
        self.last = None;
        match self.tool {
            Tool::Window => {
                let bounds = rect(anchor, point);
                if bounds[2] - bounds[0] < 3.0 || bounds[3] - bounds[1] < 3.0 {
                    return Outcome::Waiting;
                }
                self.active = false;
                Outcome::Window(bounds)
            }
            Tool::Dynamic => {
                self.active = false;
                Outcome::Dynamic
            }
        }
    }
}
fn rect(a: [f64; 2], b: [f64; 2]) -> [f64; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[0].max(b[0]),
        a[1].max(b[1]),
    ]
}
pub fn apparent_height(distance: f64, angle: f64) -> Option<f64> {
    if !distance.is_finite()
        || distance <= 0.0
        || !angle.is_finite()
        || angle <= 0.0
        || angle >= std::f64::consts::PI
    {
        return None;
    }
    let value = 2.0 * distance * (angle / 2.0).tan();
    (value.is_finite() && value > 0.0 && value <= 1e9).then_some(value)
}
pub fn scaled_value(perspective: bool, value: f64, factor: f64) -> Option<f64> {
    if !value.is_finite() || value <= 0.0 || !factor.is_finite() || factor <= 0.0 {
        return None;
    }
    if perspective {
        if value >= std::f64::consts::PI {
            return None;
        }
        Some((2.0 * ((value / 2.0).tan() * factor).atan()).clamp(1e-6, std::f64::consts::PI - 1e-4))
    } else {
        Some((value * factor).clamp(1e-7, 1e9))
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_view_scaled_value(perspective: bool, value: f64, factor: f64) -> f64 {
    scaled_value(perspective, value, factor).unwrap_or(f64::NAN)
}
struct State {
    drag: Option<Drag>,
    output: [f64; 4],
}
fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(State {
            drag: None,
            output: [0.0; 4],
        })
    })
}
fn effect(state: &mut State, outcome: Outcome) -> u32 {
    match outcome {
        Outcome::Waiting => 0,
        Outcome::Window(bounds) => {
            state.output = bounds;
            1
        }
        Outcome::Scale(factor) => {
            state.output = [factor, 0.0, 0.0, 0.0];
            2
        }
        Outcome::Dynamic => 3,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_perspective_height(distance: f64, angle: f64) -> f64 {
    apparent_height(distance, angle).unwrap_or(f64::NAN)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_view_tool_start(kind: u32) -> bool {
    let tool = match kind {
        1 => Tool::Window,
        2 => Tool::Dynamic,
        _ => return false,
    };
    let Ok(mut state) = state().lock() else {
        return false;
    };
    state.drag = Some(Drag::new(tool));
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_view_tool_press(x: f64, y: f64) -> bool {
    let Ok(mut state) = state().lock() else {
        return false;
    };
    state.drag.as_mut().is_some_and(|s| s.press([x, y]).is_ok())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_view_tool_motion(x: f64, y: f64, height: f64) -> u32 {
    let Ok(mut state) = state().lock() else {
        return 0;
    };
    let outcome = state
        .drag
        .as_mut()
        .and_then(|s| s.motion([x, y], height).ok())
        .unwrap_or(Outcome::Waiting);
    effect(&mut state, outcome)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_view_tool_release(x: f64, y: f64) -> u32 {
    let Ok(mut state) = state().lock() else {
        return 0;
    };
    let outcome = state
        .drag
        .as_mut()
        .map_or(Outcome::Waiting, |s| s.release([x, y]));
    effect(&mut state, outcome)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_view_tool_value(axis: usize) -> f64 {
    state()
        .lock()
        .ok()
        .and_then(|s| s.output.get(axis).copied())
        .unwrap_or(f64::NAN)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_view_tool_cancel() {
    if let Ok(mut state) = state().lock() {
        if let Some(drag) = state.drag.as_mut() {
            drag.cancel();
        }
        state.drag = None;
    }
}
static CROSSHAIRS: AtomicBool = AtomicBool::new(false);
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_crosshairs() -> bool {
    CROSSHAIRS.load(Ordering::Relaxed)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_crosshairs_toggle() -> bool {
    !CROSSHAIRS.fetch_xor(true, Ordering::Relaxed)
}
