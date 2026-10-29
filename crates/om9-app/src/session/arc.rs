//! Arc command (`Arc`) ported from the Qt Curve controller: three-point and
//! center/start/end construction.

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const ARC: &str = "OM9-CURVE-007";
pub const ARC_START: &str = "OM9-CURVE-008";

#[derive(Debug, Clone)]
pub struct ArcPending {
    pub mode: &'static str,
    pub points: Vec<Point3>,
    pub radius: Option<f64>,
}

impl Default for ArcPending {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcPending {
    pub fn new() -> Self {
        Self {
            mode: "ThreePoint",
            points: Vec::new(),
            radius: None,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match (self.mode, self.points.len()) {
            ("StartPoint", 0) => "Start of arc",
            ("StartPoint", 1) => "Point along the start tangent",
            ("StartPoint", _) => "End of arc",
            ("Center", 0) => "Center of arc",
            ("Center", 1) => "Start of arc",
            ("Center", _) => "End of arc",
            (_, 0) => "Start of arc",
            (_, 1) => "Point through arc",
            (_, _) => "End of arc",
        }
    }
}

impl Controller {
    pub fn dispatch_arc(&mut self) -> Result<(), Error> {
        self.arc_pending = Some(ArcPending::new());
        Ok(())
    }

    pub fn dispatch_arc_start(&mut self) -> Result<(), Error> {
        let mut pending = ArcPending::new();
        pending.mode = "StartPoint";
        self.arc_pending = Some(pending);
        Ok(())
    }

    pub fn submit_arc_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.arc_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            pending.points.pop();
            self.arc_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text
            .trim()
            .strip_prefix("Radius=")
            .or_else(|| text.trim().strip_prefix("Diameter="))
        {
            if pending.mode != "Center" {
                self.arc_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "Radius=/Diameter= requires Center mode",
                )));
            }
            let Some(mut radius) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.arc_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            };
            if text.trim().starts_with("Diameter=") {
                radius *= 0.5;
            }
            pending.radius = Some(radius);
            self.arc_pending = Some(pending);
            return Some(Ok(()));
        }
        let mode = match option.as_str() {
            "center" | "c" => Some("Center"),
            "3point" | "3p" | "3" => Some("ThreePoint"),
            "startpoint" | "direction" => Some("StartPoint"),
            _ => None,
        };
        if let Some(mode) = mode {
            if !pending.points.is_empty() {
                self.arc_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "choose arc mode before the first point",
                )));
            }
            pending.mode = mode;
            pending.points.clear();
            pending.radius = None;
            self.arc_pending = Some(pending);
            return Some(Ok(()));
        }
        self.arc_pending = Some(pending);
        Some(Err(Error::Invalid("arc needs points")))
    }

    pub fn accept_arc_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.arc_pending.take() else {
            return Err(Error::Missing("arc command"));
        };
        pending.points.push(point);
        if pending.points.len() < 3 {
            self.arc_pending = Some(pending);
            return Ok(());
        }
        let (a, b, c) = (pending.points[0], pending.points[1], pending.points[2]);
        self.arc_pending = None;
        let plane = self.input_plane();
        let plane_normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let plane_normal = unit(plane_normal)?;
        if pending.mode == "StartPoint" {
            let start = a;
            let tangent_point = b;
            let end = c;
            let tangent = unit([
                tangent_point[0] - start[0],
                tangent_point[1] - start[1],
                tangent_point[2] - start[2],
            ])?;
            let mut normal = cross(plane_normal, tangent);
            if normal[0].abs() + normal[1].abs() + normal[2].abs() < 1e-12 {
                normal = plane.y;
            }
            let normal = unit(normal)?;
            let delta = [start[0] - end[0], start[1] - end[1], start[2] - end[2]];
            let denominator =
                2.0 * (normal[0] * delta[0] + normal[1] * delta[1] + normal[2] * delta[2]);
            if denominator.abs() <= 1e-12 {
                return Err(Error::Invalid(
                    "arc start direction is collinear with the end",
                ));
            }
            let length_squared = delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2);
            let offset = -length_squared / denominator;
            let center = [
                start[0] + normal[0] * offset,
                start[1] + normal[1] * offset,
                start[2] + normal[2] * offset,
            ];
            let radius = distance(center, start);
            let x = unit([
                start[0] - center[0],
                start[1] - center[1],
                start[2] - center[2],
            ])?;
            let y = unit(cross(plane_normal, x))?;
            let end_angle = signed_angle(x, y, end, center);
            let sweep = if end_angle <= 0.0 {
                end_angle + std::f64::consts::TAU
            } else {
                end_angle
            };
            return self.commit_arc(center, x, y, radius, 0.0, sweep);
        }
        if pending.mode == "Center" {
            let center = a;
            let start = b;
            let end = c;
            let x = unit([
                start[0] - center[0],
                start[1] - center[1],
                start[2] - center[2],
            ])?;
            let radius = pending.radius.unwrap_or_else(|| distance(center, start));
            if pending.radius.is_none()
                && (distance(center, end) - radius).abs() > 1e-6 * radius.max(1.0)
            {
                return Err(Error::Invalid("arc points must share the radius"));
            }
            let y = unit(cross(plane_normal, x))?;
            let start_angle = 0.0;
            let end_angle = signed_angle(x, y, end, center);
            let sweep = if end_angle <= 0.0 {
                end_angle + std::f64::consts::TAU
            } else {
                end_angle
            };
            return self.commit_arc(center, x, y, radius, start_angle, sweep);
        }
        let curve = om9_core::curve_build::arc_through(a, b, c)?;
        self.commit_geometry(ARC, "Arc", Geometry::Curve(curve))
    }

    fn commit_arc(
        &mut self,
        center: Point3,
        x: Point3,
        y: Point3,
        radius: f64,
        start_angle: f64,
        sweep: f64,
    ) -> Result<(), Error> {
        let curve = om9_core::curve_build::arc_by_center(center, x, y, radius, start_angle, sweep)?;
        self.commit_geometry(ARC, "Arc", Geometry::Curve(curve))
    }

    pub fn arc_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.arc_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = Vec::new();
        if pending.points.is_empty() {
            tokens.push(("3Point", "3Point"));
            tokens.push(("Center", "Center"));
            tokens.push(("StartPoint", "StartPoint"));
        }
        if pending.mode == "Center" && pending.points.is_empty() {
            tokens.push(("Radius=", "Radius="));
            tokens.push(("Diameter=", "Diameter="));
        }
        if !pending.points.is_empty() {
            tokens.push(("Undo", "Undo"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }

    #[allow(clippy::collapsible_if)]
    pub fn arc_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.arc_pending.as_ref() else {
            return Vec::new();
        };
        if pending.points.len() == 2 && pending.mode == "StartPoint" {
            let start = pending.points[0];
            let tangent_point = pending.points[1];
            let end = point;
            let plane = self.input_plane();
            let plane_normal = match unit([
                plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
            ]) {
                Ok(normal) => normal,
                Err(_) => return Vec::new(),
            };
            let Ok(tangent) = unit([
                tangent_point[0] - start[0],
                tangent_point[1] - start[1],
                tangent_point[2] - start[2],
            ]) else {
                return Vec::new();
            };
            let mut normal = cross(plane_normal, tangent);
            if normal[0].abs() + normal[1].abs() + normal[2].abs() < 1e-12 {
                normal = plane.y;
            }
            let Ok(normal) = unit(normal) else {
                return Vec::new();
            };
            let delta = [start[0] - end[0], start[1] - end[1], start[2] - end[2]];
            let denominator =
                2.0 * (normal[0] * delta[0] + normal[1] * delta[1] + normal[2] * delta[2]);
            if denominator.abs() <= 1e-12 {
                return Vec::new();
            }
            let length_squared = delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2);
            let offset = -length_squared / denominator;
            let center = [
                start[0] + normal[0] * offset,
                start[1] + normal[1] * offset,
                start[2] + normal[2] * offset,
            ];
            let radius = distance(center, start);
            let Ok(x) = unit([
                start[0] - center[0],
                start[1] - center[1],
                start[2] - center[2],
            ]) else {
                return Vec::new();
            };
            let Ok(y) = unit(cross(plane_normal, x)) else {
                return Vec::new();
            };
            let end_angle = signed_angle(x, y, end, center);
            let sweep = if end_angle <= 0.0 {
                end_angle + std::f64::consts::TAU
            } else {
                end_angle
            };
            return arc_segments(center, x, y, radius, 0.0, sweep, 32);
        }
        if pending.points.len() == 2 && pending.mode == "Center" {
            let center = pending.points[0];
            let start = pending.points[1];
            let plane = self.input_plane();
            let Ok(normal) = unit([
                plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
            ]) else {
                return Vec::new();
            };
            let Ok(x) = unit([
                start[0] - center[0],
                start[1] - center[1],
                start[2] - center[2],
            ]) else {
                return Vec::new();
            };
            let radius = pending.radius.unwrap_or_else(|| distance(center, start));
            if radius <= 1e-9 {
                return Vec::new();
            }
            let y = match unit(cross(normal, x)) {
                Ok(y) => y,
                Err(_) => return Vec::new(),
            };
            let end_angle = signed_angle(x, y, point, center);
            let sweep = if end_angle <= 0.0 {
                end_angle + std::f64::consts::TAU
            } else {
                end_angle
            };
            return arc_segments(center, x, y, radius, 0.0, sweep, 32);
        }
        if pending.points.len() == 2
            && let Ok(Curve::Circle {
                center,
                x,
                y,
                radius,
                angle_domain,
                ..
            }) = om9_core::curve_build::arc_through(pending.points[0], pending.points[1], point)
        {
            let angles = angle_domain.unwrap_or([0.0, std::f64::consts::TAU]);
            let steps = 32;
            let point_at = |angle: f64| {
                let (sin, cos) = angle.sin_cos();
                [
                    center[0] + x[0] * radius * cos + y[0] * radius * sin,
                    center[1] + x[1] * radius * cos + y[1] * radius * sin,
                    center[2] + x[2] * radius * cos + y[2] * radius * sin,
                ]
            };
            return (0..steps)
                .map(|index| {
                    let first = angles[0] + (angles[1] - angles[0]) * index as f64 / steps as f64;
                    let second =
                        angles[0] + (angles[1] - angles[0]) * (index + 1) as f64 / steps as f64;
                    [point_at(first), point_at(second)]
                })
                .collect();
        }
        pending
            .points
            .iter()
            .copied()
            .chain([point])
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| [pair[0], pair[1]])
            .collect()
    }
}

fn unit(vector: Point3) -> Result<Point3, Error> {
    let length = (vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2)).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("arc frame"));
    }
    Ok([vector[0] / length, vector[1] / length, vector[2] / length])
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn signed_angle(x: Point3, y: Point3, point: Point3, center: Point3) -> f64 {
    let dx = point[0] - center[0];
    let dy = point[1] - center[1];
    let dz = point[2] - center[2];
    let cos = dx * x[0] + dy * x[1] + dz * x[2];
    let sin = dx * y[0] + dy * y[1] + dz * y[2];
    sin.atan2(cos)
}

fn arc_segments(
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    start: f64,
    sweep: f64,
    steps: usize,
) -> Vec<[Point3; 2]> {
    let point_at = |angle: f64| {
        let (sin, cos) = angle.sin_cos();
        [
            center[0] + x[0] * radius * cos + y[0] * radius * sin,
            center[1] + x[1] * radius * cos + y[1] * radius * sin,
            center[2] + x[2] * radius * cos + y[2] * radius * sin,
        ]
    };
    (0..steps)
        .map(|index| {
            let first = start + sweep * index as f64 / steps as f64;
            let second = start + sweep * (index + 1) as f64 / steps as f64;
            [point_at(first), point_at(second)]
        })
        .collect()
}
