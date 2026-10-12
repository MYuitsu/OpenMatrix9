//! OM9-CURVE-006 portable analytic ellipse construction.
use crate::circle::check_point;
pub type Point = [f64; 3];
type Axes = [Point; 3];
const AXES: Axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
const MIN: f64 = 1e-7;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Center,
    Diameter,
    Corner,
    Vertical,
    FromFoci,
    AroundCurve,
}
#[derive(Clone, Debug)]
pub struct Ellipse {
    pub mode: Mode,
    pub deformable: bool,
    pub degree: usize,
    pub point_count: usize,
    pub mark_foci: bool,
    pub axes: Axes,
    selected: bool,
    path_normal: Option<Point>,
}
impl Default for Ellipse {
    fn default() -> Self {
        Self {
            mode: Mode::Center,
            deformable: false,
            degree: 3,
            point_count: 8,
            mark_foci: false,
            axes: AXES,
            selected: false,
            path_normal: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub center: Point,
    pub major_direction: Point,
    pub normal: Point,
    pub major: f64,
    pub minor: f64,
}
fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: Point, t: f64) -> Point {
    a.map(|v| v * t)
}
fn dot(a: Point, b: Point) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
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
fn unit(a: Point) -> Result<Point, String> {
    let n = norm(a);
    if !n.is_finite() || n < MIN {
        Err("Ellipse needs distinct, non-collinear axis points".into())
    } else {
        Ok(scale(a, 1. / n))
    }
}
fn project(a: Point, n: Point) -> Point {
    sub(a, scale(n, dot(a, n)))
}
impl Plan {
    pub fn new(
        center: Point,
        direction: Point,
        normal: Point,
        first: f64,
        second: f64,
    ) -> Result<Self, String> {
        check_point(center)?;
        if !first.is_finite() || !second.is_finite() || first < MIN || second < MIN {
            return Err("Ellipse semiaxes must be at least 1e-7 mm".into());
        }
        let normal = unit(normal)?;
        let mut direction = unit(project(direction, normal))?;
        let (major, minor) = if first >= second {
            (first, second)
        } else {
            direction = cross(normal, direction);
            (second, first)
        };
        let v = cross(normal, direction);
        for i in 0..3 {
            let extent = (major * direction[i]).hypot(minor * v[i]);
            if !extent.is_finite() || center[i].abs() + extent > 1e9 {
                return Err("Ellipse extent exceeds coordinate limits".into());
            }
        }
        Ok(Self {
            center,
            major_direction: direction,
            normal,
            major,
            minor,
        })
    }
    pub fn value(&self, t: f64) -> Point {
        let angle = std::f64::consts::TAU * t;
        let v = cross(self.normal, self.major_direction);
        std::array::from_fn(|i| {
            self.center[i]
                + self.major * angle.cos() * self.major_direction[i]
                + self.minor * angle.sin() * v[i]
        })
    }
    pub fn sample(&self) -> Vec<Point> {
        let mut p = (0..128)
            .map(|i| self.value(i as f64 / 128.))
            .collect::<Vec<_>>();
        p.push(p[0]);
        p
    }
    pub fn foci(&self) -> [Point; 2] {
        let d = ((self.major - self.minor) * (self.major + self.minor)).sqrt();
        [
            add(self.center, scale(self.major_direction, d)),
            sub(self.center, scale(self.major_direction, d)),
        ]
    }
    pub fn transform_unit(&self, p: Point) -> Point {
        let v = cross(self.normal, self.major_direction);
        std::array::from_fn(|i| {
            self.center[i] + self.major * p[0] * self.major_direction[i] + self.minor * p[1] * v[i]
        })
    }
}
impl Ellipse {
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.selected = false;
        self.path_normal = None;
    }
    pub fn reference_mode(&self, points: &[Point]) -> u32 {
        if self.mode != Mode::AroundCurve || !points.is_empty() {
            0
        } else if self.selected {
            2
        } else {
            1
        }
    }
    pub fn reference(
        &mut self,
        points: &mut Vec<Point>,
        p: Point,
        normal: Point,
        kind: u32,
    ) -> Result<(), String> {
        match (self.reference_mode(points), kind) {
            (1, 1) => {
                self.selected = true;
                Ok(())
            }
            (2, 2) => {
                check_point(p)?;
                let n = unit(normal)?;
                self.path_normal = Some(n);
                points.push(p);
                Ok(())
            }
            _ => Err("AroundCurve requires a native edge followed by a center on that edge".into()),
        }
    }
    pub fn undo(&mut self, points: &mut Vec<Point>) -> Result<(), String> {
        if points.pop().is_some() {
            if points.is_empty() {
                self.path_normal = None;
            }
            Ok(())
        } else if self.selected {
            self.selected = false;
            self.path_normal = None;
            Ok(())
        } else {
            Err("No Ellipse point or curve to undo".into())
        }
    }
    fn normal(&self) -> Point {
        self.path_normal.unwrap_or(self.axes[2])
    }
    fn first_axis(&self, points: &[Point]) -> Result<(Point, Point, f64, Point), String> {
        let d = project(sub(points[1], points[0]), self.normal());
        let direction = unit(d)?;
        let center = if self.mode == Mode::Diameter {
            add(points[0], scale(d, 0.5))
        } else {
            points[0]
        };
        let radius = norm(d) * if self.mode == Mode::Diameter { 0.5 } else { 1. };
        if radius < MIN {
            return Err("Ellipse first semiaxis is too small".into());
        }
        let normal = if self.mode == Mode::Vertical {
            unit(cross(direction, self.axes[2]))?
        } else {
            self.normal()
        };
        Ok((center, direction, radius, normal))
    }
    pub fn pick(&self, points: &mut Vec<Point>, p: Point) -> Result<Option<Plan>, String> {
        check_point(p)?;
        if self.reference_mode(points) != 0 {
            return Err("Select an edge and its center before picking ellipse axes".into());
        }
        if points.is_empty() {
            points.push(p);
            return Ok(None);
        }
        if self.mode == Mode::Corner {
            let d = sub(p, points[0]);
            let x = dot(d, self.axes[0]);
            let y = dot(d, self.axes[1]);
            let center = add(
                points[0],
                scale(add(scale(self.axes[0], x), scale(self.axes[1], y)), 0.5),
            );
            let plan = Plan::new(
                center,
                self.axes[0],
                self.axes[2],
                x.abs() * 0.5,
                y.abs() * 0.5,
            )?;
            points.push(p);
            return Ok(Some(plan));
        }
        if points.len() == 1 {
            if self.mode == Mode::FromFoci {
                unit(sub(p, points[0]))?;
            } else {
                let trial = [points[0], p];
                self.first_axis(&trial)?;
            }
            points.push(p);
            return Ok(None);
        }
        let plan = if self.mode == Mode::FromFoci {
            let center = scale(add(points[0], points[1]), 0.5);
            let separation = norm(sub(points[1], points[0]));
            let direction = unit(sub(points[1], points[0]))?;
            let d = sub(p, center);
            let normal = unit(cross(direction, d))?;
            let a = (norm(sub(p, points[0])) + norm(sub(p, points[1]))) * 0.5;
            let c = separation * 0.5;
            if a <= c {
                return Err("Sum of distances must exceed the focus separation".into());
            }
            Plan::new(center, direction, normal, a, ((a - c) * (a + c)).sqrt())?
        } else {
            let (center, direction, a, normal) = self.first_axis(points)?;
            let second = dot(sub(p, center), cross(normal, direction)).abs();
            Plan::new(center, direction, normal, a, second)?
        };
        points.push(p);
        Ok(Some(plan))
    }
    pub fn numeric_point(&self, points: &[Point], size: f64) -> Result<Point, String> {
        if !size.is_finite() || size < MIN {
            return Err("Ellipse axis length must be positive".into());
        }
        if matches!(self.mode, Mode::Corner | Mode::FromFoci)
            || points.is_empty()
            || self.reference_mode(points) != 0
        {
            return Err("This Ellipse step requires a point".into());
        }
        if points.len() == 1 {
            let n = self.normal();
            let direction =
                unit(project(self.axes[0], n)).or_else(|_| unit(project(self.axes[1], n)))?;
            Ok(add(points[0], scale(direction, size)))
        } else {
            let (center, direction, _, normal) = self.first_axis(points)?;
            Ok(add(center, scale(cross(normal, direction), size)))
        }
    }
    pub fn pick_plane(&self, points: &[Point]) -> Option<(Point, Point)> {
        if self.mode == Mode::FromFoci {
            return None;
        }
        let origin = *points.first()?;
        let normal = if self.mode == Mode::Vertical && points.len() > 1 {
            self.first_axis(points).ok()?.3
        } else {
            self.normal()
        };
        Some((origin, normal))
    }
    pub fn step(&self, points: &[Point]) -> &'static str {
        match self.reference_mode(points) {
            1 => return "Select curve (Curve=Object.EdgeN)",
            2 => return "Center on curve (OnCurve=fraction)",
            _ => {}
        }
        match (self.mode, points.len()) {
            (Mode::Corner, 0) => "First corner",
            (Mode::Corner, _) => "Opposite corner",
            (Mode::FromFoci, 0) => "First focus",
            (Mode::FromFoci, 1) => "Second focus",
            (Mode::FromFoci, _) => "Point on ellipse",
            (Mode::Diameter, 0) => "First axis start",
            (Mode::Diameter, 1) => "First axis end or full diameter",
            (_, 0) => "Center",
            (_, 1) => "First semiaxis endpoint or length",
            _ => "Second semiaxis endpoint or length",
        }
    }
}
