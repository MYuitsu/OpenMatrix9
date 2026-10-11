// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9 gumball policy and double-precision transforms; no native handles.
use crate::{core_objects::Stamp, geometry_frame::IDENTITY, units};
use std::collections::{HashMap, HashSet};
pub type Point = [f64; 3];
pub const ICONS: &[&str] = &[
    "InfoSettingsGumballON",
    "InfoSettingsGumballAlignment",
    "InfoSettingsRelocateGumball",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandKind {
    Gumball,
    Alignment,
    Relocate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandInput {
    pub kind: CommandKind,
    pub value: Option<u32>,
}
pub fn parse_command(text: &str) -> Result<Option<CommandInput>, GumballError> {
    let mut words = text.split_whitespace();
    let Some(name) = words.next() else {
        return Ok(None);
    };
    let Some(command) = kind(name) else {
        return Ok(None);
    };
    let value = if let Some(option) = words.next() {
        if words.next().is_some() {
            return Err(GumballError::InvalidInput);
        }
        let option = option.to_ascii_lowercase();
        Some(match (command, option.as_str()) {
            (CommandKind::Gumball, "off") => 0,
            (CommandKind::Gumball, "on") => 1,
            (CommandKind::Gumball, "toggle") => 2,
            (CommandKind::Alignment, "cplane") => 0,
            (CommandKind::Alignment, "object") => 1,
            (CommandKind::Alignment, "world") => 2,
            _ => return Err(GumballError::InvalidInput),
        })
    } else {
        None
    };
    Ok(Some(CommandInput {
        kind: command,
        value,
    }))
}
pub fn command(icon: &str) -> Option<&'static str> {
    match icon {
        "InfoSettingsGumballON" => Some("OM9_Gumball"),
        "InfoSettingsGumballAlignment" => Some("OM9_GumballAlignment"),
        "InfoSettingsRelocateGumball" => Some("OM9_RelocateGumball"),
        _ => None,
    }
}
pub fn kind(name: &str) -> Option<CommandKind> {
    let n = name.trim().trim_start_matches('_');
    let n = n.strip_prefix("OM9_").unwrap_or(n);
    if n.eq_ignore_ascii_case("Gumball") {
        Some(CommandKind::Gumball)
    } else if n.eq_ignore_ascii_case("GumballAlignment") {
        Some(CommandKind::Alignment)
    } else if n.eq_ignore_ascii_case("RelocateGumball") || n.eq_ignore_ascii_case("GumballRelocate")
    {
        Some(CommandKind::Relocate)
    } else {
        None
    }
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    CPlane,
    Object,
    World,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plane {
    XY,
    YZ,
    ZX,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    MoveAxis(Axis),
    MovePlane(Plane),
    MoveFree,
    Rotate(Axis),
    Scale(Axis),
}
impl Handle {
    pub fn code(self) -> u32 {
        match self {
            Self::MoveAxis(a) => a as u32,
            Self::MovePlane(p) => 3 + p as u32,
            Self::MoveFree => 6,
            Self::Rotate(a) => 7 + a as u32,
            Self::Scale(a) => 10 + a as u32,
        }
    }
    pub fn from_code(n: u32) -> Option<Self> {
        let a = |n| match n {
            0 => Axis::X,
            1 => Axis::Y,
            _ => Axis::Z,
        };
        Some(match n {
            0..=2 => Self::MoveAxis(a(n)),
            3 => Self::MovePlane(Plane::XY),
            4 => Self::MovePlane(Plane::YZ),
            5 => Self::MovePlane(Plane::ZX),
            6 => Self::MoveFree,
            7..=9 => Self::Rotate(a(n - 7)),
            10..=12 => Self::Scale(a(n - 10)),
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub origin: Point,
    pub axes: [Point; 3],
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            origin: [0.; 3],
            axes: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        }
    }
}
impl Frame {
    pub fn from_directions(origin: Point, x: Point, normal: Point) -> Result<Self, GumballError> {
        let x = unit(x)?;
        let y = unit(cross(normal, x))?;
        Self::new(origin, [x, y, cross(x, y)])
    }
    pub fn new(origin: Point, axes: [Point; 3]) -> Result<Self, GumballError> {
        let f = Self { origin, axes };
        f.validate()?;
        Ok(f)
    }
    pub fn validate(self) -> Result<(), GumballError> {
        let mut m = IDENTITY;
        for r in 0..3 {
            for c in 0..3 {
                m[4 * r + c] = self.axes[c][r];
            }
            m[4 * r + 3] = self.origin[r];
        }
        crate::geometry_frame::Frame::compose(IDENTITY, IDENTITY, m, IDENTITY)
            .map_err(|_| GumballError::InvalidFrame)?;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub stamp: Stamp,
    pub parent: u64,
    pub world: [f64; 16],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerRay {
    pub origin: Point,
    pub direction: Point,
}
#[derive(Clone, Debug)]
pub struct BeginContext {
    pub frame: Frame,
    pub cplane: Frame,
    pub sources: Vec<Source>,
    pub document: u64,
    pub view_generation: u64,
    pub unit_generation: u64,
    pub mm_per_unit: f64,
    pub handle: Handle,
    pub start_ray: PointerRay,
    pub modifiers: u32,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModifierKey {
    Shift = 1,
    Ctrl = 2,
    Alt = 4,
}
#[derive(Clone, Copy, Debug)]
pub struct ModifierEvent {
    pub key: ModifierKey,
    pub pressed: bool,
    pub auto_repeat: bool,
}
#[derive(Clone, Debug)]
pub enum Event {
    Enable(bool),
    Align(Alignment),
    ObserveFrame(Frame, Frame),
    ObserveOwner(Option<Stamp>, Frame, Frame),
    Begin(BeginContext),
    Motion(PointerRay),
    Modifier(ModifierEvent),
    Numeric(String),
    Accept,
    Release,
    Undo,
    Cancel,
    BeginRelocate,
    RelocatePoint(Point),
    RelocateRay(PointerRay),
    CommitSucceeded,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Off,
    Ready,
    Pressed,
    Dragging,
    Relocate,
    Commit,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Move,
    Rotate,
    Scale,
    Extrude,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    Preview,
    Commit,
    FrameChanged,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GumballError {
    InvalidFrame,
    InvalidInput,
    InvalidSources,
    InvalidState,
    Projection,
    Limit,
}
impl std::fmt::Display for GumballError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidFrame => "Gumball frame is unresolved or invalid",
                Self::InvalidInput => "Invalid gumball value or dimension",
                Self::InvalidSources => "Gumball selection is invalid or stale",
                Self::InvalidState => "No compatible gumball gesture is active",
                Self::Projection =>
                    "Handle axis is parallel to the view; use numeric input or change view",
                Self::Limit => "Gumball input exceeds the session budget",
            }
        )
    }
}
impl std::error::Error for GumballError {}
#[derive(Clone, Debug, PartialEq)]
pub struct TransformPlan {
    pub matrix: [f64; 16],
    pub operation: Operation,
    pub copy: bool,
    pub sources: Vec<Source>,
    pub generation: u64,
    pub provenance: Option<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub enabled: bool,
    pub phase: Phase,
    pub alignment: Alignment,
    pub frame: Frame,
    pub handle: Option<Handle>,
    pub copy: bool,
    pub constraint: Option<f64>,
    pub generation: u64,
    pub preview: Option<TransformPlan>,
}
#[derive(Clone)]
struct Value {
    amount: f64,
    axis_direction: f64,
    vector: Point,
    constraint: Option<f64>,
    numeric: Option<String>,
}
#[derive(Clone)]
struct Gesture {
    context: BeginContext,
    start: Point,
    value: Value,
    history: Vec<Value>,
    modifiers: u32,
    copy: bool,
    extrude: bool,
    relocate: bool,
}
#[derive(Clone)]
pub struct GumballSession {
    enabled: bool,
    phase: Phase,
    alignment: Alignment,
    frame: Frame,
    cplane: Frame,
    generation: u64,
    gesture: Option<Gesture>,
    relocate: Vec<Point>,
    old_frame: Frame,
    owner: Option<Stamp>,
    owner_frames: HashMap<(u64,u64,u64,u64),(Stamp,Frame)>,
}
impl Default for GumballSession {
    fn default() -> Self {
        Self::new()
    }
}
impl GumballSession {
    pub fn new() -> Self {
        Self {
            enabled: false,
            phase: Phase::Off,
            alignment: Alignment::CPlane,
            frame: Frame::default(),
            cplane: Frame::default(),
            generation: 1,
            gesture: None,
            relocate: Vec::new(),
            old_frame: Frame::default(),
            owner: None,
            owner_frames: HashMap::new(),
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            enabled: self.enabled,
            phase: self.phase,
            alignment: self.alignment,
            frame: self.frame,
            handle: self.gesture.as_ref().map(|g| g.context.handle),
            copy: self.gesture.as_ref().is_some_and(|g| g.copy),
            constraint: self.gesture.as_ref().and_then(|g| g.value.constraint),
            generation: self.generation,
            preview: self.plan().ok(),
        }
    }
    /// Errors are atomic, including checkpoints and accepted input provenance.
    pub fn apply(&mut self, event: Event) -> Result<Effect, GumballError> {
        let mut next = self.clone();
        let effect = next.update(event)?;
        *self = next;
        Ok(effect)
    }
    fn update(&mut self, event: Event) -> Result<Effect, GumballError> {
        match event {
            Event::Enable(on) => {
                self.cancel();
                self.enabled = on;
                self.phase = if on { Phase::Ready } else { Phase::Off };
            }
            Event::Align(a) => {
                if self.gesture.is_some() || self.phase == Phase::Relocate {
                    return Err(GumballError::InvalidState);
                }
                self.alignment = a;
                if let Some(s)=self.owner {self.owner_frames.remove(&(s.document,s.object,s.generation,s.epoch));}
            }
            Event::ObserveFrame(f, cp) => {
                if self.gesture.is_some() || self.phase == Phase::Relocate {
                    return Err(GumballError::InvalidState);
                }
                f.validate()?;
                cp.validate()?;
                self.frame = f;
                self.cplane = cp;
            }
            Event::ObserveOwner(owner,f,cp) => {
                if self.gesture.is_some()||self.phase==Phase::Relocate {return Err(GumballError::InvalidState);}
                f.validate()?;cp.validate()?;
                if owner.is_some_and(|s|[s.document,s.object,s.generation,s.revision,s.epoch].contains(&0)){return Err(GumballError::InvalidInput);}
                self.owner=owner;self.frame=f;self.cplane=cp;
                if let Some(s)=owner {
                    if let Some((captured,frame))=self.owner_frames.get(&(s.document,s.object,s.generation,s.epoch)) {
                        if *captured==s {self.frame=*frame;}
                    }
                }
            }
            Event::Begin(mut c) => {
                if !self.enabled || self.gesture.is_some() || self.phase == Phase::Relocate {
                    return Err(GumballError::InvalidState);
                }
                c.frame.validate()?;
                c.cplane.validate()?;
                validate_ray(c.start_ray)?;
                if c.document == 0
                    || c.view_generation == 0
                    || c.unit_generation == 0
                    || !c.mm_per_unit.is_finite()
                    || c.mm_per_unit <= 0.
                    || c.modifiers & !15 != 0
                {
                    return Err(GumballError::InvalidInput);
                }
                c.sources = roots(&c.sources, c.document)?;
                self.generation = self.generation.checked_add(1).ok_or(GumballError::Limit)?;
                let start = project(&c, c.start_ray).unwrap_or(c.frame.origin);
                self.old_frame = c.frame;
                self.frame = c.frame;
                self.cplane = c.cplane;
                self.phase = Phase::Pressed;
                self.gesture = Some(Gesture {
                    copy: c.modifiers & 4 != 0,
                    relocate: c.modifiers & 2 != 0,
                    modifiers: c.modifiers,
                    context: c,
                    start,
                    value: Value {
                        amount: 0.,
                        axis_direction: 1.,
                        vector: [0.; 3],
                        constraint: None,
                        numeric: None,
                    },
                    history: Vec::new(),
                    extrude: false,
                });
                if matches!(
                    self.gesture.as_ref().unwrap().context.handle,
                    Handle::Scale(_)
                ) {
                    self.gesture.as_mut().unwrap().value.amount = 1.;
                }
            }
            Event::Motion(ray) => {
                if !matches!(self.phase, Phase::Pressed | Phase::Dragging) {
                    return Err(GumballError::InvalidState);
                }
                validate_ray(ray)?;
                let g = self.gesture.as_mut().ok_or(GumballError::InvalidState)?;
                let hit = project(&g.context, ray)?;
                let delta = sub(hit, g.start);
                let f = g.context.frame;
                match g.context.handle {
                    Handle::MoveAxis(a) => {
                        let projected = dot(delta, f.axes[a as usize]);
                        // A distance constraint follows the pointer's side of
                        // the press point. Keep its last side exactly at zero;
                        // a signed input reverses that direction deliberately.
                        if projected != 0. {
                            g.value.axis_direction = if projected < 0. { -1. } else { 1. };
                        }
                        g.value.amount = g.value.constraint
                            .map_or(projected, |n| n * g.value.axis_direction);
                        g.value.vector = mul(f.axes[a as usize], g.value.amount);
                    }
                    Handle::MovePlane(_) | Handle::MoveFree => {
                        g.value.vector = if let Some(n) = g.value.constraint {
                            mul(unit(delta)?, n)
                        } else {
                            delta
                        };
                        g.value.amount = norm(g.value.vector);
                    }
                    Handle::Scale(a) => {
                        if g.relocate {
                            g.value.vector = delta;
                        } else {
                        let start = dot(sub(g.start, f.origin), f.axes[a as usize]);
                        if start.abs() < 1e-12 {
                            return Err(GumballError::Projection);
                        }
                        g.value.amount = g
                            .value
                            .constraint
                            .unwrap_or(dot(sub(hit, f.origin), f.axes[a as usize]) / start);
                        }
                    }
                    Handle::Rotate(a) => {
                        let u = unit(sub(g.start, f.origin))?;
                        let v = unit(sub(hit, f.origin))?;
                        let angle = dot(cross(u, v), f.axes[a as usize]).atan2(dot(u, v));
                        g.value.amount = if let Some(step) = g.value.constraint {
                            if step == 0. {
                                0.
                            } else {
                                (angle / step).round() * step
                            }
                        } else {
                            angle
                        };
                    }
                }
                self.phase = Phase::Dragging;
                if g.relocate {
                    self.frame = f;
                    if let Handle::Rotate(a) = g.context.handle {
                        let u=f.axes[a as usize];let (sin,cos)=g.value.amount.sin_cos();
                        for i in 0..3 {
                            let v=f.axes[i];
                            self.frame.axes[i]=add(add(mul(v,cos),mul(cross(u,v),sin)),mul(u,dot(u,v)*(1.-cos)));
                        }
                    } else {self.frame.origin = add(f.origin, g.value.vector);}
                    self.frame.validate()?;
                } else {
                    self.plan()?;
                }
                return Ok(Effect::Preview);
            }
            Event::Modifier(e) => {
                if e.auto_repeat {
                    return Ok(Effect::None);
                }
                let g = self.gesture.as_mut().ok_or(GumballError::InvalidState)?;
                let bit = e.key as u32;
                let was = g.modifiers & bit != 0;
                if e.pressed {
                    g.modifiers |= bit;
                } else {
                    g.modifiers &= !bit;
                }
                if e.key == ModifierKey::Alt && e.pressed && !was {
                    g.copy = !g.copy;
                }
                if e.key == ModifierKey::Ctrl
                    && self.phase == Phase::Dragging
                    && !g.relocate
                    && matches!(g.context.handle, Handle::MoveAxis(_))
                {
                    g.extrude = e.pressed;
                }
                if !g.relocate {
                    self.plan()?;
                }
                return Ok(Effect::Preview);
            }
            Event::Numeric(raw) => {
                if !matches!(self.phase, Phase::Pressed | Phase::Dragging) {
                    return Err(GumballError::InvalidState);
                }
                let g = self.gesture.as_mut().ok_or(GumballError::InvalidState)?;
                if raw.len() > 1024 || raw.contains('\0') {
                    return Err(GumballError::Limit);
                }
                let n = match g.context.handle {
                    Handle::Rotate(_) => {
                        units::parse_angle(&raw).map_err(|_| GumballError::InvalidInput)?
                    }
                    Handle::Scale(_) if !g.relocate => raw
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| GumballError::InvalidInput)?,
                    _ => units::parse_length(&raw, g.context.mm_per_unit)
                        .map_err(|_| GumballError::InvalidInput)?,
                };
                if !n.is_finite() || !g.relocate && matches!(g.context.handle, Handle::Scale(_)) && n == 0. {
                    return Err(GumballError::InvalidInput);
                }
                if g.history.len() == crate::coordinate_input::MAX_INPUTS {
                    return Err(GumballError::Limit);
                }
                g.history.push(g.value.clone());
                let axis_drag = self.phase == Phase::Dragging && matches!(g.context.handle, Handle::MoveAxis(_));
                g.value.amount = if axis_drag { n * g.value.axis_direction } else { n };
                g.value.numeric = Some(raw);
                g.value.vector = match g.context.handle {
                    Handle::MoveAxis(a) | Handle::Scale(a) if g.relocate || matches!(g.context.handle,Handle::MoveAxis(_)) => mul(g.context.frame.axes[a as usize], g.value.amount),
                    Handle::MoveFree | Handle::MovePlane(_) => {
                        let fallback=match g.context.handle {Handle::MovePlane(Plane::YZ)=>g.context.frame.axes[1],_=>g.context.frame.axes[0]};
                        mul(unit(g.value.vector).unwrap_or(fallback), n)
                    }
                    _ => [0.; 3],
                };
                if g.relocate {
                    let f=g.context.frame;self.frame=f;
                    if let Handle::Rotate(a)=g.context.handle {
                        let u=f.axes[a as usize];let (sin,cos)=n.sin_cos();
                        for i in 0..3 {let v=f.axes[i];self.frame.axes[i]=add(add(mul(v,cos),mul(cross(u,v),sin)),mul(u,dot(u,v)*(1.-cos)));}
                    } else {self.frame.origin=add(f.origin,g.value.vector);}
                    self.frame.validate()?;
                } else {self.plan()?;}
                return Ok(Effect::Preview);
            }
            Event::Accept => {
                if self.phase == Phase::Relocate {
                    return self.finish_relocate();
                }
                if self.phase == Phase::Dragging {
                    let g = self.gesture.as_mut().ok_or(GumballError::InvalidState)?;
                    if g.value.numeric.is_none() {
                        return Err(GumballError::InvalidState);
                    }
                    g.value.constraint = Some(if matches!(g.context.handle,Handle::MoveAxis(_)) {
                        g.value.amount * g.value.axis_direction
                    } else { g.value.amount });
                    return Ok(Effect::Preview);
                }
                return self.commit();
            }
            Event::Release => {
                return self.commit();
            }
            Event::Undo => {
                let g = self.gesture.as_mut().ok_or(GumballError::InvalidState)?;
                g.value = g.history.pop().ok_or(GumballError::InvalidState)?;
                return Ok(Effect::Preview);
            }
            Event::Cancel => {
                self.cancel();
            }
            Event::CommitSucceeded => {
                if self.phase != Phase::Commit {
                    return Err(GumballError::InvalidState);
                }
                self.gesture = None;
                self.phase = Phase::Ready;
            }
            Event::BeginRelocate => {
                if !self.enabled {
                    return Err(GumballError::InvalidState);
                }
                self.cancel();
                self.old_frame = self.frame;
                self.phase = Phase::Relocate;
                self.relocate.clear();
            }
            Event::RelocateRay(ray) => {
                validate_ray(ray)?;
                let direction=unit(ray.direction)?;
                let normal=self.cplane.axes[2];let denominator=dot(normal,direction);
                if denominator.abs()<1e-12 {return Err(GumballError::Projection);}
                let t=dot(normal,sub(self.cplane.origin,ray.origin))/denominator;
                return self.update(Event::RelocatePoint(add(ray.origin,mul(direction,t))));
            }
            Event::RelocatePoint(p) => {
                if self.phase != Phase::Relocate || self.relocate.len() >= 3 {
                    return Err(GumballError::InvalidState);
                }
                bounded(&p)?;
                if self.relocate.len() == 1 {
                    unit(sub(p, self.relocate[0]))?;
                }
                if self.relocate.len() == 2 {
                    unit(cross(
                        sub(self.relocate[1], self.relocate[0]),
                        sub(p, self.relocate[0]),
                    ))?;
                }
                self.relocate.push(p);
                if self.relocate.len() == 3 {
                    return self.finish_relocate();
                }
                return Ok(Effect::Preview);
            }
        }
        Ok(Effect::None)
    }
    fn cancel(&mut self) {
        if self.gesture.is_some() || self.phase == Phase::Relocate {
            self.frame = self.old_frame;
        }
        self.gesture = None;
        self.relocate.clear();
        self.phase = if self.enabled {
            Phase::Ready
        } else {
            Phase::Off
        };
    }
    fn commit(&mut self) -> Result<Effect, GumballError> {
        if !matches!(self.phase, Phase::Pressed | Phase::Dragging) {
            return Err(GumballError::InvalidState);
        }
        if self.gesture.as_ref().is_some_and(|g| g.relocate) {
            self.remember_owner_frame();
            self.gesture = None;
            self.phase = Phase::Ready;
            return Ok(Effect::FrameChanged);
        }
        let plan = self.plan()?;
        if plan.matrix == IDENTITY {
            self.cancel();
            return Ok(Effect::None);
        }
        self.phase = Phase::Commit;
        Ok(Effect::Commit)
    }
    fn finish_relocate(&mut self) -> Result<Effect, GumballError> {
        if self.relocate.len() < 2 {
            return Err(GumballError::InvalidInput);
        }
        let o = self.relocate[0];
        let x = unit(sub(self.relocate[1], o))?;
        let hint = if self.relocate.len() == 3 {
            sub(self.relocate[2], o)
        } else {
            let normal = self.cplane.axes[2];
            if norm(cross(x, normal)) > 1e-12 {
                cross(normal, x)
            } else {
                return Err(GumballError::InvalidInput);
            }
        };
        let z = unit(cross(x, hint))?;
        let y = cross(z, x);
        self.frame = Frame::new(o, [x, y, z])?;
        self.remember_owner_frame();
        self.phase = Phase::Ready;
        self.relocate.clear();
        Ok(Effect::FrameChanged)
    }
    fn remember_owner_frame(&mut self) {
        if let Some(s)=self.owner {
            let key=(s.document,s.object,s.generation,s.epoch);
            if self.owner_frames.len()>=256&&!self.owner_frames.contains_key(&key) {
                if let Some(old)=self.owner_frames.keys().min().copied(){self.owner_frames.remove(&old);}
            }
            self.owner_frames.insert(key,(s,self.frame));
        }
    }
    pub fn plan(&self) -> Result<TransformPlan, GumballError> {
        let g = self.gesture.as_ref().ok_or(GumballError::InvalidState)?;
        if g.relocate {
            return Err(GumballError::InvalidState);
        }
        let f = g.context.frame;
        let mut m = IDENTITY;
        let operation = match g.context.handle {
            Handle::MoveAxis(_) | Handle::MovePlane(_) | Handle::MoveFree => {
                for r in 0..3 {
                    m[4 * r + 3] = g.value.vector[r];
                }
                if g.extrude {
                    Operation::Extrude
                } else {
                    Operation::Move
                }
            }
            Handle::Rotate(a) => {
                let u = f.axes[a as usize];
                let (s, c) = g.value.amount.sin_cos();
                for r in 0..3 {
                    for col in 0..3 {
                        let skew = match (r, col) {
                            (0, 1) => -u[2],
                            (0, 2) => u[1],
                            (1, 0) => u[2],
                            (1, 2) => -u[0],
                            (2, 0) => -u[1],
                            (2, 1) => u[0],
                            _ => 0.,
                        };
                        m[4 * r + col] =
                            if r == col { c } else { 0. } + (1. - c) * u[r] * u[col] + s * skew;
                    }
                }
                pivot(&mut m, f.origin);
                Operation::Rotate
            }
            Handle::Scale(a) => {
                let n = g.value.amount;
                if !n.is_finite() || n == 0. {
                    return Err(GumballError::InvalidInput);
                }
                let uniform = g.modifiers & 1 != 0;
                for r in 0..3 {
                    for c in 0..3 {
                        m[4 * r + c] = if uniform {
                            if r == c { n } else { 0. }
                        } else {
                            f64::from(r == c)
                                + (n - 1.) * f.axes[a as usize][r] * f.axes[a as usize][c]
                        };
                    }
                }
                pivot(&mut m, f.origin);
                Operation::Scale
            }
        };
        crate::geometry_frame::Frame::compose(m, IDENTITY, IDENTITY, IDENTITY)
            .map_err(|_| GumballError::InvalidInput)?;
        Ok(TransformPlan {
            matrix: m,
            operation,
            copy: g.copy,
            sources: g.context.sources.clone(),
            generation: self.generation,
            provenance: g.value.numeric.clone(),
        })
    }
    pub fn report(&self) -> Result<String,GumballError> {
        let plan=self.plan()?;let gesture=self.gesture.as_ref().ok_or(GumballError::InvalidState)?;
        let sources=plan.sources.iter().map(|s|format!("{{\"document\":{},\"object\":{},\"generation\":{},\"revision\":{},\"epoch\":{},\"parent\":{}}}",s.stamp.document,s.stamp.object,s.stamp.generation,s.stamp.revision,s.stamp.epoch,s.parent)).collect::<Vec<_>>().join(",");
        let numeric=plan.provenance.as_deref().map(json_text).unwrap_or_else(||"null".into());let c=&gesture.context;
        let report=format!("{{\"schema_version\":1,\"feature_id\":\"OM9-INFO-020\",\"operation\":{},\"copy\":{},\"generation\":{},\"numeric\":{},\"mm_per_unit\":{},\"frame\":{{\"origin\":{:?},\"axes\":{:?}}},\"cplane\":{{\"origin\":{:?},\"axes\":{:?}}},\"matrix\":{:?},\"sources\":[{}]}}",plan.operation as u32,plan.copy,plan.generation,numeric,c.mm_per_unit,c.frame.origin,c.frame.axes,c.cplane.origin,c.cplane.axes,plan.matrix,sources);
        if report.len()>4*1024*1024{return Err(GumballError::Limit);}Ok(report)
    }
}
fn json_text(raw:&str)->String{let mut out=String::from("\"");for c in raw.chars(){match c{'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),c if c.is_control()=>out.push_str(&format!("\\u{:04x}",c as u32)),c=>out.push(c)}}out.push('"');out}
fn roots(sources: &[Source], document: u64) -> Result<Vec<Source>, GumballError> {
    if sources.is_empty() || sources.len() > crate::native_input_tree::MAX_NODES {
        return Err(GumballError::InvalidSources);
    }
    let mut map = HashMap::new();
    for s in sources {
        let t = s.stamp;
        if t.document != document || t.object == 0 || t.generation == 0 || t.epoch == 0 {
            return Err(GumballError::InvalidSources);
        }
        crate::geometry_frame::Frame::compose(s.world, IDENTITY, IDENTITY, IDENTITY)
            .map_err(|_| GumballError::InvalidSources)?;
        if let Some(old) = map.insert(t.object, s) {
            if old != s {
                return Err(GumballError::InvalidSources);
            }
        }
    }
    let mut seen = HashSet::new();
    let mut roots = Vec::new();
    for s in sources {
        if !seen.insert(s.stamp.object) {
            continue;
        }
        let mut p = s.parent;
        let mut chain = HashSet::new();
        let mut owned = false;
        while p != 0 {
            if chain.len() >= crate::native_input_tree::MAX_DEPTH {return Err(GumballError::Limit);}
            if !chain.insert(p) || p == s.stamp.object {
                return Err(GumballError::InvalidSources);
            }
            if let Some(parent) = map.get(&p) {
                owned = true;
                p = parent.parent;
            } else {
                break;
            }
        }
        if !owned {
            roots.push(s.clone());
        }
    }
    Ok(roots)
}
fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn mul(a: Point, n: f64) -> Point {
    a.map(|v| v * n)
}
fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: Point) -> f64 {
    a[0].hypot(a[1]).hypot(a[2])
}
fn bounded(p: &[f64]) -> Result<(), GumballError> {
    if p.iter().any(|v| !v.is_finite() || v.abs() > 1e150) {
        Err(GumballError::InvalidInput)
    } else {
        Ok(())
    }
}
fn unit(a: Point) -> Result<Point, GumballError> {
    bounded(&a)?;
    let n = norm(a);
    if n <= 1e-12 || !n.is_finite() {
        return Err(GumballError::InvalidInput);
    }
    Ok(mul(a, 1. / n))
}
fn validate_ray(r: PointerRay) -> Result<(), GumballError> {
    bounded(&r.origin)?;
    unit(r.direction)?;
    Ok(())
}
fn pivot(m: &mut [f64; 16], o: Point) {
    for r in 0..3 {
        m[4 * r + 3] = o[r] - (0..3).map(|c| m[4 * r + c] * o[c]).sum::<f64>();
    }
}
fn project(c: &BeginContext, ray: PointerRay) -> Result<Point, GumballError> {
    let f = c.frame;
    let d = unit(ray.direction)?;
    match c.handle {
        Handle::MoveAxis(a) | Handle::Scale(a) => {
            let axis = f.axes[a as usize];
            let q = sub(ray.origin, f.origin);
            let ad = dot(axis, d);
            let den = 1. - ad * ad;
            if den.abs() < 1e-12 {
                return Err(GumballError::Projection);
            }
            let t = (dot(axis, q) - ad * dot(d, q)) / den;
            Ok(add(f.origin, mul(axis, t)))
        }
        _ => {
            let n = match c.handle {
                Handle::MovePlane(p) => {
                    f.axes[match p {
                        Plane::XY => 2,
                        Plane::YZ => 0,
                        Plane::ZX => 1,
                    }]
                }
                Handle::Rotate(a) => f.axes[a as usize],
                _ => unit(c.start_ray.direction)?,
            };
            let den = dot(n, d);
            if den.abs() < 1e-12 {
                return Err(GumballError::Projection);
            }
            let t = dot(n, sub(f.origin, ray.origin)) / den;
            let p = add(ray.origin, mul(d, t));
            bounded(&p)?;
            Ok(p)
        }
    }
}
