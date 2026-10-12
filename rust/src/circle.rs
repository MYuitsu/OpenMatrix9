//! OM9-CURVE-005 analytic construction. Bounds/plane choices are OM9 policy.
use std::f64::consts::{PI, TAU};
#[path = "circle_fit.rs"]
mod fit;
pub use fit::fit_circle;
pub type Point = [f64; 3];
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Center,
    TwoPoint,
    ThreePoint,
    Vertical,
    AroundCurve,
    FitPoints,
    Tangent,
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Size {
    #[default]
    Radius,
    Diameter,
    Circumference,
    Area,
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub center: Point,
    pub normal: Point,
    pub radius: f64,
}
#[derive(Clone)]
pub struct Circle {
    pub mode: Mode,
    pub size: Size,
    pub axes: Option<[Point; 3]>,
    pub normal: Option<Point>,
    pub orientating: bool,
    pub oriented: bool,
    pub radius_constraint: Option<f64>,
    pub radius_pending: bool,
    pub path_selected: bool,
    pub constraints: Vec<(Point, bool)>,
    pub point_mode: bool,
    pub from_first: bool,
    pub solution: Option<usize>,
    pub deformable: bool,
    pub degree: usize,
    pub point_count: usize,
    pub fit_deviation: f64,
    pub tangent_vertical: bool,
    pub history: bool,
}
impl Default for Circle {
    fn default() -> Self {
        Self {
            mode: Mode::Center,
            size: Size::Radius,
            axes: None,
            normal: None,
            orientating: false,
            oriented: false,
            radius_constraint: None,
            radius_pending: false,
            path_selected: false,
            constraints: Vec::new(),
            point_mode: false,
            from_first: false,
            solution: None,
            deformable: false,
            degree: 3,
            point_count: 8,
            fit_deviation: 0.,
            tangent_vertical: false,
            history: false,
        }
    }
}
fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: Point, n: f64) -> Point {
    a.map(|v| v * n)
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
    dot(a, a).sqrt()
}
fn unit(a: Point) -> Result<Point, String> {
    let n = norm(a);
    if !n.is_finite() || n < 1e-7 {
        Err("Circle needs a distinct point/direction".into())
    } else {
        Ok(scale(a, 1. / n))
    }
}
pub fn check_point(p: Point) -> Result<(), String> {
    if p.iter().any(|v| !v.is_finite() || v.abs() > 1e9) {
        Err("Circle coordinate exceeds finite ±1e9 mm limits".into())
    } else {
        Ok(())
    }
}
/// Infer a common plane from complete native curve evidence, keeping CPlane
/// when it already contains the geometry. This does not fit nonplanar curves.
pub fn tangent_frame(
    points: &[Point],
    origin: Point,
    preferred: [Point; 3],
) -> Result<[Point; 3], String> {
    check_point(origin)?;
    if points.is_empty() || points.len() > 30003 {
        return Err("Invalid tangent plane evidence count".into());
    }
    for p in points {
        check_point(*p)?;
    }
    for i in 0..3 {
        if !preferred[i].iter().all(|n| n.is_finite())
            || (norm(preferred[i]) - 1.).abs() > 1e-8
            || (0..i).any(|j| dot(preferred[i], preferred[j]).abs() > 1e-8)
        {
            return Err("Invalid tangent frame".into());
        }
    }
    let first = points[0];
    let edge = points
        .iter()
        .map(|p| sub(*p, first))
        .max_by(|a, b| norm(*a).total_cmp(&norm(*b)))
        .unwrap();
    let direction = unit(edge)?;
    let span = norm(edge);
    if points
        .iter()
        .all(|p| dot(sub(*p, origin), preferred[2]).abs() <= f64::min(1e-6, span * 1e-10))
    {
        return Ok(preferred);
    }
    let perpendicular = points
        .iter()
        .map(|p| {
            let d = sub(*p, first);
            sub(d, scale(direction, dot(d, direction)))
        })
        .max_by(|a, b| norm(*a).total_cmp(&norm(*b)))
        .unwrap();
    let mut z = if norm(perpendicular) / span > 1e-10 {
        unit(cross(
            direction,
            scale(perpendicular, 1. / norm(perpendicular)),
        ))?
    } else {
        let n = sub(preferred[2], scale(direction, dot(preferred[2], direction)));
        if norm(n) > 1e-7 {
            unit(n)?
        } else {
            unit(sub(
                preferred[1],
                scale(direction, dot(preferred[1], direction)),
            ))?
        }
    };
    if points.iter().any(|p| dot(sub(*p, origin), z).abs() > 1e-6) {
        return Err(
            "Tangent inputs do not share a plane; general nonplanar solving is unsupported".into(),
        );
    }
    let orientation = if dot(z, preferred[2]).abs() > 1e-10 {
        dot(z, preferred[2])
    } else {
        z.into_iter()
            .max_by(|a, b| a.abs().total_cmp(&b.abs()))
            .unwrap()
    };
    if orientation < 0. {
        z = scale(z, -1.);
    }
    let candidate = sub(preferred[0], scale(z, dot(preferred[0], z)));
    let x = if norm(candidate) > 1e-7 {
        unit(candidate)?
    } else {
        direction
    };
    Ok([x, unit(cross(z, x))?, z])
}
impl Plan {
    pub fn new(center: Point, normal: Point, radius: f64) -> Result<Self, String> {
        check_point(center)?;
        let normal = unit(normal)?;
        if !radius.is_finite() || !(1e-7..=1e9).contains(&radius) {
            return Err("Circle radius must be between 1e-7 and 1e9 mm".into());
        }
        for i in 0..3 {
            if center[i].abs() + radius * (1. - normal[i] * normal[i]).max(0.).sqrt() > 1e9 {
                return Err("Circle extent exceeds coordinate limits".into());
            }
        }
        Ok(Self {
            center,
            normal,
            radius,
        })
    }
    pub fn outline(&self) -> Vec<Point> {
        self.sample(128)
    }
    pub fn sample(&self, count: usize) -> Vec<Point> {
        let reference = if self.normal[0].abs() < 0.8 {
            [1., 0., 0.]
        } else {
            [0., 1., 0.]
        };
        let x = unit(sub(
            reference,
            scale(self.normal, dot(reference, self.normal)),
        ))
        .unwrap();
        let y = cross(self.normal, x);
        let mut points: Vec<_> = (0..count)
            .map(|i| {
                let t = TAU * i as f64 / count as f64;
                add(
                    self.center,
                    scale(add(scale(x, t.cos()), scale(y, t.sin())), self.radius),
                )
            })
            .collect();
        points.push(points[0]);
        points
    }
}
impl Circle {
    pub fn axes(&self) -> [Point; 3] {
        self.axes.unwrap_or(crate::rectangle::AXES)
    }
    pub fn center_mode(&self) -> bool {
        matches!(self.mode, Mode::Center | Mode::Vertical | Mode::AroundCurve)
    }
    pub fn radius(&self, value: f64) -> Result<f64, String> {
        if !value.is_finite() || value <= 0. {
            return Err("Circle size must be finite and positive".into());
        }
        Ok(match self.size {
            Size::Radius => value,
            Size::Diameter => value / 2.,
            Size::Circumference => value / TAU,
            Size::Area => (value / PI).sqrt(),
        })
    }
    pub fn numeric(&self, points: &[Point], text: &str) -> Result<Plan, String> {
        if !self.center_mode() {
            return Err(
                "Numeric size / 3Point Radius constraint is not implemented in this mode".into(),
            );
        }
        if points.len() != 1 || self.orientating {
            return Err("Pick the center and requested orientation before entering size".into());
        }
        let normal = self.normal.unwrap_or(if self.mode == Mode::Vertical {
            self.axes()[1]
        } else {
            self.axes()[2]
        });
        Plan::new(
            points[0],
            normal,
            self.radius(dimension(text, self.size == Size::Area)?)?,
        )
    }
    pub fn diameter_normal(&self, edge: Point) -> Result<Point, String> {
        let direction = unit(edge)?;
        let axes = self.axes();
        let projected = sub(axes[2], scale(direction, dot(axes[2], direction)));
        if norm(projected) > 1e-7 {
            unit(projected)
        } else {
            unit(sub(axes[1], scale(direction, dot(axes[1], direction))))
        }
    }
    pub fn pick(&mut self, points: &mut Vec<Point>, p: Point) -> Result<Option<Plan>, String> {
        check_point(p)?;
        if self.mode == Mode::FitPoints {
            if points.len() >= 1024 {
                return Err("FitPoints accepts at most 1024 points".into());
            }
            points.push(p);
            return Ok(None);
        }
        if self.mode == Mode::Tangent && self.radius_pending {
            let origin = *points
                .last()
                .ok_or("Enter Radius numerically before the first tangent curve")?;
            self.set_radius(points, norm(sub(p, origin)))?;
            return Ok(None);
        }
        if self.mode == Mode::Tangent {
            if !self.point_mode {
                return Err("Pick a native curve, or choose Point for a free point".into());
            }
            self.constraint(points, p, true)?;
            return Ok(None);
        }
        if self.mode == Mode::AroundCurve && points.is_empty() {
            return Err(
                "Select a native curve then pick its center or use OnCurve=fraction".into(),
            );
        }
        if self.mode == Mode::ThreePoint && points.len() == 2 && self.radius_pending {
            let radius = norm(sub(p, points[1]));
            self.set_radius(points, radius)?;
            return Ok(None);
        }
        if points.is_empty() {
            points.push(p);
            return Ok(None);
        }
        if self.orientating {
            let normal = unit(sub(p, points[0]))?;
            self.normal = Some(normal);
            self.orientating = false;
            self.oriented = true;
            return Ok(None);
        }
        let a = points[0];
        let plan = match self.mode {
            Mode::Center | Mode::Vertical | Mode::AroundCurve => {
                let normal = if self.mode == Mode::Vertical {
                    let z = self.axes()[2];
                    let horizontal = sub(sub(p, a), scale(z, dot(sub(p, a), z)));
                    unit(cross(horizontal, z))?
                } else {
                    self.normal.unwrap_or(self.axes()[2])
                };
                let d = sub(p, a);
                let projected = sub(d, scale(normal, dot(d, normal)));
                // Area location displays the area of the radius under the cursor.
                let radius = if self.size == Size::Area {
                    norm(projected)
                } else {
                    self.radius(norm(projected))?
                };
                Plan::new(a, normal, radius)?
            }
            Mode::TwoPoint => {
                let edge = sub(p, a);
                Plan::new(
                    add(a, scale(edge, 0.5)),
                    self.diameter_normal(edge)?,
                    norm(edge) / 2.,
                )?
            }
            Mode::ThreePoint => {
                if points.len() == 1 {
                    unit(sub(p, a))?;
                    points.push(p);
                    return Ok(None);
                }
                let u = sub(points[1], a);
                if let Some(radius) = self.radius_constraint {
                    let direction = unit(u)?;
                    let mid = add(a, scale(u, 0.5));
                    let delta = sub(p, mid);
                    let side = unit(sub(delta, scale(direction, dot(delta, direction))))?;
                    let height = ((radius - norm(u) / 2.) * (radius + norm(u) / 2.))
                        .max(0.)
                        .sqrt();
                    let center = add(mid, scale(side, height));
                    let plan = Plan::new(center, cross(direction, side), radius)?;
                    points.push(p);
                    return Ok(Some(plan));
                }
                let v = sub(p, a);
                let w = cross(u, v);
                unit(v)?;
                unit(sub(p, points[1]))?;
                if norm(w) <= 1e-10 * norm(u) * norm(v) {
                    return Err("Three Circle points must not be collinear".into());
                }
                let offset = scale(
                    add(scale(cross(w, u), dot(v, v)), scale(cross(v, w), dot(u, u))),
                    0.5 / dot(w, w),
                );
                let center = add(a, offset);
                // w carries square-length units; normalize before applying a
                // direction tolerance so valid tiny circles are not rejected.
                Plan::new(center, scale(w, 1. / norm(w)), norm(offset))?
            }
            Mode::FitPoints | Mode::Tangent => unreachable!(),
        };
        points.push(p);
        Ok(Some(plan))
    }
    pub fn undo(&mut self, points: &mut Vec<Point>) -> Result<(), String> {
        if self.radius_pending {
            self.radius_pending = false;
            return Ok(());
        }
        if self.mode == Mode::ThreePoint && self.radius_constraint.take().is_some() {
            return Ok(());
        }
        if self.mode == Mode::Tangent {
            self.constraints
                .pop()
                .ok_or("No tangent constraint to undo")?;
            points.pop();
            self.point_mode = false;
            self.solution = None;
            return Ok(());
        }
        if self.mode == Mode::AroundCurve && points.is_empty() && self.path_selected {
            self.path_selected = false;
            self.normal = None;
            return Ok(());
        }
        if self.orientating {
            self.orientating = false;
            return Ok(());
        }
        if self.oriented {
            self.normal = None;
            self.oriented = false;
            self.orientating = true;
            return Ok(());
        }
        points.pop().ok_or("No Circle point to undo")?;
        Ok(())
    }
    pub fn set_radius(&mut self, points: &[Point], radius: f64) -> Result<(), String> {
        if !radius.is_finite() || !(1e-7..=1e9).contains(&radius) {
            return Err("Radius must be 1e-7 through 1e9 mm".into());
        }
        if self.mode == Mode::ThreePoint {
            if points.len() != 2 {
                return Err("3Point Radius requires two circumference points".into());
            }
            if radius < norm(sub(points[1], points[0])) / 2. {
                return Err("Radius is smaller than half the chord".into());
            }
        } else if self.mode != Mode::Tangent || self.constraints.len() > 2 {
            return Err("Radius constraint requires at most two tangent constraints".into());
        }
        self.radius_constraint = Some(radius);
        self.radius_pending = false;
        self.solution = None;
        Ok(())
    }
    pub fn tangent_ready(&self) -> bool {
        self.mode == Mode::Tangent
            && self.constraints.len()
                == if self.radius_constraint.is_some() {
                    2
                } else {
                    3
                }
    }
    pub fn constraint(
        &mut self,
        points: &mut Vec<Point>,
        p: Point,
        free: bool,
    ) -> Result<(), String> {
        check_point(p)?;
        if self.mode != Mode::Tangent || self.tangent_ready() {
            return Err("Undo a tangent constraint before adding another".into());
        }
        if free && self.from_first && self.constraints.is_empty() {
            return Err("FromFirstPoint requires the first constraint to be a curve".into());
        }
        self.constraints.push((p, free));
        points.push(p);
        self.point_mode = false;
        self.solution = None;
        Ok(())
    }
    pub fn reference_mode(&self, points: &[Point]) -> u32 {
        match self.mode {
            Mode::AroundCurve if points.is_empty() => {
                if self.path_selected {
                    2
                } else {
                    1
                }
            }
            Mode::Tangent => {
                if self.radius_pending {
                    0
                } else if self.tangent_ready() {
                    5
                } else if self.point_mode {
                    0
                } else {
                    3
                }
            }
            Mode::FitPoints => 4,
            _ => 0,
        }
    }
    pub fn pick_plane(&self, points: &[Point]) -> Option<(Point, Point)> {
        let origin = *points.first()?;
        let normal = if self.mode == Mode::ThreePoint && points.len() == 2 {
            self.diameter_normal(sub(points[1], origin)).ok()?
        } else if self.orientating || self.mode == Mode::Vertical {
            self.axes()[2]
        } else {
            self.normal.unwrap_or(self.axes()[2])
        };
        Some((origin, normal))
    }
}
pub fn dimension(text: &str, area: bool) -> Result<f64, String> {
    let lower = text.trim().to_ascii_lowercase();
    let suffixes: &[(&str, f64)] = if area {
        &[
            ("mm^2", 1.),
            ("mm2", 1.),
            ("cm^2", 100.),
            ("cm2", 100.),
            ("in^2", 645.16),
            ("in2", 645.16),
        ]
    } else {
        &[("mm", 1.), ("cm", 10.), ("in", 25.4)]
    };
    let mut value = lower.as_str();
    let mut factor = 1.;
    for (suffix, scale) in suffixes {
        if let Some(number) = lower.strip_suffix(suffix) {
            value = number.trim();
            factor = *scale;
            break;
        }
    }
    let value = value.parse::<f64>().map_err(|_| {
        if area {
            "Enter area in mm2/cm2/in2"
        } else {
            "Enter size in mm/cm/in"
        }
    })? * factor;
    if !value.is_finite() || value <= 0. {
        return Err("Circle size must be finite and positive".into());
    }
    Ok(value)
}
