//! Ellipse command (Center, major/minor radii) ported from the Qt Curve controller.

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const ELLIPSE: &str = "OM9-CURVE-006";

#[derive(Debug, Clone)]
pub struct EllipsePending {
    pub mode: &'static str,
    pub points: Vec<Point3>,
    pub center: Option<Point3>,
    pub x_axis: Point3,
    pub radius_x: Option<f64>,
    pub radius_y: Option<f64>,
    pub mark_foci: bool,
    pub path: Option<Curve>,
    pub normal: Option<Point3>,
    pub deformable: bool,
    pub degree: usize,
    pub point_count: usize,
}

impl Default for EllipsePending {
    fn default() -> Self {
        Self::new()
    }
}

impl EllipsePending {
    pub fn new() -> Self {
        Self {
            mode: "Center",
            points: Vec::new(),
            center: None,
            x_axis: [1.0, 0.0, 0.0],
            radius_x: None,
            radius_y: None,
            mark_foci: false,
            path: None,
            normal: None,
            deformable: false,
            degree: 3,
            point_count: 32,
        }
    }

    pub fn prompt(&self) -> &'static str {
        if self.mode == "Diameter" {
            return match self.points.len() {
                0 => "First diameter point",
                1 => "Second diameter point",
                _ => "Minor radius point",
            };
        }
        if self.mode == "Vertical" {
            return match self.points.len() {
                0 => "Center of ellipse",
                1 => "Vertical radius point",
                _ => "Horizontal radius point",
            };
        }
        if self.mode == "FromFoci" {
            return match self.points.len() {
                0 => "First focus",
                1 => "Second focus",
                _ => "Point on ellipse",
            };
        }
        if self.mode == "Corner" {
            return if self.center.is_none() {
                "First corner of ellipse box"
            } else {
                "Opposite corner of ellipse box"
            };
        }
        if self.center.is_none() {
            "Center of ellipse"
        } else if self.radius_x.is_none() {
            "Major radius"
        } else {
            "Minor radius"
        }
    }
}

impl Controller {
    pub fn dispatch_ellipse(&mut self) -> Result<(), Error> {
        let path = self.selected_ordered().into_iter().find_map(|id| {
            let entity = self.document().entities.get(&id)?;
            match &entity.geometry {
                Geometry::Curve(curve) => Some(curve.clone()),
                _ => None,
            }
        });
        let mut pending = EllipsePending::new();
        pending.path = path;
        self.ellipse_pending = Some(pending);
        Ok(())
    }

    pub fn submit_ellipse_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.ellipse_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.radius_y.take().is_none()
                && pending.radius_x.take().is_none()
                && pending.center.take().is_none()
            {
                // already cleared
            }
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(mode) = match option.as_str() {
            "center" | "c" => Some("Center"),
            "corner" => Some("Corner"),
            "diameter" | "d" => Some("Diameter"),
            "vertical" | "v" => Some("Vertical"),
            "fromfoci" | "foci" | "f" => Some("FromFoci"),
            "aroundcurve" | "ac" => Some("AroundCurve"),
            _ => None,
        } {
            pending.mode = mode;
            pending.points.clear();
            pending.center = None;
            pending.radius_x = None;
            pending.radius_y = None;
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "deformable" || option == "approx" {
            pending.deformable = !pending.deformable;
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text
            .trim()
            .strip_prefix("Deformable=")
            .or_else(|| text.trim().strip_prefix("Approx="))
        {
            let value = value.trim().to_ascii_lowercase();
            pending.deformable = matches!(value.as_str(), "on" | "yes" | "true" | "1");
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value.parse::<usize>().ok().filter(|value| (1..=11).contains(value)) else {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be 1..11")));
            };
            pending.degree = degree;
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("PointCount=") {
            let Some(points) = value.parse::<usize>().ok().filter(|value| (4..=256).contains(value)) else {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("point count must be 4..256")));
            };
            pending.point_count = points;
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("MarkFoci=") {
            let value = value.trim().to_ascii_lowercase();
            pending.mark_foci = matches!(value.as_str(), "on" | "yes" | "true" | "1");
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "markfoci" {
            pending.mark_foci = !pending.mark_foci;
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.mode == "AroundCurve" && pending.center.is_none() {
            let Some(value) = text.trim().strip_prefix("OnCurve=") else {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("AroundCurve needs OnCurve=fraction")));
            };
            let Some(fraction) = value.parse::<f64>().ok().filter(|value| (0.0..=1.0).contains(value)) else {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("OnCurve fraction must be 0..1")));
            };
            let Some(path) = pending.path.clone() else {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("select a curve before AroundCurve")));
            };
            let center = match om9_core::curve_ops::point_at(&path, fraction, self.coordinator.limits()) {
                Ok(point) => point,
                Err(error) => return Some(Err(error)),
            };
            let tangent = match om9_core::curve_ops::tangent_at(&path, fraction, self.coordinator.limits()) {
                Ok(tangent) => tangent,
                Err(error) => return Some(Err(error)),
            };
            let norm = (tangent[0].powi(2) + tangent[1].powi(2) + tangent[2].powi(2)).sqrt();
            if norm <= 1e-12 {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("path tangent")));
            }
            let normal = [tangent[0] / norm, tangent[1] / norm, tangent[2] / norm];
            let plane = self.input_plane();
            let dot = plane.x[0] * normal[0] + plane.x[1] * normal[1] + plane.x[2] * normal[2];
            let mut x = [
                plane.x[0] - normal[0] * dot,
                plane.x[1] - normal[1] * dot,
                plane.x[2] - normal[2] * dot,
            ];
            let mut x_norm = (x[0].powi(2) + x[1].powi(2) + x[2].powi(2)).sqrt();
            if x_norm <= 1e-12 {
                x = plane.y;
                x_norm = (x[0].powi(2) + x[1].powi(2) + x[2].powi(2)).sqrt();
            }
            if x_norm <= 1e-12 {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("path frame")));
            }
            x = [x[0] / x_norm, x[1] / x_norm, x[2] / x_norm];
            pending.center = Some(center);
            pending.normal = Some(normal);
            pending.x_axis = x;
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("RadiusX=") {
            let Some(radius) = value.parse::<f64>().ok().filter(|value| *value > 0.0) else {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            };
            pending.radius_x = Some(radius);
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("RadiusY=") {
            let Some(radius) = value.parse::<f64>().ok().filter(|value| *value > 0.0) else {
                self.ellipse_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            };
            pending.radius_y = Some(radius);
            self.ellipse_pending = Some(pending);
            return Some(Ok(()));
        }
        if text.trim().is_empty()
            && pending.center.is_some()
            && pending.radius_x.is_some()
            && pending.radius_y.is_some()
        {
            self.ellipse_pending = Some(pending);
            return Some(self.commit_ellipse());
        }
        self.ellipse_pending = Some(pending);
        Some(Err(Error::Invalid("ellipse needs points or RadiusX=/RadiusY=")))
    }

    pub fn accept_ellipse_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.ellipse_pending.take() else {
            return Err(Error::Missing("ellipse command"));
        };
        if pending.mode == "Diameter" {
            pending.points.push(point);
            if pending.points.len() == 2 {
                let a = pending.points[0];
                let b = pending.points[1];
                let center = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5];
                let delta = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let radius = (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt() * 0.5;
                if radius <= 1e-9 {
                    self.ellipse_pending = Some(pending);
                    return Err(Error::Invalid("ellipse diameter must be positive"));
                }
                pending.center = Some(center);
                pending.radius_x = Some(radius);
                pending.x_axis = [delta[0] / (radius * 2.0), delta[1] / (radius * 2.0), delta[2] / (radius * 2.0)];
                self.ellipse_pending = Some(pending);
                return Ok(());
            }
            if pending.points.len() >= 3 {
                let Some(center) = pending.center else {
                    self.ellipse_pending = Some(pending);
                    return Err(Error::Invalid("ellipse diameter center"));
                };
                let radius_y = distance(center, point);
                if radius_y <= 1e-9 {
                    self.ellipse_pending = Some(pending);
                    return Err(Error::Invalid("ellipse radius must be positive"));
                }
                pending.radius_y = Some(radius_y);
                self.ellipse_pending = Some(pending);
                return self.commit_ellipse();
            }
            self.ellipse_pending = Some(pending);
            return Ok(());
        }
        if pending.mode == "Vertical" {
            pending.points.push(point);
            if pending.points.len() == 1 {
                pending.center = Some(point);
                self.ellipse_pending = Some(pending);
                return Ok(());
            }
            let Some(center) = pending.center else {
                self.ellipse_pending = Some(pending);
                return Err(Error::Invalid("ellipse center"));
            };
            let radius = distance(center, point);
            if radius <= 1e-9 {
                self.ellipse_pending = Some(pending);
                return Err(Error::Invalid("ellipse radius must be positive"));
            }
            if pending.points.len() == 2 {
                pending.radius_y = Some(radius);
                pending.x_axis = self.input_plane().x;
                self.ellipse_pending = Some(pending);
                return Ok(());
            }
            pending.radius_x = Some(radius);
            self.ellipse_pending = Some(pending);
            return self.commit_ellipse();
        }
        if pending.mode == "FromFoci" {
            pending.points.push(point);
            if pending.points.len() < 3 {
                self.ellipse_pending = Some(pending);
                return Ok(());
            }
            let first = pending.points[0];
            let second = pending.points[1];
            let on_curve = pending.points[2];
            let center = [
                (first[0] + second[0]) * 0.5,
                (first[1] + second[1]) * 0.5,
                (first[2] + second[2]) * 0.5,
            ];
            let foci = distance(first, second);
            let c = foci * 0.5;
            let a = (distance(first, on_curve) + distance(second, on_curve)) * 0.5;
            let b = (a * a - c * c).sqrt();
            if a <= 1e-9 || b <= 1e-9 {
                self.ellipse_pending = Some(pending);
                return Err(Error::Invalid("ellipse foci are degenerate"));
            }
            let direction = [
                (second[0] - first[0]) / foci,
                (second[1] - first[1]) / foci,
                (second[2] - first[2]) / foci,
            ];
            pending.center = Some(center);
            pending.radius_x = Some(a);
            pending.radius_y = Some(b);
            pending.x_axis = direction;
            let mark = pending.mark_foci;
            self.ellipse_pending = Some(pending);
            let result = self.commit_ellipse();
            if result.is_ok() && mark {
                for focus in [first, second] {
                    self.commit_geometry(
                        ELLIPSE,
                        "EllipseFoci",
                        Geometry::Point(focus),
                    )?;
                }
            }
            return result;
        }
        if pending.mode == "Corner" {
            if pending.center.is_none() {
                pending.center = Some(point);
                self.ellipse_pending = Some(pending);
                return Ok(());
            }
            let Some(first) = pending.center else {
                self.ellipse_pending = Some(pending);
                return Err(Error::Invalid("ellipse box first corner"));
            };
            let second = point;
            let plane = self.input_plane();
            let delta = [second[0] - first[0], second[1] - first[1], second[2] - first[2]];
            let along_x = delta[0] * plane.x[0] + delta[1] * plane.x[1] + delta[2] * plane.x[2];
            let along_y = delta[0] * plane.y[0] + delta[1] * plane.y[1] + delta[2] * plane.y[2];
            let radius_x = along_x.abs() * 0.5;
            let radius_y = along_y.abs() * 0.5;
            if radius_x <= 1e-9 || radius_y <= 1e-9 {
                self.ellipse_pending = Some(pending);
                return Err(Error::Invalid("ellipse box must be non-degenerate"));
            }
            let center = [
                first[0] + plane.x[0] * along_x * 0.5 + plane.y[0] * along_y * 0.5,
                first[1] + plane.x[1] * along_x * 0.5 + plane.y[1] * along_y * 0.5,
                first[2] + plane.x[2] * along_x * 0.5 + plane.y[2] * along_y * 0.5,
            ];
            pending.center = Some(center);
            pending.radius_x = Some(radius_x);
            pending.radius_y = Some(radius_y);
            pending.x_axis = plane.x;
            self.ellipse_pending = Some(pending);
            return self.commit_ellipse();
        }
        if pending.center.is_none() {
            pending.center = Some(point);
            self.ellipse_pending = Some(pending);
            return Ok(());
        }
        let Some(center) = pending.center else {
            self.ellipse_pending = Some(pending);
            return Err(Error::Invalid("ellipse center"));
        };
        if pending.radius_x.is_none() {
            let delta = [point[0] - center[0], point[1] - center[1], point[2] - center[2]];
            let radius = (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt();
            if radius <= 1e-9 {
                self.ellipse_pending = Some(pending);
                return Err(Error::Invalid("ellipse radius must be positive"));
            }
            pending.radius_x = Some(radius);
            pending.x_axis = [delta[0] / radius, delta[1] / radius, delta[2] / radius];
            self.ellipse_pending = Some(pending);
            return Ok(());
        }
        let Some(center) = pending.center else {
            self.ellipse_pending = Some(pending);
            return Err(Error::Invalid("ellipse center"));
        };
        let radius_y = distance(center, point);
        if radius_y <= 1e-9 {
            self.ellipse_pending = Some(pending);
            return Err(Error::Invalid("ellipse radius must be positive"));
        }
        pending.radius_y = Some(radius_y);
        self.ellipse_pending = Some(pending);
        self.commit_ellipse()
    }

    fn commit_ellipse(&mut self) -> Result<(), Error> {
        let Some(pending) = self.ellipse_pending.take() else {
            return Err(Error::Missing("ellipse command"));
        };
        let Some(center) = pending.center else {
            return Err(Error::Invalid("ellipse center"));
        };
        let Some(radius_x) = pending.radius_x else {
            return Err(Error::Invalid("ellipse major radius"));
        };
        let Some(radius_y) = pending.radius_y else {
            return Err(Error::Invalid("ellipse minor radius"));
        };
        let x = pending.x_axis;
        let plane = self.input_plane();
        let normal = pending.normal.unwrap_or([
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ]);
        let mut y = [
            normal[1] * x[2] - normal[2] * x[1],
            normal[2] * x[0] - normal[0] * x[2],
            normal[0] * x[1] - normal[1] * x[0],
        ];
        let norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
        if norm <= 1e-12 {
            y = [0.0, 1.0, 0.0];
        } else {
            y = [y[0] / norm, y[1] / norm, y[2] / norm];
        }
        let exact = om9_core::curve_build::ellipse(
            center,
            x,
            y,
            radius_x,
            radius_y,
            self.coordinator.limits(),
        )?;
        let curve = if pending.deformable {
            if pending.point_count <= pending.degree {
                return Err(Error::Invalid("PointCount must exceed Degree"));
            }
            let rebuilt = om9_core::curve_build::rebuild(
                &Curve::Nurbs(exact),
                pending.degree,
                pending.point_count,
                om9_core::curve_build::KnotMode::ChordLength,
                self.coordinator.limits(),
            )?;
            Curve::Nurbs(rebuilt)
        } else {
            Curve::Nurbs(exact)
        };
        let result = self.commit_geometry(ELLIPSE, "Ellipse", Geometry::Curve(curve));
        if result.is_ok() && pending.mark_foci && pending.mode != "FromFoci" {
            let c_squared = radius_x * radius_x - radius_y * radius_y;
            if c_squared > 1e-18 {
                let c = c_squared.sqrt();
                for sign in [-1.0, 1.0] {
                    let focus = [
                        center[0] + x[0] * c * sign,
                        center[1] + x[1] * c * sign,
                        center[2] + x[2] * c * sign,
                    ];
                    self.commit_geometry(ELLIPSE, "EllipseFoci", Geometry::Point(focus))?;
                }
            }
        }
        result
    }

    pub fn ellipse_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.ellipse_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = Vec::new();
        if pending.points.is_empty() && pending.center.is_none() {
            tokens.extend([
                ("Center", "Center"),
                ("Corner", "Corner"),
                ("Diameter", "Diameter"),
                ("Vertical", "Vertical"),
                ("FromFoci", "FromFoci"),
                ("AroundCurve", "AroundCurve"),
            ]);
        }
        if pending.mode == "AroundCurve" && pending.center.is_none() {
            tokens.push(("OnCurve=", "OnCurve="));
        }
        if pending.mode == "FromFoci" || pending.center.is_some() {
            tokens.push(("MarkFoci", "MarkFoci"));
        }
        tokens.push(("Deformable", "Deformable"));
        tokens.push(("Approx", "Approx"));
        if pending.deformable {
            tokens.push(("Degree=", "Degree="));
            tokens.push(("PointCount=", "PointCount="));
        }
        if pending.center.is_some() {
            tokens.push(("RadiusX=", "RadiusX="));
            tokens.push(("RadiusY=", "RadiusY="));
        }
        if pending.center.is_some() || !pending.points.is_empty() {
            tokens.push(("Undo", "Undo"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }

    pub fn ellipse_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.ellipse_pending.as_ref() else {
            return Vec::new();
        };
        let Some(center) = pending.center else {
            return Vec::new();
        };
        let (radius_x, radius_y, x_axis) = if let Some(radius_x) = pending.radius_x {
            (radius_x, pending.radius_y.unwrap_or(distance(center, point)), pending.x_axis)
        } else {
            let delta = [point[0] - center[0], point[1] - center[1], point[2] - center[2]];
            let radius = (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt();
            if radius <= 1e-9 {
                return Vec::new();
            }
            (radius, radius, [delta[0] / radius, delta[1] / radius, delta[2] / radius])
        };
        if radius_x <= 1e-9 || radius_y <= 1e-9 {
            return Vec::new();
        }
        let plane = self.input_plane();
        let normal = pending.normal.unwrap_or([
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ]);
        let mut y = [
            normal[1] * x_axis[2] - normal[2] * x_axis[1],
            normal[2] * x_axis[0] - normal[0] * x_axis[2],
            normal[0] * x_axis[1] - normal[1] * x_axis[0],
        ];
        let norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
        if norm <= 1e-12 {
            y = [0.0, 1.0, 0.0];
        } else {
            y = [y[0] / norm, y[1] / norm, y[2] / norm];
        }
        let point_at = |angle: f64| {
            let (sin, cos) = angle.sin_cos();
            [
                center[0] + x_axis[0] * radius_x * cos + y[0] * radius_y * sin,
                center[1] + x_axis[1] * radius_x * cos + y[1] * radius_y * sin,
                center[2] + x_axis[2] * radius_x * cos + y[2] * radius_y * sin,
            ]
        };
        (0..64)
            .map(|index| {
                let first = std::f64::consts::TAU * index as f64 / 64.0;
                let second = std::f64::consts::TAU * (index + 1) as f64 / 64.0;
                [point_at(first), point_at(second)]
            })
            .collect()
    }
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
