//! Circle command behaviour ported from the FreeCAD Qt Curve controller:
//! Center+radius, diameter (2Point) and circumcircle (3Point).

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const CIRCLE: &str = "OM9-CURVE-005";

#[derive(Debug, Clone)]
pub struct CirclePending {
    pub mode: &'static str,
    pub size: &'static str,
    pub points: Vec<Point3>,
    pub path: Option<Curve>,
    pub tangent_curves: Vec<Curve>,
    pub normal: Option<Point3>,
    pub radius: Option<f64>,
    pub solution: usize,
    pub tangent_point: Option<Point3>,
    pub deformable: bool,
    pub degree: usize,
    pub point_count: usize,
}

impl Default for CirclePending {
    fn default() -> Self {
        Self::new()
    }
}

impl CirclePending {
    pub fn new() -> Self {
        Self {
            mode: "Center",
            size: "Radius",
            points: Vec::new(),
            path: None,
            tangent_curves: Vec::new(),
            normal: None,
            radius: None,
            solution: 1,
            tangent_point: None,
            deformable: false,
            degree: 3,
            point_count: 32,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match (self.mode, self.points.len()) {
            ("Center", 0) => "Center of circle",
            ("Center", _) => "Radius",
            ("TwoPoint", 0) => "First diameter point",
            ("TwoPoint", _) => "Second diameter point",
            ("ThreePoint", 0) => "First point on circle",
            ("ThreePoint", 1) => "Second point on circle",
            ("ThreePoint", _) => "Third point on circle",
            ("Vertical", 0) => "Center of circle",
            ("Vertical", _) => "Radius or point on circle",
            ("FitPoints", 0) => "First fit point",
            ("FitPoints", _) => "Next fit point or Enter to fit",
            ("AroundCurve", 0) => "OnCurve=fraction or point on curve",
            ("AroundCurve", _) => "Radius or point on circle",
            ("Tangent", _) if self.size == "Radius" => "Radius=, or Enter after Solution=",
            ("Tangent", _) => "Solution=1/2 or Enter",
            _ => "Circle",
        }
    }
}

impl Controller {
    pub fn dispatch_circle(&mut self) -> Result<(), Error> {
        let curves: Vec<Curve> = self
            .selected_ordered()
            .into_iter()
            .filter_map(|id| {
                let entity = self.document().entities.get(&id)?;
                match &entity.geometry {
                    Geometry::Curve(curve) => Some(curve.clone()),
                    _ => None,
                }
            })
            .collect();
        let mut pending = CirclePending::new();
        pending.path = curves.first().cloned();
        pending.tangent_curves = curves.into_iter().take(2).collect();
        self.circle_pending = Some(pending);
        Ok(())
    }

    pub fn submit_circle_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.circle_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            pending.points.pop();
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = option.strip_prefix("size=") {
            let size = match value {
                "radius" | "r" => "Radius",
                "diameter" | "d" => "Diameter",
                "circumference" | "c" => "Circumference",
                "area" | "a" => "Area",
                _ => {
                    self.circle_pending = Some(pending);
                    return Some(Err(Error::Invalid("Size must be Radius, Diameter, Circumference or Area")));
                }
            };
            pending.size = size;
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "deformable" || option == "approx" {
            pending.deformable = !pending.deformable;
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text
            .trim()
            .strip_prefix("Deformable=")
            .or_else(|| text.trim().strip_prefix("Approx="))
        {
            pending.deformable = matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "on" | "yes" | "true" | "1"
            );
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value.parse::<usize>().ok().filter(|value| (1..=11).contains(value)) else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be 1..11")));
            };
            pending.degree = degree;
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("PointCount=") {
            let Some(count) = value.parse::<usize>().ok().filter(|value| *value >= 4) else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("point count must be at least four")));
            };
            pending.point_count = count;
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(mode) = match option.as_str() {
            "center" | "c" => Some("Center"),
            "2point" | "2p" | "2" => Some("TwoPoint"),
            "3point" | "3p" | "3" => Some("ThreePoint"),
            "vertical" | "v" => Some("Vertical"),
            "fitpoints" | "fit" => Some("FitPoints"),
            "aroundcurve" | "ac" => Some("AroundCurve"),
            "tangent" | "tan" => Some("Tangent"),
            _ => None,
        } {
            pending.mode = mode;
            pending.points.clear();
            pending.normal = None;
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.mode == "Tangent" {
            if let Some(value) = text.trim().strip_prefix("Radius=") {
                let Some(radius) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                    self.circle_pending = Some(pending);
                    return Some(Err(Error::Invalid("radius must be positive")));
                };
                pending.size = "Radius";
                pending.radius = Some(radius);
                self.circle_pending = Some(pending);
                return Some(Ok(()));
            }
            if let Some(value) = text.trim().strip_prefix("Point=") {
                let parts: Vec<f64> = value
                    .split(',')
                    .filter_map(|part| part.trim().parse::<f64>().ok())
                    .collect();
                if parts.len() != 3 || parts.iter().any(|value| !value.is_finite()) {
                    self.circle_pending = Some(pending);
                    return Some(Err(Error::Invalid("Point=x,y,z")));
                }
                pending.tangent_point = Some([parts[0], parts[1], parts[2]]);
                self.circle_pending = Some(pending);
                return Some(Ok(()));
            }
            if let Some(value) = text.trim().strip_prefix("Solution=") {
                let Some(solution) = value.parse::<usize>().ok().filter(|value| (1..=4).contains(value)) else {
                    self.circle_pending = Some(pending);
                    return Some(Err(Error::Invalid("solution must be 1..4")));
                };
                pending.solution = solution;
                self.circle_pending = Some(pending);
                return Some(Ok(()));
            }
            if !text.trim().is_empty() {
                if let Some(radius) = text
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value > 0.0)
                {
                    pending.size = "Radius";
                    pending.radius = Some(radius);
                    self.circle_pending = Some(pending);
                    return Some(Ok(()));
                }
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("Tangent accepts Radius=, Solution= or Enter")));
            }
            let Some(radius) = pending.radius else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("Radius= is required")));
            };
            let plane = self.input_plane();
            let normal = [
                plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
            ];
            let (center, x, y) = if pending.tangent_curves.len() == 2 {
                match tangent_line_circle(
                    &pending.tangent_curves,
                    radius,
                    pending.solution,
                    normal,
                    plane.x,
                ) {
                    Ok(frame) => frame,
                    Err(error) => return Some(Err(error)),
                }
            } else if pending.tangent_curves.len() == 1 {
                let Some(point) = pending.tangent_point else {
                    self.circle_pending = Some(pending);
                    return Some(Err(Error::Invalid("Tangent needs a point or a second line")));
                };
                match tangent_line_point_circle(
                    &pending.tangent_curves[0],
                    point,
                    radius,
                    pending.solution,
                    normal,
                    plane.x,
                ) {
                    Ok(frame) => frame,
                    Err(error) => return Some(Err(error)),
                }
            } else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("Tangent needs a line and a point or two lines")));
            };
            self.circle_rebuild = Some((pending.deformable, pending.degree, pending.point_count));
            self.circle_pending = None;
            return Some(self.commit_circle_center(center, x, y, radius));
        }
        if pending.mode == "AroundCurve" && pending.points.is_empty() {
            let Some(value) = text.trim().strip_prefix("OnCurve=") else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("AroundCurve needs OnCurve=fraction")));
            };
            let Some(fraction) = value.parse::<f64>().ok().filter(|value| (0.0..=1.0).contains(value)) else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("OnCurve fraction must be 0..1")));
            };
            let Some(path) = pending.path.clone() else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("select a curve before AroundCurve")));
            };
            let point = match om9_core::curve_ops::point_at(&path, fraction, self.coordinator.limits()) {
                Ok(point) => point,
                Err(error) => return Some(Err(error)),
            };
            let tangent = match om9_core::curve_ops::tangent_at(&path, fraction, self.coordinator.limits()) {
                Ok(tangent) => tangent,
                Err(error) => return Some(Err(error)),
            };
            pending.points.push(point);
            pending.normal = Some(tangent);
            self.circle_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.mode == "FitPoints" {
            if text.trim().is_empty() {
                if pending.points.len() < 3 {
                    self.circle_pending = Some(pending);
                    return Some(Err(Error::Invalid("FitPoints needs at least three points")));
                }
                let points = pending.points.clone();
                self.circle_rebuild = Some((pending.deformable, pending.degree, pending.point_count));
            self.circle_pending = None;
                let (center, x, y, radius) = match fit_circle(&points) {
                    Ok(fit) => fit,
                    Err(error) => return Some(Err(error)),
                };
                return Some(self.commit_circle_center(center, x, y, radius));
            }
            self.circle_pending = Some(pending);
            return Some(Err(Error::Invalid("FitPoints accepts points, Undo or Enter")));
        }
        if matches!(pending.mode, "Center" | "Vertical" | "AroundCurve") && pending.points.len() == 1 {
            let trimmed = text.trim();
            let (raw, size) = if let Some(value) = trimmed.strip_prefix("Radius=") {
                (value, "Radius")
            } else if let Some(value) = trimmed.strip_prefix("Diameter=") {
                (value, "Diameter")
            } else if let Some(value) = trimmed.strip_prefix("Circumference=") {
                (value, "Circumference")
            } else if let Some(value) = trimmed.strip_prefix("Area=") {
                (value, "Area")
            } else {
                (trimmed, pending.size)
            };
            let Some(value) = raw
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            };
            let radius = match size {
                "Diameter" => value * 0.5,
                "Circumference" => value / std::f64::consts::TAU,
                "Area" => (value / std::f64::consts::PI).sqrt(),
                _ => value,
            };
            if !radius.is_finite() || radius <= 0.0 {
                self.circle_pending = Some(pending);
                return Some(Err(Error::Invalid("circle size must be positive")));
            }
            let center = pending.points[0];
            let plane = self.input_plane();
            self.circle_rebuild = Some((pending.deformable, pending.degree, pending.point_count));
            self.circle_pending = None;
            if pending.mode == "Vertical" {
                return Some(self.commit_circle_normal(center, plane.y, radius));
            }
            if pending.mode == "AroundCurve" {
                let normal = pending.normal.unwrap_or_else(|| {
                    [
                        plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                        plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                        plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                    ]
                });
                return Some(self.commit_circle_normal(center, normal, radius));
            }
            return Some(self.commit_circle_center(center, plane.x, plane.y, radius));
        }
        self.circle_pending = Some(pending);
        Some(Err(Error::Invalid("circle needs points or a radius")))
    }

    pub fn accept_circle_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.circle_pending.take() else {
            return Err(Error::Missing("circle command"));
        };
        if pending.mode == "AroundCurve" && pending.points.is_empty() {
            let Some(path) = pending.path.clone() else {
                self.circle_pending = Some(pending);
                return Err(Error::Invalid("select a curve before AroundCurve"));
            };
            let (fraction, projected, _) =
                om9_core::curve_surface_ops::closest_point_on_curve(
                    &path,
                    point,
                    self.coordinator.limits(),
                )?;
            let tangent = om9_core::curve_ops::tangent_at(
                &path,
                fraction,
                self.coordinator.limits(),
            )?;
            pending.points.push(projected);
            pending.normal = Some(tangent);
            self.circle_pending = Some(pending);
            return Ok(());
        }
        pending.points.push(point);
        match pending.mode {
            "Vertical" if pending.points.len() >= 2 => {
                let center = pending.points[0];
                let point = pending.points[1];
                let plane = self.input_plane();
                let d = [point[0] - center[0], point[1] - center[1], point[2] - center[2]];
                let z = [
                    plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                    plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                    plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                ];
                let dot_dz = d[0] * z[0] + d[1] * z[1] + d[2] * z[2];
                let horizontal = [d[0] - z[0] * dot_dz, d[1] - z[1] * dot_dz, d[2] - z[2] * dot_dz];
                let normal = [
                    horizontal[1] * z[2] - horizontal[2] * z[1],
                    horizontal[2] * z[0] - horizontal[0] * z[2],
                    horizontal[0] * z[1] - horizontal[1] * z[0],
                ];
                let radius = (horizontal[0].powi(2) + horizontal[1].powi(2) + horizontal[2].powi(2)).sqrt();
                if radius <= 1e-9 {
                    self.circle_pending = Some(pending);
                    return Err(Error::Invalid("circle radius must be positive"));
                }
                self.circle_rebuild = Some((pending.deformable, pending.degree, pending.point_count));
            self.circle_pending = None;
                self.commit_circle_normal(center, normal, radius)
            }
            "Center" if pending.points.len() >= 2 => {
                let center = pending.points[0];
                let radius = distance(center, pending.points[1]);
                if radius <= 1e-9 {
                    self.circle_pending = Some(pending);
                    return Err(Error::Invalid("circle radius must be positive"));
                }
                let plane = self.input_plane();
                self.circle_rebuild = Some((pending.deformable, pending.degree, pending.point_count));
            self.circle_pending = None;
                self.commit_circle_center(center, plane.x, plane.y, radius)
            }
            "TwoPoint" if pending.points.len() >= 2 => {
                let a = pending.points[0];
                let b = pending.points[1];
                self.circle_rebuild = Some((pending.deformable, pending.degree, pending.point_count));
            self.circle_pending = None;
                self.commit_circle_diameter(a, b)
            }
            "FitPoints" => {
                self.circle_pending = Some(pending);
                Ok(())
            }
            "ThreePoint" if pending.points.len() >= 3 => {
                let a = pending.points[0];
                let b = pending.points[1];
                let c = pending.points[2];
                self.circle_rebuild = Some((pending.deformable, pending.degree, pending.point_count));
            self.circle_pending = None;
                let (center, x, y, radius) = circumcircle(a, b, c)?;
                self.commit_circle_center(center, x, y, radius)
            }
            _ => {
                self.circle_pending = Some(pending);
                Ok(())
            }
        }
    }

    fn circle_preview_ring(
        &self,
        center: Point3,
        x: Point3,
        y: Point3,
        radius: f64,
    ) -> Vec<[Point3; 2]> {
        let Some(pending) = self.circle_pending.as_ref() else {
            return circle_ring(center, x, y, radius, 64);
        };
        if !pending.deformable {
            return circle_ring(center, x, y, radius, 64);
        }
        let curve = Curve::Circle {
            center,
            x,
            y,
            radius,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        };
        let Ok(rebuilt) = om9_core::curve_build::rebuild(
            &curve,
            pending.degree,
            pending.point_count,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) else {
            return circle_ring(center, x, y, radius, 64);
        };
        let rebuilt = Curve::Nurbs(rebuilt);
        let Ok(points) = om9_core::surface_kernel::sample_curve(
            &rebuilt,
            64,
            self.coordinator.limits(),
        ) else {
            return circle_ring(center, x, y, radius, 64);
        };
        points.windows(2).map(|pair| [pair[0], pair[1]]).collect()
    }

    fn commit_circle_normal(
        &mut self,
        center: Point3,
        normal: Point3,
        radius: f64,
    ) -> Result<(), Error> {
        let norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if norm <= 1e-12 {
            return Err(Error::Invalid("circle normal"));
        }
        let normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];
        let plane = self.input_plane();
        let mut x = [
            plane.x[0] - normal[0] * (plane.x[0] * normal[0] + plane.x[1] * normal[1] + plane.x[2] * normal[2]),
            plane.x[1] - normal[1] * (plane.x[0] * normal[0] + plane.x[1] * normal[1] + plane.x[2] * normal[2]),
            plane.x[2] - normal[2] * (plane.x[0] * normal[0] + plane.x[1] * normal[1] + plane.x[2] * normal[2]),
        ];
        let mut x_norm = (x[0].powi(2) + x[1].powi(2) + x[2].powi(2)).sqrt();
        if x_norm <= 1e-12 {
            x = plane.y;
            x_norm = (x[0].powi(2) + x[1].powi(2) + x[2].powi(2)).sqrt();
        }
        x = [x[0] / x_norm, x[1] / x_norm, x[2] / x_norm];
        let mut y = [
            normal[1] * x[2] - normal[2] * x[1],
            normal[2] * x[0] - normal[0] * x[2],
            normal[0] * x[1] - normal[1] * x[0],
        ];
        let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
        if y_norm <= 1e-12 {
            return Err(Error::Invalid("circle frame"));
        }
        y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
        self.commit_circle_center(center, x, y, radius)
    }

    fn commit_circle_center(
        &mut self,
        center: Point3,
        x: Point3,
        y: Point3,
        radius: f64,
    ) -> Result<(), Error> {
        let curve = Curve::Circle {
            center,
            x,
            y,
            radius,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        };
        let rebuild = self.circle_rebuild.take().unwrap_or((false, 3, 32));
        let curve = if rebuild.0 {
            if rebuild.2 <= rebuild.1 {
                return Err(Error::Invalid("PointCount must exceed Degree"));
            }
            let rebuilt = om9_core::curve_build::rebuild(
                &curve,
                rebuild.1,
                rebuild.2,
                om9_core::curve_build::KnotMode::ChordLength,
                self.coordinator.limits(),
            )?;
            Curve::Nurbs(rebuilt)
        } else {
            curve
        };
        self.commit_geometry(CIRCLE, "Circle", Geometry::Curve(curve))
    }

    fn commit_circle_diameter(&mut self, a: Point3, b: Point3) -> Result<(), Error> {
        let diameter = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let length = (diameter[0].powi(2) + diameter[1].powi(2) + diameter[2].powi(2)).sqrt();
        if length <= 1e-9 {
            return Err(Error::Invalid("circle diameter must be positive"));
        }
        let center = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5];
        let x = [diameter[0] / length, diameter[1] / length, diameter[2] / length];
        let plane = self.input_plane();
        let normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let mut y = [
            normal[1] * x[2] - normal[2] * x[1],
            normal[2] * x[0] - normal[0] * x[2],
            normal[0] * x[1] - normal[1] * x[0],
        ];
        let norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
        if norm <= 1e-12 {
            return Err(Error::Invalid("diameter is parallel to the CPlane normal"));
        }
        y = [y[0] / norm, y[1] / norm, y[2] / norm];
        self.commit_circle_center(center, x, y, length * 0.5)
    }
}

impl Controller {
    pub fn circle_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.circle_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![
            ("Center", "Center"),
            ("2Point", "2Point"),
            ("3Point", "3Point"),
            ("Vertical", "Vertical"),
            ("FitPoints", "FitPoints"),
            ("AroundCurve", "AroundCurve"),
            ("Tangent", "Tangent"),
        ];
        tokens.push(("Deformable", "Deformable"));
        tokens.push(("Approx", "Approx"));
        if pending.deformable {
            tokens.push(("Degree=", "Degree="));
            tokens.push(("PointCount=", "PointCount="));
        }
        if pending.mode == "Tangent" {
            tokens.push(("Radius=", "Radius="));
            tokens.push(("Point=", "Point="));
            tokens.push(("Solution=", "Solution="));
        }
        if pending.points.is_empty() && pending.mode == "AroundCurve" {
            tokens.push(("OnCurve=", "OnCurve="));
        }
        if pending.points.len() == 1 && pending.mode == "Center" {
            tokens.push(("Size=Diameter", "Size=Diameter"));
            tokens.push(("Size=Circumference", "Size=Circumference"));
            tokens.push(("Size=Area", "Size=Area"));
            tokens.push(("Radius=", "Radius="));
            tokens.push(("Diameter=", "Diameter="));
            tokens.push(("Circumference=", "Circumference="));
            tokens.push(("Area=", "Area="));
        }
        if !pending.points.is_empty() {
            tokens.push(("Undo", "Undo"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }

    pub fn circle_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.circle_pending.as_ref() else {
            return Vec::new();
        };
        if pending.mode == "Tangent" {
            let Some(radius) = pending.radius else {
                return Vec::new();
            };
            let plane = self.input_plane();
            let normal = [
                plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
            ];
            let frame = if pending.tangent_curves.len() == 2 {
                tangent_line_circle(
                    &pending.tangent_curves,
                    radius,
                    pending.solution,
                    normal,
                    plane.x,
                )
            } else if pending.tangent_curves.len() == 1 {
                let Some(point) = pending.tangent_point else {
                    return Vec::new();
                };
                tangent_line_point_circle(
                    &pending.tangent_curves[0],
                    point,
                    radius,
                    pending.solution,
                    normal,
                    plane.x,
                )
            } else {
                return Vec::new();
            };
            let Ok((center, x, y)) = frame else {
                return Vec::new();
            };
            return self.circle_preview_ring(center, x, y, radius);
        }
        if pending.mode == "AroundCurve" && pending.points.len() == 1 {
            let center = pending.points[0];
            let Some(normal) = pending.normal else {
                return Vec::new();
            };
            let norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
            if norm <= 1e-12 {
                return Vec::new();
            }
            let normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];
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
                return Vec::new();
            }
            x = [x[0] / x_norm, x[1] / x_norm, x[2] / x_norm];
            let mut y = [
                normal[1] * x[2] - normal[2] * x[1],
                normal[2] * x[0] - normal[0] * x[2],
                normal[0] * x[1] - normal[1] * x[0],
            ];
            let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
            if y_norm <= 1e-12 {
                return Vec::new();
            }
            y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
            let radius = distance(center, point);
            if radius <= 1e-9 {
                return Vec::new();
            }
            return self.circle_preview_ring(center, x, y, radius);
        }
        if pending.mode == "FitPoints" {
            let mut points = pending.points.clone();
            points.push(point);
            if points.len() < 3 {
                return Vec::new();
            }
            let Ok((center, x, y, radius)) = fit_circle(&points) else {
                return Vec::new();
            };
            return self.circle_preview_ring(center, x, y, radius);
        }
        let (center, x, y, radius) = match pending.mode {
            "Center" if pending.points.len() == 1 => {
                let center = pending.points[0];
                let plane = self.input_plane();
                (center, plane.x, plane.y, distance(center, point))
            }
            "TwoPoint" if pending.points.len() == 1 => {
                let first = pending.points[0];
                let diameter = [point[0] - first[0], point[1] - first[1], point[2] - first[2]];
                let length = (diameter[0].powi(2) + diameter[1].powi(2) + diameter[2].powi(2)).sqrt();
                if length <= 1e-9 {
                    return Vec::new();
                }
                let center = [
                    (first[0] + point[0]) * 0.5,
                    (first[1] + point[1]) * 0.5,
                    (first[2] + point[2]) * 0.5,
                ];
                let x = [diameter[0] / length, diameter[1] / length, diameter[2] / length];
                let plane = self.input_plane();
                let normal = [
                    plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                    plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                    plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                ];
                let mut y = [
                    normal[1] * x[2] - normal[2] * x[1],
                    normal[2] * x[0] - normal[0] * x[2],
                    normal[0] * x[1] - normal[1] * x[0],
                ];
                let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if y_norm <= 1e-12 {
                    return Vec::new();
                }
                y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
                (center, x, y, length * 0.5)
            }
            "ThreePoint" if pending.points.len() == 2 => {
                match circumcircle(pending.points[0], pending.points[1], point) {
                    Ok((center, x, y, radius)) => (center, x, y, radius),
                    Err(_) => return Vec::new(),
                }
            }
            "Vertical" if pending.points.len() == 1 => {
                let center = pending.points[0];
                let plane = self.input_plane();
                let z = [
                    plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                    plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                    plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
                ];
                let d = [point[0] - center[0], point[1] - center[1], point[2] - center[2]];
                let axial = d[0] * z[0] + d[1] * z[1] + d[2] * z[2];
                let horizontal = [d[0] - z[0] * axial, d[1] - z[1] * axial, d[2] - z[2] * axial];
                let normal = [
                    horizontal[1] * z[2] - horizontal[2] * z[1],
                    horizontal[2] * z[0] - horizontal[0] * z[2],
                    horizontal[0] * z[1] - horizontal[1] * z[0],
                ];
                let radius = (horizontal[0].powi(2) + horizontal[1].powi(2) + horizontal[2].powi(2)).sqrt();
                if radius <= 1e-9 {
                    return Vec::new();
                }
                let norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
                if norm <= 1e-12 {
                    return Vec::new();
                }
                let normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];
                let x = [horizontal[0] / radius, horizontal[1] / radius, horizontal[2] / radius];
                let mut y = [
                    normal[1] * x[2] - normal[2] * x[1],
                    normal[2] * x[0] - normal[0] * x[2],
                    normal[0] * x[1] - normal[1] * x[0],
                ];
                let y_norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if y_norm <= 1e-12 {
                    return Vec::new();
                }
                y = [y[0] / y_norm, y[1] / y_norm, y[2] / y_norm];
                (center, x, y, radius)
            }
            _ => return Vec::new(),
        };
        if !radius.is_finite() || radius <= 1e-9 {
            return Vec::new();
        }
        let point_at = |angle: f64| {
            let (sin, cos) = angle.sin_cos();
            [
                center[0] + x[0] * radius * cos + y[0] * radius * sin,
                center[1] + x[1] * radius * cos + y[1] * radius * sin,
                center[2] + x[2] * radius * cos + y[2] * radius * sin,
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

fn circle_ring(center: Point3, x: Point3, y: Point3, radius: f64, steps: usize) -> Vec<[Point3; 2]> {
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
            let first = std::f64::consts::TAU * index as f64 / steps as f64;
            let second = std::f64::consts::TAU * (index + 1) as f64 / steps as f64;
            [point_at(first), point_at(second)]
        })
        .collect()
}

fn tangent_line_circle(
    curves: &[Curve],
    radius: f64,
    solution: usize,
    normal: Point3,
    plane_x: Point3,
) -> Result<(Point3, Point3, Point3), Error> {
    if curves.len() < 2 {
        return Err(Error::Invalid("Tangent needs two line curves"));
    }
    let (a0, a1) = match curves[0] {
        Curve::Line { start, end, .. } => (start, end),
        _ => return Err(Error::Invalid("Tangent needs two line curves")),
    };
    let (b0, b1) = match curves[1] {
        Curve::Line { start, end, .. } => (start, end),
        _ => return Err(Error::Invalid("Tangent needs two line curves")),
    };
    let d1 = unit([a1[0] - a0[0], a1[1] - a0[1], a1[2] - a0[2]]).ok_or(Error::Invalid("tangent line"))?;
    let d2 = unit([b1[0] - b0[0], b1[1] - b0[1], b1[2] - b0[2]]).ok_or(Error::Invalid("tangent line"))?;
    let n1 = unit(cross(d1, normal)).ok_or(Error::Invalid("tangent frame"))?;
    let n2 = unit(cross(d2, normal)).ok_or(Error::Invalid("tangent frame"))?;
    let denominator = dot(cross(d1, d2), normal);
    if denominator.abs() <= 1e-12 {
        return Err(Error::Invalid("tangent lines are parallel"));
    }
    let signs = [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)];
    let mut centers = Vec::new();
    for (s1, s2) in signs {
        let base = [
            b0[0] + n2[0] * radius * s2 - a0[0] - n1[0] * radius * s1,
            b0[1] + n2[1] * radius * s2 - a0[1] - n1[1] * radius * s1,
            b0[2] + n2[2] * radius * s2 - a0[2] - n1[2] * radius * s1,
        ];
        let t = dot(cross(base, d2), normal) / denominator;
        let center = [
            a0[0] + d1[0] * t + n1[0] * radius * s1,
            a0[1] + d1[1] * t + n1[1] * radius * s1,
            a0[2] + d1[2] * t + n1[2] * radius * s1,
        ];
        centers.push(center);
    }
    let center = centers[(solution - 1).min(centers.len() - 1)];
    let mut x = [plane_x[0], plane_x[1], plane_x[2]];
    let dot = dot(x, normal);
    x = [x[0] - normal[0] * dot, x[1] - normal[1] * dot, x[2] - normal[2] * dot];
    let x = unit(x).ok_or(Error::Invalid("tangent frame"))?;
    let y = unit(cross(normal, x)).ok_or(Error::Invalid("tangent frame"))?;
    Ok((center, x, y))
}

fn tangent_line_point_circle(
    curve: &Curve,
    point: Point3,
    radius: f64,
    solution: usize,
    normal: Point3,
    plane_x: Point3,
) -> Result<(Point3, Point3, Point3), Error> {
    let Curve::Line { start, end, .. } = curve else {
        return Err(Error::Invalid("Tangent needs a line"));
    };
    let d = unit([end[0] - start[0], end[1] - start[1], end[2] - start[2]])
        .ok_or(Error::Invalid("tangent line"))?;
    let n = unit(cross(d, normal)).ok_or(Error::Invalid("tangent frame"))?;
    let mut centers = Vec::new();
    for sign in [1.0, -1.0] {
        let offset = [
            start[0] + n[0] * radius * sign,
            start[1] + n[1] * radius * sign,
            start[2] + n[2] * radius * sign,
        ];
        let w = [offset[0] - point[0], offset[1] - point[1], offset[2] - point[2]];
        let along = dot(w, d);
        let disc = along * along - (dot(w, w) - radius * radius);
        if disc < 0.0 {
            continue;
        }
        let root = disc.sqrt();
        for t in [-along - root, -along + root] {
            let center = [offset[0] + d[0] * t, offset[1] + d[1] * t, offset[2] + d[2] * t];
            centers.push(center);
        }
    }
    if centers.is_empty() {
        return Err(Error::Invalid("no tangent circle for point"));
    }
    let center = centers[(solution - 1).min(centers.len() - 1)];
    let dot_value = dot(plane_x, normal);
    let x = unit([
        plane_x[0] - normal[0] * dot_value,
        plane_x[1] - normal[1] * dot_value,
        plane_x[2] - normal[2] * dot_value,
    ])
    .ok_or(Error::Invalid("tangent frame"))?;
    let y = unit(cross(normal, x)).ok_or(Error::Invalid("tangent frame"))?;
    Ok((center, x, y))
}

fn unit(vector: Point3) -> Option<Point3> {
    let length = (vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2)).sqrt();
    if length <= 1e-12 {
        None
    } else {
        Some([vector[0] / length, vector[1] / length, vector[2] / length])
    }
}

fn fit_circle(points: &[Point3]) -> Result<(Point3, Point3, Point3, f64), Error> {
    if points.len() < 3 {
        return Err(Error::Invalid("FitPoints needs at least three points"));
    }
    let count = points.len() as f64;
    let centroid = [
        points.iter().map(|point| point[0]).sum::<f64>() / count,
        points.iter().map(|point| point[1]).sum::<f64>() / count,
        points.iter().map(|point| point[2]).sum::<f64>() / count,
    ];
    let mut best = [0.0; 3];
    let mut best_len = 0.0;
    for point in points {
        let delta = [point[0] - centroid[0], point[1] - centroid[1], point[2] - centroid[2]];
        let length = delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2);
        if length > best_len {
            best_len = length;
            best = delta;
        }
    }
    let length = best_len.sqrt();
    if length <= 1e-9 {
        return Err(Error::Invalid("fit points are coincident"));
    }
    let u = [best[0] / length, best[1] / length, best[2] / length];
    let mut v = [0.0; 3];
    let mut v_len = 0.0;
    for point in points {
        let delta = [point[0] - centroid[0], point[1] - centroid[1], point[2] - centroid[2]];
        let dot = delta[0] * u[0] + delta[1] * u[1] + delta[2] * u[2];
        let residual = [
            delta[0] - u[0] * dot,
            delta[1] - u[1] * dot,
            delta[2] - u[2] * dot,
        ];
        let length = residual[0].powi(2) + residual[1].powi(2) + residual[2].powi(2);
        if length > v_len {
            v_len = length;
            v = residual;
        }
    }
    if v_len <= 1e-18 {
        return Err(Error::Invalid("fit points are collinear"));
    }
    let v_len = v_len.sqrt();
    let v = [v[0] / v_len, v[1] / v_len, v[2] / v_len];
    let normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let mut matrix = [[0.0f64; 3]; 3];
    let mut rhs = [0.0f64; 3];
    for point in points {
        let delta = [point[0] - centroid[0], point[1] - centroid[1], point[2] - centroid[2]];
        let x = delta[0] * u[0] + delta[1] * u[1] + delta[2] * u[2];
        let y = delta[0] * v[0] + delta[1] * v[1] + delta[2] * v[2];
        let row = [2.0 * x, 2.0 * y, 1.0];
        let value = x * x + y * y;
        for (i, left) in row.iter().enumerate() {
            rhs[i] += left * value;
            for (j, right) in row.iter().enumerate() {
                matrix[i][j] += left * right;
            }
        }
    }
    let solution = solve3(matrix, rhs).ok_or(Error::Invalid("fit circle is singular"))?;
    let cx = solution[0];
    let cy = solution[1];
    let radius_squared = solution[2] + cx * cx + cy * cy;
    if !radius_squared.is_finite() || radius_squared <= 0.0 {
        return Err(Error::Invalid("fit circle radius"));
    }
    let center = [
        centroid[0] + u[0] * cx + v[0] * cy,
        centroid[1] + u[1] * cx + v[1] * cy,
        centroid[2] + u[2] * cx + v[2] * cy,
    ];
    let _ = normal;
    Ok((center, u, v, radius_squared.sqrt()))
}

fn solve3(mut a: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    for column in 0..3 {
        let pivot = (column..3).max_by(|left, right| {
            a[*left][column].abs().total_cmp(&a[*right][column].abs())
        })?;
        if a[pivot][column].abs() <= 1e-18 {
            return None;
        }
        a.swap(column, pivot);
        b.swap(column, pivot);
        let divisor = a[column][column];
        for value in a[column].iter_mut().skip(column) {
            *value /= divisor;
        }
        b[column] /= divisor;
        let pivot_row = a[column];
        for row in 0..3 {
            if row == column {
                continue;
            }
            let factor = a[row][column];
            if factor == 0.0 {
                continue;
            }
            for (index, value) in a[row].iter_mut().enumerate().skip(column) {
                *value -= factor * pivot_row[index];
            }
            b[row] -= factor * b[column];
        }
    }
    Some(b)
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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

fn circumcircle(
    a: Point3,
    b: Point3,
    c: Point3,
) -> Result<(Point3, Point3, Point3, f64), Error> {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let nn = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    if nn <= 1e-20 {
        return Err(Error::Invalid("three points are collinear"));
    }
    let uu = u[0] * u[0] + u[1] * u[1] + u[2] * u[2];
    let vv = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    let vn = [
        v[1] * n[2] - v[2] * n[1],
        v[2] * n[0] - v[0] * n[2],
        v[0] * n[1] - v[1] * n[0],
    ];
    let nu = [
        n[1] * u[2] - n[2] * u[1],
        n[2] * u[0] - n[0] * u[2],
        n[0] * u[1] - n[1] * u[0],
    ];
    let center = [
        a[0] + (vn[0] * uu + nu[0] * vv) / (2.0 * nn),
        a[1] + (vn[1] * uu + nu[1] * vv) / (2.0 * nn),
        a[2] + (vn[2] * uu + nu[2] * vv) / (2.0 * nn),
    ];
    let radius = distance(center, a);
    let x = [
        (a[0] - center[0]) / radius,
        (a[1] - center[1]) / radius,
        (a[2] - center[2]) / radius,
    ];
    let normal = [n[0] / nn.sqrt(), n[1] / nn.sqrt(), n[2] / nn.sqrt()];
    let y = [
        normal[1] * x[2] - normal[2] * x[1],
        normal[2] * x[0] - normal[0] * x[2],
        normal[0] * x[1] - normal[1] * x[0],
    ];
    Ok((center, x, y, radius))
}
