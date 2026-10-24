//! Helix command (`Helix`, OM9-CURVE-030).

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const HELIX: &str = "OM9-CURVE-030";

#[derive(Debug, Clone)]
pub struct HelixPending {
    pub center: Option<Point3>,
    pub radius: Option<f64>,
    pub height: f64,
    pub turns: f64,
    pub axis: Option<Point3>,
}

impl Default for HelixPending {
    fn default() -> Self {
        Self::new()
    }
}

impl HelixPending {
    pub fn new() -> Self {
        Self {
            center: None,
            radius: None,
            height: 10.0,
            turns: 3.0,
            axis: None,
        }
    }

    pub fn prompt(&self) -> &'static str {
        if self.center.is_none() {
            "Axis start of helix"
        } else if self.radius.is_none() {
            "Radius"
        } else {
            "Height=, Turns=, or Enter"
        }
    }
}

impl Controller {
    pub fn dispatch_helix(&mut self) -> Result<(), Error> {
        self.helix_pending = Some(HelixPending::new());
        Ok(())
    }

    pub fn submit_helix_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.helix_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.radius.take().is_none() {
                pending.center = None;
            }
            self.helix_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Radius=") {
            let Some(radius) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.helix_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            };
            pending.radius = Some(radius);
            self.helix_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Axis=") {
            let parts: Vec<f64> = value
                .split(',')
                .filter_map(|part| part.trim().parse::<f64>().ok())
                .collect();
            if parts.len() != 3 || parts.iter().any(|value| !value.is_finite()) {
                self.helix_pending = Some(pending);
                return Some(Err(Error::Invalid("Axis=x,y,z")));
            }
            let length = (parts[0].powi(2) + parts[1].powi(2) + parts[2].powi(2)).sqrt();
            if length <= 1e-12 {
                self.helix_pending = Some(pending);
                return Some(Err(Error::Invalid("axis must be non-zero")));
            }
            pending.axis = Some([parts[0] / length, parts[1] / length, parts[2] / length]);
            self.helix_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Height=") {
            let Some(height) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.helix_pending = Some(pending);
                return Some(Err(Error::Invalid("height must be positive")));
            };
            pending.height = height;
            self.helix_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Turns=") {
            let Some(turns) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.helix_pending = Some(pending);
                return Some(Err(Error::Invalid("turns must be positive")));
            };
            pending.turns = turns;
            self.helix_pending = Some(pending);
            return Some(Ok(()));
        }
        if text.trim().is_empty() && pending.radius.is_some() {
            self.helix_pending = Some(pending);
            return Some(self.commit_helix());
        }
        self.helix_pending = Some(pending);
        Some(Err(Error::Invalid("Helix needs axis start, radius, or Enter")))
    }

    pub fn accept_helix_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.helix_pending.take() else {
            return Err(Error::Missing("helix command"));
        };
        if pending.center.is_none() {
            pending.center = Some(point);
            self.helix_pending = Some(pending);
            return Ok(());
        }
        let Some(center) = pending.center else {
            self.helix_pending = Some(pending);
            return Err(Error::Invalid("helix center"));
        };
        let radius = distance(center, point);
        if radius <= 1e-9 {
            self.helix_pending = Some(pending);
            return Err(Error::Invalid("helix radius must be positive"));
        }
        pending.radius = Some(radius);
        self.helix_pending = Some(pending);
        Ok(())
    }

    fn commit_helix(&mut self) -> Result<(), Error> {
        let Some(pending) = self.helix_pending.take() else {
            return Err(Error::Missing("helix command"));
        };
        let (Some(center), Some(radius)) = (pending.center, pending.radius) else {
            return Err(Error::Invalid("helix center and radius"));
        };
        let samples = (pending.turns * 64.0).ceil().max(16.0) as usize;
        let mut points = Vec::with_capacity(samples + 1);
        for index in 0..=samples {
            let fraction = index as f64 / samples as f64;
            let angle = std::f64::consts::TAU * pending.turns * fraction;
            let plane = self.input_plane();
            let normal = pending.axis.unwrap_or([
                plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
            ]);
            let helper = if normal[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
            let u = unit(cross(helper, normal)).unwrap_or(plane.x);
            let v = unit(cross(normal, u)).unwrap_or(plane.y);
            let (sin, cos) = angle.sin_cos();
            points.push([
                center[0] + u[0] * radius * cos + v[0] * radius * sin + normal[0] * pending.height * fraction,
                center[1] + u[1] * radius * cos + v[1] * radius * sin + normal[1] * pending.height * fraction,
                center[2] + u[2] * radius * cos + v[2] * radius * sin + normal[2] * pending.height * fraction,
            ]);
        }
        let curve = om9_core::curve_build::interpolate(
            &points,
            3,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        )?;
        self.commit_geometry(HELIX, "Helix", Geometry::Curve(Curve::Nurbs(curve)))
    }


    pub fn helix_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.helix_pending.as_ref() else {
            return Vec::new();
        };
        let Some(center) = pending.center else {
            return Vec::new();
        };
        let radius = pending.radius.unwrap_or_else(|| distance(center, point));
        if radius <= 1e-9 {
            return Vec::new();
        }
        let samples = (pending.turns * 32.0).ceil().max(8.0) as usize;
        let plane = self.input_plane();
        let normal = pending.axis.unwrap_or([
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ]);
        let helper = if normal[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
        let u = unit(cross(helper, normal)).unwrap_or(plane.x);
        let v = unit(cross(normal, u)).unwrap_or(plane.y);
        let point_at = |fraction: f64| {
            let angle = std::f64::consts::TAU * pending.turns * fraction;
            let (sin, cos) = angle.sin_cos();
            [
                center[0] + u[0] * radius * cos + v[0] * radius * sin + normal[0] * pending.height * fraction,
                center[1] + u[1] * radius * cos + v[1] * radius * sin + normal[1] * pending.height * fraction,
                center[2] + u[2] * radius * cos + v[2] * radius * sin + normal[2] * pending.height * fraction,
            ]
        };
        (0..samples)
            .map(|index| {
                [
                    point_at(index as f64 / samples as f64),
                    point_at((index + 1) as f64 / samples as f64),
                ]
            })
            .collect()
    }
    pub fn helix_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.helix_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![
            ("Radius=", "Radius="),
            ("Height=", "Height="),
            ("Turns=", "Turns="),
            ("Axis=", "Axis="),
        ];
        if pending.center.is_some() {
            tokens.push(("Undo", "Undo"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(vector: Point3) -> Option<Point3> {
    let length = (vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2)).sqrt();
    if length <= 1e-12 {
        None
    } else {
        Some([vector[0] / length, vector[1] / length, vector[2] / length])
    }
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
