// SPDX-License-Identifier: LGPL-2.1-or-later
//! Model-space annotation semantics. Lines are a derived pick/display representation;
//! the versioned extension owns values, witnesses, text placement and style.
use crate::{
    Error, Result,
    budget::Limits,
    geometry::{Curve, Geometry, Point3, finite},
    model::{Entity, Extension},
};
use serde::{Deserialize, Serialize};
pub const SLOT: &str = "om9.annotation";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Horizontal,
    Vertical,
    Aligned,
    Rotated,
    Angular,
    Diameter,
    Radius,
    Leader,
    Text,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    pub height_mm: f64,
    pub arrow_mm: f64,
    pub precision: usize,
    pub scale: f64,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            height_mm: 2.5,
            arrow_mm: 1.,
            precision: 2,
            scale: 1.,
        }
    }
}
impl Style {
    pub fn validate(&self) -> Result<()> {
        if !self.height_mm.is_finite()
            || self.height_mm <= 0.
            || !self.arrow_mm.is_finite()
            || self.arrow_mm <= 0.
            || !self.scale.is_finite()
            || self.scale <= 0.
            || self.precision > 8
        {
            return Err(Error::Invalid("annotation style"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub kind: Kind,
    pub points: Vec<Point3>,
    pub x: Point3,
    pub y: Point3,
    pub text_point: Point3,
    pub text: Option<String>,
    pub rotation_degrees: f64,
    pub style: Style,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub lines: Vec<[Point3; 2]>,
    pub text: String,
    pub value: Option<f64>,
    pub center: Point3,
}
fn add(a: Point3, b: Point3) -> Point3 {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point3, b: Point3) -> Point3 {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: Point3, v: f64) -> Point3 {
    a.map(|x| x * v)
}
fn dot(a: Point3, b: Point3) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn length(a: Point3) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: Point3) -> Result<Point3> {
    let n = length(a);
    if n <= 1e-10 {
        return Err(Error::Invalid("dimension has zero direction"));
    }
    Ok(scale(a, 1. / n))
}
fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn midpoint(a: Point3, b: Point3) -> Point3 {
    scale(add(a, b), 0.5)
}
impl Annotation {
    /// Recompute display semantics from the actual occurrence placement.
    pub fn placed(&self, placement: crate::geometry::Transform) -> Result<Self> {
        let mut r = self.clone();
        let origin = placement.point([0.; 3])?;
        let vector = |v: Point3| -> Result<Point3> { Ok(sub(placement.point(v)?, origin)) };
        let x = vector(self.x)?;
        let y = vector(self.y)?;
        let sx = length(x);
        let sy = length(y);
        if matches!(self.kind, Kind::Radius | Kind::Diameter)
            && ((sx - sy).abs() > 1e-8 * sx.max(sy) || dot(unit(x)?, unit(y)?).abs() > 1e-8)
        {
            return Err(Error::Unsupported(
                "radial dimension under a non-circular affine placement",
            ));
        }
        r.x = unit(x)?;
        r.y = unit(cross(unit(cross(x, y))?, r.x))?;
        if self.kind == Kind::Rotated {
            let t = self.rotation_degrees.to_radians();
            let d = vector(add(scale(self.x, t.cos()), scale(self.y, t.sin())))?;
            r.rotation_degrees = dot(d, r.y).atan2(dot(d, r.x)).to_degrees();
        }
        for point in &mut r.points {
            *point = placement.point(*point)?;
        }
        r.text_point = placement.point(r.text_point)?;
        r.style.height_mm *= sy;
        r.style.arrow_mm *= sx;
        Ok(r)
    }
    pub fn read(e: &Entity) -> Result<Self> {
        let ext = e
            .extensions
            .get(SLOT)
            .ok_or(Error::Missing("annotation selection"))?;
        if ext.version != 1 {
            return Err(Error::Unsupported("annotation extension version"));
        }
        serde_json::from_str(&ext.payload).map_err(|_| Error::Invalid("annotation payload"))
    }
    pub fn extension(&self, limits: &Limits) -> Result<Extension> {
        self.layout(limits)?;
        Extension::new(
            1,
            serde_json::to_string(self).map_err(|_| Error::Invalid("annotation serialization"))?,
        )
    }
    pub fn geometry(&self, limits: &Limits) -> Result<Geometry> {
        let layout = self.layout(limits)?;
        let segments: Vec<Curve> = layout
            .lines
            .iter()
            .filter(|[a, b]| length(sub(*a, *b)) > 1e-10)
            .map(|[a, b]| Curve::Line {
                start: *a,
                end: *b,
                domain: [0., 1.],
            })
            .collect();
        if segments.is_empty() {
            Ok(Geometry::Point(self.text_point))
        } else {
            Ok(Geometry::Curve(Curve::Polycurve {
                segments,
                reversed: false,
            }))
        }
    }
    pub fn layout(&self, limits: &Limits) -> Result<Layout> {
        self.style.validate()?;
        finite(self.text_point)?;
        finite(self.x)?;
        finite(self.y)?;
        if (length(self.x) - 1.).abs() > 1e-8
            || (length(self.y) - 1.).abs() > 1e-8
            || dot(self.x, self.y).abs() > 1e-8
        {
            return Err(Error::Invalid("annotation plane"));
        }
        if !self.rotation_degrees.is_finite() {
            return Err(Error::Invalid("dimension rotation"));
        }
        if self.points.len() > limits.max_points as usize {
            return Err(Error::Budget("annotation points"));
        }
        for p in &self.points {
            finite(*p)?;
        }
        if let Some(t) = &self.text
            && (t.len() > 4096 || t.contains('\0'))
        {
            return Err(Error::Invalid("annotation text"));
        }
        let expected = match self.kind {
            Kind::Text => 0,
            Kind::Angular => 3,
            _ => 2,
        };
        if self.points.len() != expected {
            return Err(Error::Invalid("annotation witness count"));
        }
        let mut lines = Vec::new();
        let mut value = None;
        let mut center = self.text_point;
        let normal = cross(self.x, self.y);
        let mut arrows = Vec::new();
        match self.kind {
            Kind::Text => {}
            Kind::Leader => {
                lines.extend([
                    [self.points[0], self.points[1]],
                    [self.points[1], self.text_point],
                ]);
                arrows.push((self.points[0], unit(sub(self.points[1], self.points[0]))?));
            }
            Kind::Angular => {
                let p = self.points[0];
                let a = unit(sub(self.points[1], p))?;
                let b = unit(sub(self.points[2], p))?;
                if dot(a, normal).abs() > 1e-7 || dot(b, normal).abs() > 1e-7 {
                    return Err(Error::Invalid("angular witnesses outside annotation plane"));
                }
                let signed = dot(cross(a, b), normal).atan2(dot(a, b));
                let radians = signed.abs();
                if radians < 1e-9 {
                    return Err(Error::Invalid("zero dimension angle"));
                }
                let radius = length(sub(self.text_point, p));
                if radius < 1e-9 {
                    return Err(Error::Invalid("angular dimension radius"));
                }
                let tangent = scale(cross(normal, a), signed.signum());
                let at = |t: f64| {
                    add(
                        p,
                        scale(add(scale(a, t.cos()), scale(tangent, t.sin())), radius),
                    )
                };
                for i in 0..32 {
                    lines.push([
                        at(radians * i as f64 / 32.),
                        at(radians * (i + 1) as f64 / 32.),
                    ]);
                }
                lines.extend([[p, at(0.)], [p, at(radians)]]);
                center = at(radians / 2.);
                value = Some(radians.to_degrees());
                arrows.push((at(0.), tangent));
                arrows.push((
                    at(radians),
                    scale(
                        add(scale(a, -radians.sin()), scale(tangent, radians.cos())),
                        -1.,
                    ),
                ));
            }
            Kind::Radius | Kind::Diameter => {
                let c = self.points[0];
                let edge = self.points[1];
                let r = length(sub(edge, c));
                let dir = unit(sub(edge, c))?;
                let a = if self.kind == Kind::Diameter {
                    add(c, scale(dir, -r))
                } else {
                    c
                };
                lines.push([a, edge]);
                lines.push([edge, self.text_point]);
                center = midpoint(a, edge);
                arrows.push((edge, scale(dir, -1.)));
                if self.kind == Kind::Diameter {
                    arrows.push((a, dir));
                }
                value = Some(if self.kind == Kind::Diameter {
                    2. * r
                } else {
                    r
                });
            }
            _ => {
                let a = self.points[0];
                let b = self.points[1];
                let direction = match self.kind {
                    Kind::Horizontal => self.x,
                    Kind::Vertical => self.y,
                    Kind::Aligned => unit(sub(b, a))?,
                    Kind::Rotated => {
                        let t = self.rotation_degrees.to_radians();
                        add(scale(self.x, t.cos()), scale(self.y, t.sin()))
                    }
                    _ => unreachable!(),
                };
                if dot(direction, normal).abs() > 1e-7 {
                    return Err(Error::Invalid("aligned dimension outside annotation plane"));
                }
                let distance = dot(sub(b, a), direction);
                if distance.abs() < 1e-10 {
                    return Err(Error::Invalid("zero dimension length"));
                }
                let side = cross(normal, direction);
                let offset = dot(sub(self.text_point, a), side);
                let start = add(a, scale(side, offset));
                let end = add(start, scale(direction, distance));
                lines.extend([[a, start], [b, end], [start, end]]);
                center = midpoint(start, end);
                value = Some(distance.abs());
                arrows.push((start, scale(direction, distance.signum())));
                arrows.push((end, scale(direction, -distance.signum())));
            }
        }
        for (tip, direction) in arrows {
            let side = unit(cross(normal, direction))?;
            let back = add(tip, scale(direction, self.style.arrow_mm));
            for sign in [-1., 1.] {
                lines.push([
                    tip,
                    add(back, scale(side, sign * self.style.arrow_mm * 0.3)),
                ]);
            }
        }
        if value.is_some_and(|v| !v.is_finite()) {
            return Err(Error::Invalid("nonfinite dimension value"));
        }
        for [a, b] in &lines {
            finite(*a)?;
            finite(*b)?;
        }
        let automatic = if let Some(v) = value {
            let prefix = match self.kind {
                Kind::Diameter => "Ø",
                Kind::Radius => "R",
                _ => "",
            };
            let suffix = if self.kind == Kind::Angular {
                "°"
            } else {
                " mm"
            };
            let v = if self.kind == Kind::Angular {
                v
            } else {
                v * self.style.scale
            };
            if !v.is_finite() {
                return Err(Error::Invalid("scaled dimension value"));
            }
            format!("{prefix}{:.*}{suffix}", self.style.precision, v)
        } else {
            String::new()
        };
        let text = self.text.as_ref().map_or(automatic.clone(), |t| {
            if value.is_some() {
                t.replace("<>", &automatic)
            } else {
                t.clone()
            }
        });
        if matches!(self.kind, Kind::Text | Kind::Leader) && text.trim().is_empty() {
            return Err(Error::Invalid("empty annotation text"));
        }
        Ok(Layout {
            lines,
            text,
            value,
            center,
        })
    }
}
