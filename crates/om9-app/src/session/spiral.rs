//! Spiral command (`Spiral`, OM9-CURVE-029).

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const SPIRAL: &str = "OM9-CURVE-029";

#[derive(Debug, Clone)]
pub struct SpiralPending {
    pub center: Option<Point3>,
    pub start_radius: Option<f64>,
    pub end_radius: Option<f64>,
    pub turns: f64,
    pub height: f64,
    pub axis: Option<Point3>,
}

impl Default for SpiralPending {
    fn default() -> Self {
        Self::new()
    }
}

impl SpiralPending {
    pub fn new() -> Self {
        Self {
            center: None,
            start_radius: None,
            end_radius: None,
            turns: 3.0,
            height: 0.0,
            axis: None,
        }
    }

    pub fn prompt(&self) -> &'static str {
        if self.center.is_none() {
            "Center of spiral"
        } else if self.start_radius.is_none() {
            "Start radius"
        } else {
            "EndRadius=, Turns=, Height=, or Enter"
        }
    }
}

impl Controller {
    pub fn dispatch_spiral(&mut self) -> Result<(), Error> {
        self.spiral_pending = Some(SpiralPending::new());
        Ok(())
    }

    pub fn submit_spiral_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.spiral_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.start_radius.take().is_none() {
                pending.center = None;
            }
            self.spiral_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Axis=") {
            let parts: Vec<f64> = value
                .split(',')
                .filter_map(|part| part.trim().parse::<f64>().ok())
                .collect();
            if parts.len() != 3 || parts.iter().any(|value| !value.is_finite()) {
                self.spiral_pending = Some(pending);
                return Some(Err(Error::Invalid("Axis=x,y,z")));
            }
            let length = (parts[0].powi(2) + parts[1].powi(2) + parts[2].powi(2)).sqrt();
            if length <= 1e-12 {
                self.spiral_pending = Some(pending);
                return Some(Err(Error::Invalid("axis must be non-zero")));
            }
            pending.axis = Some([parts[0] / length, parts[1] / length, parts[2] / length]);
            self.spiral_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("StartRadius=") {
            let Some(radius) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.spiral_pending = Some(pending);
                return Some(Err(Error::Invalid("start radius must be positive")));
            };
            pending.start_radius = Some(radius);
            self.spiral_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("EndRadius=") {
            let Some(radius) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.spiral_pending = Some(pending);
                return Some(Err(Error::Invalid("end radius must be positive")));
            };
            pending.end_radius = Some(radius);
            self.spiral_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Turns=") {
            let Some(turns) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.spiral_pending = Some(pending);
                return Some(Err(Error::Invalid("turns must be positive")));
            };
            pending.turns = turns;
            self.spiral_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Height=") {
            let Some(height) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value >= 0.0)
            else {
                self.spiral_pending = Some(pending);
                return Some(Err(Error::Invalid("height must be non-negative")));
            };
            pending.height = height;
            self.spiral_pending = Some(pending);
            return Some(Ok(()));
        }
        if text.trim().is_empty() && pending.start_radius.is_some() {
            self.spiral_pending = Some(pending);
            return Some(self.commit_spiral());
        }
        self.spiral_pending = Some(pending);
        Some(Err(Error::Invalid("Spiral needs center, radius or Enter")))
    }

    pub fn accept_spiral_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.spiral_pending.take() else {
            return Err(Error::Missing("spiral command"));
        };
        if pending.center.is_none() {
            pending.center = Some(point);
            self.spiral_pending = Some(pending);
            return Ok(());
        }
        let Some(center) = pending.center else {
            self.spiral_pending = Some(pending);
            return Err(Error::Invalid("spiral center"));
        };
        let radius = distance(center, point);
        if radius <= 1e-9 {
            self.spiral_pending = Some(pending);
            return Err(Error::Invalid("spiral radius must be positive"));
        }
        pending.start_radius = Some(radius);
        self.spiral_pending = Some(pending);
        Ok(())
    }

    fn commit_spiral(&mut self) -> Result<(), Error> {
        let Some(pending) = self.spiral_pending.take() else {
            return Err(Error::Missing("spiral command"));
        };
        let (Some(center), Some(start)) = (pending.center, pending.start_radius) else {
            return Err(Error::Invalid("spiral center and radius"));
        };
        let end = pending.end_radius.unwrap_or(start);
        let samples = (pending.turns * 64.0).ceil().max(16.0) as usize;
        let mut points = Vec::with_capacity(samples + 1);
        for index in 0..=samples {
            let fraction = index as f64 / samples as f64;
            let angle = std::f64::consts::TAU * pending.turns * fraction;
            let radius = start + (end - start) * fraction;
            let plane = self.input_plane();
            let normal = pending.axis.unwrap_or([
                plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
                plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
                plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
            ]);
            let helper = if normal[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let u = unit(cross(helper, normal)).unwrap_or(plane.x);
            let v = unit(cross(normal, u)).unwrap_or(plane.y);
            let (sin, cos) = angle.sin_cos();
            points.push([
                center[0]
                    + u[0] * radius * cos
                    + v[0] * radius * sin
                    + normal[0] * pending.height * fraction,
                center[1]
                    + u[1] * radius * cos
                    + v[1] * radius * sin
                    + normal[1] * pending.height * fraction,
                center[2]
                    + u[2] * radius * cos
                    + v[2] * radius * sin
                    + normal[2] * pending.height * fraction,
            ]);
        }
        let curve = om9_core::curve_build::interpolate(
            &points,
            3,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        )?;
        self.commit_geometry(SPIRAL, "Spiral", Geometry::Curve(Curve::Nurbs(curve)))
    }

    pub fn spiral_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.spiral_pending.as_ref() else {
            return Vec::new();
        };
        let Some(center) = pending.center else {
            return Vec::new();
        };
        let start = pending
            .start_radius
            .unwrap_or_else(|| distance(center, point));
        if start <= 1e-9 {
            return Vec::new();
        }
        let end = pending.end_radius.unwrap_or(start);
        let samples = (pending.turns * 32.0).ceil().max(8.0) as usize;
        let plane = self.input_plane();
        let normal = pending.axis.unwrap_or([
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ]);
        let helper = if normal[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = unit(cross(helper, normal)).unwrap_or(plane.x);
        let v = unit(cross(normal, u)).unwrap_or(plane.y);
        let point_at = |fraction: f64| {
            let angle = std::f64::consts::TAU * pending.turns * fraction;
            let radius = start + (end - start) * fraction;
            let (sin, cos) = angle.sin_cos();
            [
                center[0]
                    + u[0] * radius * cos
                    + v[0] * radius * sin
                    + normal[0] * pending.height * fraction,
                center[1]
                    + u[1] * radius * cos
                    + v[1] * radius * sin
                    + normal[1] * pending.height * fraction,
                center[2]
                    + u[2] * radius * cos
                    + v[2] * radius * sin
                    + normal[2] * pending.height * fraction,
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
    pub fn spiral_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.spiral_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![
            ("StartRadius=", "StartRadius="),
            ("EndRadius=", "EndRadius="),
            ("Turns=", "Turns="),
            ("Height=", "Height="),
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
