//! OM9-SOLID-012 / OM9-SOLID-014: native solid creation sessions.
//! Host decisions: mm, right-handed CPlane, standalone BRep, 1e-7 mm minimum.
use crate::coordinates::{Frame, PointInput};
use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};
type Point = [f64; 3];
mod geometry;
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
    mode: Mode,
    points: Vec<Point>,
    circle: Option<(Point, f64, Point)>,
    radius: f64,
    point_constraint: bool,
    constraint_types: Vec<bool>,
    output: Option<[f64; 15]>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Corners,
    Diagonal,
    ThreePoint,
    Vertical,
    Center,
    Cube,
    TwoPoint,
    FourPoint,
    FitPoints,
    AroundCurve,
    Tangent,
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
                mode: if kind == Kind::Box {
                    Mode::Corners
                } else {
                    Mode::Center
                },
                points: Vec::new(),
                circle: None,
                radius: 0.,
                point_constraint: false,
                constraint_types: Vec::new(),
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
                    if self.data.mode == Mode::Center {
                        "Base center"
                    } else {
                        "First corner (Diagonal / 3Point / Vertical / Center / Cube / Esc)"
                    }
                } else {
                    "Center (2Point / 3Point / 4Point / FitPoints / Vertical / AroundCurve / Tangent / Esc)"
                }
            }
            2 => {
                if matches!(self.data.mode, Mode::Diagonal | Mode::Cube) {
                    "Opposite diagonal corner"
                } else if matches!(self.data.mode, Mode::ThreePoint | Mode::Vertical) {
                    "End of first edge or Length"
                } else {
                    "Other base corner or Length"
                }
            }
            3 => "Width (Enter = Length)",
            4 => "Height (Enter = Width)",
            5 => "Radius (pick a point or enter a positive value)",
            6 => "Ready to create solid",
            7 => "Other diameter endpoint or Diameter",
            8 => "Second point",
            9 => "Third circle point or Radius",
            10 => "Fourth point outside circle plane",
            11 => "Direction off chord to orient radius circle",
            15 => "FitPoints: add points, Enter to fit, Undo / Esc",
            12 => "Radius for the picked chord",
            16 => "Select a native curve (Path=Object.EdgeN)",
            17 => "Pick center on curve (OnCurve=0..1)",
            20 => {
                if self.data.point_constraint {
                    "Pick a non-tangent Point"
                } else {
                    "Select tangent curve (Curve=Object.EdgeN; Point / Radius after two constraints)"
                }
            }
            21 => "Resolve tangent construction or Undo / Esc",
            22 => "Radius for two tangent/Point constraints",
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
        if self.kind == Kind::Sphere && text.eq_ignore_ascii_case("point") && self.phase() == 20 {
            self.data.point_constraint = true;
            return Ok(Effect::Waiting);
        }
        if self.kind == Kind::Sphere && text.eq_ignore_ascii_case("radius") && self.phase() == 9 {
            self.data.phase = 12;
            return Ok(Effect::Waiting);
        }
        if self.kind == Kind::Sphere
            && text.eq_ignore_ascii_case("radius")
            && self.phase() == 20
            && self.data.points.len() == 2
        {
            self.data.phase = 22;
            return Ok(Effect::Waiting);
        }
        let mode_text = text
            .strip_prefix("Mode=")
            .or_else(|| text.strip_prefix("mode="))
            .unwrap_or(text)
            .to_ascii_lowercase();
        let mode = match (self.kind, mode_text.as_str()) {
            (Kind::Box, "corners") => Some(Mode::Corners),
            (Kind::Box, "diagonal") => Some(Mode::Diagonal),
            (Kind::Box, "vertical") => Some(Mode::Vertical),
            (Kind::Box, "center") => Some(Mode::Center),
            (Kind::Box, "cube") => Some(Mode::Cube),
            (_, "3point" | "3p") => Some(Mode::ThreePoint),
            (Kind::Sphere, "center") => Some(Mode::Center),
            (Kind::Sphere, "vertical") => Some(Mode::Vertical),
            (Kind::Sphere, "2point" | "2p") => Some(Mode::TwoPoint),
            (Kind::Sphere, "4point" | "4p") => Some(Mode::FourPoint),
            (Kind::Sphere, "fitpoints") => Some(Mode::FitPoints),
            (Kind::Sphere, "aroundcurve") => Some(Mode::AroundCurve),
            (Kind::Sphere, "tangent") => Some(Mode::Tangent),
            _ => None,
        };
        if let Some(mode) = mode {
            if self.phase() != 1
                && !(self.kind == Kind::Box
                    && mode == Mode::Cube
                    && self.phase() == 2
                    && self.data.mode == Mode::Diagonal)
            {
                return Err("Select the construction mode before the first point".into());
            }
            self.data.mode = mode;
            match mode {
                Mode::FitPoints => self.data.phase = 15,
                Mode::AroundCurve => self.data.phase = 16,
                Mode::Tangent => {
                    self.data.phase = 20;
                    self.data.first = self.frame.0;
                    self.data.axes = self.frame.1;
                }
                _ => {}
            }
            return Ok(Effect::Waiting);
        }
        if text.is_empty() {
            return match self.phase() {
                3 => self.numeric(self.data.length),
                4 => self.numeric(self.data.width.abs()),
                15 => {
                    let (c, r) = geometry::fit_points(&self.data.points)?;
                    self.data.first = c;
                    self.finish_sphere(r)
                }
                _ => Err("Enter a point or dimension before confirming".into()),
            };
        }
        if let Some((option, value)) = text.split_once('=') {
            if self.kind == Kind::Sphere
                && option.eq_ignore_ascii_case("Radius")
                && matches!(self.phase(), 20 | 22)
                && self.data.points.len() == 2
            {
                let r = dimension(number(value)?)?;
                if r <= 0. {
                    return Err("Radius must be positive".into());
                }
                self.data.radius = r;
                self.data.phase = 21;
                return Ok(Effect::Waiting);
            }
            if self.kind == Kind::Sphere
                && option.eq_ignore_ascii_case("Diameter")
                && self.phase() == 5
            {
                return self.numeric(number(value)? * 0.5);
            }
            if self.kind == Kind::Sphere
                && option.eq_ignore_ascii_case("Radius")
                && self.phase() == 9
            {
                let r = dimension(number(value)?)?;
                if r <= 0.
                    || r < dot(
                        sub(self.data.points[1], self.data.points[0]),
                        sub(self.data.points[1], self.data.points[0]),
                    )
                    .sqrt()
                        * 0.5
                {
                    return Err("Radius must be at least half the chord".into());
                }
                self.data.radius = r;
                self.data.phase = 11;
                return Ok(Effect::Waiting);
            }
            let allowed = match self.phase() {
                2 => "Length",
                3 => "Width",
                4 => "Height",
                5 => "Radius",
                7 => "Diameter",
                12 => "Radius",
                22 => "Radius",
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
        let frame = Frame::new(self.frame.0, self.frame.1)?;
        let previous = (self.phase() != 1).then_some(self.data.last);
        let point = PointInput::parse(text, number)?.resolve(frame, previous)?;
        self.point_inner(point.world)
    }
    fn numeric(&mut self, n: f64) -> Result<Effect, String> {
        let n = dimension(n)?;
        match self.phase() {
            2 => {
                if matches!(self.data.mode, Mode::Diagonal | Mode::Cube) {
                    return Err("Pick the opposite diagonal corner".into());
                }
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
            7 => {
                if n <= 0. {
                    return Err("Diameter must be positive".into());
                }
                let c = add(self.data.first, scale(self.data.axes[0], n * 0.5));
                self.data.first = c;
                self.finish_sphere(n * 0.5)
            }
            12 => {
                if n <= 0.
                    || n < dot(
                        sub(self.data.points[1], self.data.points[0]),
                        sub(self.data.points[1], self.data.points[0]),
                    )
                    .sqrt()
                        * 0.5
                {
                    return Err("Radius must be at least half the chord".into());
                }
                self.data.radius = n;
                self.data.phase = 11;
                Ok(Effect::Waiting)
            }
            22 => {
                if n <= 0. {
                    return Err("Radius must be positive".into());
                }
                self.data.radius = n;
                self.data.phase = 21;
                Ok(Effect::Waiting)
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
                if self.data.mode == Mode::Vertical {
                    self.data.axes = [
                        self.frame.1[0],
                        self.frame.1[2],
                        scale(self.frame.1[1], -1.),
                    ];
                }
                self.data.points = vec![p];
                self.data.phase = if self.kind == Kind::Sphere {
                    match self.data.mode {
                        Mode::TwoPoint => 7,
                        Mode::ThreePoint | Mode::FourPoint => 8,
                        _ => 5,
                    }
                } else {
                    2
                };
                Ok(Effect::Waiting)
            }
            2 => {
                let d = sub(p, self.data.first);
                if self.data.mode == Mode::Cube {
                    let (axes, len) = geometry::cube_axes(self.data.axes, d)?;
                    self.data.axes = axes;
                    self.data.length = len;
                    self.data.width = len;
                    return self.finish_box(len);
                }
                let z = self.data.axes[2];
                if matches!(self.data.mode, Mode::ThreePoint | Mode::Vertical) {
                    let plane_normal = if self.data.mode == Mode::Vertical {
                        self.data.axes[1]
                    } else {
                        z
                    };
                    let edge = sub(d, scale(plane_normal, dot(d, plane_normal)));
                    let length = dimension(dot(edge, edge).sqrt())?;
                    self.data.axes[0] = scale(edge, 1. / length);
                    if self.data.mode == Mode::Vertical {
                        self.data.axes[2] = cross(self.data.axes[0], self.data.axes[1]);
                    } else {
                        self.data.axes[1] = cross(z, self.data.axes[0]);
                    }
                    self.data.length = length;
                    self.data.phase = 3;
                } else {
                    let k = if self.data.mode == Mode::Center {
                        2.
                    } else {
                        1.
                    };
                    self.data.length = dimension(k * dot(d, self.data.axes[0]))?;
                    self.data.width = dimension(k * dot(d, self.data.axes[1]))?;
                    self.data.phase = 4;
                }
                self.data.last = p;
                Ok(Effect::Waiting)
            }
            3 => {
                let factor = if self.data.mode == Mode::Center {
                    2.
                } else {
                    1.
                };
                let effect =
                    self.numeric(factor * dot(sub(p, self.data.first), self.data.axes[1]))?;
                self.data.last = p;
                Ok(effect)
            }
            4 => self.finish_box(dot(sub(p, self.data.first), self.data.axes[2])),
            5 => self.finish_sphere(dot(sub(p, self.data.first), sub(p, self.data.first)).sqrt()),
            7 => {
                let radius =
                    dimension(dot(sub(p, self.data.first), sub(p, self.data.first)).sqrt() * 0.5)?;
                self.data.first = scale(add(self.data.first, p), 0.5);
                self.finish_sphere(radius)
            }
            8 => {
                dimension(dot(sub(p, self.data.first), sub(p, self.data.first)).sqrt())?;
                self.data.points.push(p);
                self.data.last = p;
                self.data.phase = 9;
                Ok(Effect::Waiting)
            }
            9 => {
                let circle = geometry::circle(self.data.points[0], self.data.points[1], p)?;
                self.circle_result(circle)
            }
            10 => {
                let (c, r) = geometry::fourth(self.data.circle.unwrap(), p)?;
                self.data.first = c;
                self.finish_sphere(r)
            }
            11 => {
                let circle = geometry::radius_circle(
                    self.data.points[0],
                    self.data.points[1],
                    self.data.radius,
                    p,
                )?;
                self.circle_result(circle)
            }
            15 => {
                if self.data.points.len() >= 1024 {
                    return Err("FitPoints supports up to 1024 points".into());
                }
                self.data.points.push(p);
                self.data.last = p;
                Ok(Effect::Waiting)
            }
            20 => {
                if !self.data.point_constraint {
                    return Err("Select a curve or choose Point first".into());
                }
                self.push_constraint(p, true)
            }
            _ => Err("No editable solid command".into()),
        }
    }
    fn push_constraint(&mut self, p: Point, is_point: bool) -> Result<Effect, String> {
        if self.data.points.len() >= 3 {
            return Err("Three constraints already selected".into());
        }
        if self.data.points.is_empty() {
            self.data.first = p;
            self.data.axes = self.frame.1;
        }
        self.data.points.push(p);
        self.data.constraint_types.push(is_point);
        self.data.last = p;
        self.data.point_constraint = false;
        if self.data.points.len() == 3 {
            self.data.phase = 21;
        }
        Ok(Effect::Waiting)
    }
    pub fn reference_mode(&self) -> u32 {
        match self.phase() {
            16 => 1,
            17 => 2,
            20 if !self.data.point_constraint => 3,
            15 => 4,
            _ => 0,
        }
    }
    pub fn accept_reference(&mut self, p: Point, tangent: Point) -> Result<Effect, String> {
        let before = self.data.clone();
        let effect = (|| {
            if !finite(p) {
                return Err("Invalid reference point".into());
            }
            match self.phase() {
                16 => {
                    self.data.phase = 17;
                    self.data.last = p;
                    Ok(Effect::Waiting)
                }
                17 => {
                    let z = geometry::unit(tangent)?;
                    let candidate = if z[0].abs() < 0.8 {
                        [1., 0., 0.]
                    } else {
                        [0., 1., 0.]
                    };
                    let x = geometry::unit(cross(candidate, z))?;
                    self.data.axes = [x, cross(z, x), z];
                    self.data.first = p;
                    self.data.last = p;
                    self.data.phase = 5;
                    Ok(Effect::Waiting)
                }
                20 if !self.data.point_constraint => self.push_constraint(p, false),
                _ => Err("No curve selection is expected".into()),
            }
        })();
        match effect {
            Ok(e) => {
                self.history.push(before);
                Ok(e)
            }
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    pub fn resolve(&mut self, center: Point, radius: f64) -> Result<Effect, String> {
        if self.phase() != 21 || !finite(center) {
            return Err("No valid tangent solution is expected".into());
        }
        let before = self.data.clone();
        self.data.first = center;
        match self.finish_sphere(radius) {
            Ok(e) => Ok(e),
            Err(e) => {
                self.data = before;
                Err(e)
            }
        }
    }
    pub fn append_fit_points(&mut self, points: &[Point]) -> Result<Effect, String> {
        if self.phase() != 15
            || self.data.points.len() + points.len() > 1024
            || points.iter().any(|&p| !finite(p))
        {
            return Err("Invalid FitPoints batch or point limit exceeded".into());
        }
        if points.is_empty() {
            return Ok(Effect::Waiting);
        }
        self.history.push(self.data.clone());
        self.data.points.extend_from_slice(points);
        self.data.last = *points.last().unwrap();
        Ok(Effect::Waiting)
    }
    fn circle_result(&mut self, circle: (Point, f64, Point)) -> Result<Effect, String> {
        self.data.circle = Some(circle);
        if self.data.mode == Mode::FourPoint {
            self.data.phase = 10;
            Ok(Effect::Waiting)
        } else {
            self.data.first = circle.0;
            self.finish_sphere(circle.1)
        }
    }
    fn finish_box(&mut self, height: f64) -> Result<Effect, String> {
        let dims = [
            dimension(self.data.length)?,
            dimension(self.data.width)?,
            dimension(height)?,
        ];
        let mut origin = self.data.first;
        if self.data.mode == Mode::Center {
            origin = sub(
                sub(origin, scale(self.data.axes[0], dims[0] * 0.5)),
                scale(self.data.axes[1], dims[1] * 0.5),
            );
        }
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
        let mut candidate = Self {
            kind: self.kind,
            data: self.data.clone(),
            frame: self.frame,
            history: Vec::new(),
        };
        candidate.point(candidate.constrain(p, ortho)).ok()?;
        if candidate.data.mode == Mode::FitPoints && candidate.data.points.len() >= 3 {
            candidate.input("").ok()?;
        }
        candidate.output()
    }
    pub fn constrain(&self, p: Point, ortho: bool) -> Point {
        if !ortho
            || self.phase() != 2
            || !matches!(self.data.mode, Mode::ThreePoint | Mode::Vertical)
        {
            return p;
        }
        let delta = sub(p, self.data.first);
        let x = dot(delta, self.data.axes[0]);
        let y_axis = if self.data.mode == Mode::Vertical {
            scale(self.data.axes[2], -1.)
        } else {
            self.data.axes[1]
        };
        let y = dot(delta, y_axis);
        add(
            self.data.first,
            if x.abs() >= y.abs() {
                scale(self.data.axes[0], x)
            } else {
                scale(y_axis, y)
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
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_reference_mode() -> u32 {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, Session::reference_mode)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_mode_name() -> *const c_char {
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let name = guard.session.as_ref().map_or(c"", |s| match s.data.mode {
        Mode::Corners => c"Corners",
        Mode::Diagonal => c"Diagonal",
        Mode::ThreePoint => c"3Point",
        Mode::Vertical => c"Vertical",
        Mode::Center => c"Center",
        Mode::Cube => c"Cube",
        Mode::TwoPoint => c"2Point",
        Mode::FourPoint => c"4Point",
        Mode::FitPoints => c"FitPoints",
        Mode::AroundCurve => c"AroundCurve",
        Mode::Tangent => c"Tangent",
    });
    name.as_ptr()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_reference(x: f64, y: f64, z: f64, tx: f64, ty: f64, tz: f64) -> u32 {
    change(|s| s.accept_reference([x, y, z], [tx, ty, tz]))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_resolve(x: f64, y: f64, z: f64, r: f64) -> u32 {
    change(|s| s.resolve([x, y, z], r))
}
/// # Safety
/// Non-null output addresses 12 doubles: tangent frame origin and its axes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_reference_frame(p: *mut f64) -> bool {
    if p.is_null() {
        return false;
    }
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &guard.session else {
        return false;
    };
    let mut b = [0.; 12];
    b[..3].copy_from_slice(&s.data.first);
    for i in 0..3 {
        b[3 + i * 3..6 + i * 3].copy_from_slice(&s.data.axes[i]);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(b.as_ptr(), p, 12);
    }
    true
}
/// # Safety
/// Non-null output addresses 12 doubles: up to three x,y,z,is_point constraints.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_constraints(p: *mut f64) -> usize {
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &guard.session else {
        return 0;
    };
    if s.data.mode != Mode::Tangent {
        return 0;
    }
    if !p.is_null() {
        for (i, point) in s.data.points.iter().enumerate() {
            unsafe {
                std::ptr::copy_nonoverlapping(point.as_ptr(), p.add(i * 4), 3);
                *p.add(i * 4 + 3) = if s.data.constraint_types[i] { 1. } else { 0. };
            }
        }
    }
    s.data.points.len()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_tangent_radius() -> f64 {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0., |s| s.data.radius)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_solid_point_count() -> usize {
    state()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .session
        .as_ref()
        .map_or(0, |s| s.data.points.len())
}
/// # Safety
/// Input points addresses count*3 readable doubles; count is at most 1024.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_fit_points(p: *const f64, count: usize) -> u32 {
    if p.is_null() || count > 1024 {
        return 0;
    }
    let values = unsafe { std::slice::from_raw_parts(p, count * 3) };
    let points = values.as_chunks::<3>().0.to_vec();
    change(|s| s.append_fit_points(&points))
}
/// # Safety
/// Non-null output addresses 6 doubles: special point-pick plane origin and normal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_pick_plane(p: *mut f64) -> bool {
    if p.is_null() {
        return false;
    }
    let guard = state().lock().unwrap_or_else(|p| p.into_inner());
    let Some(s) = &guard.session else {
        return false;
    };
    if !((s.kind == Kind::Box && s.data.mode == Mode::Vertical && s.phase() == 3)
        || (s.kind == Kind::Sphere
            && matches!(s.data.mode, Mode::Vertical | Mode::AroundCurve)
            && s.phase() == 5))
    {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.data.first.as_ptr(), p, 3);
        std::ptr::copy_nonoverlapping(s.data.axes[2].as_ptr(), p.add(3), 3);
    }
    true
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
