//! Polygon command (`Polygon`, OM9-CURVE-028).

use om9_core::geometry::{Geometry, Point3};

use super::{Controller, Error};

pub const POLYGON: &str = "OM9-CURVE-028";

#[derive(Debug, Clone)]
pub struct PolygonPending {
    pub center: Option<Point3>,
    pub radius: Option<f64>,
    pub sides: usize,
    pub circumscribed: bool,
    pub start_angle: f64,
}

impl Default for PolygonPending {
    fn default() -> Self {
        Self::new()
    }
}

impl PolygonPending {
    pub fn new() -> Self {
        Self {
            center: None,
            radius: None,
            sides: 4,
            circumscribed: false,
            start_angle: 0.0,
        }
    }

    pub fn prompt(&self) -> &'static str {
        if self.center.is_none() {
            "Center of polygon"
        } else if self.radius.is_none() {
            "Radius"
        } else {
            "Sides=, Circumscribed, or Enter"
        }
    }
}

impl Controller {
    pub fn dispatch_polygon(&mut self) -> Result<(), Error> {
        self.polygon_pending = Some(PolygonPending::new());
        Ok(())
    }

    pub fn submit_polygon_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.polygon_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.radius.take().is_none() {
                pending.center = None;
            }
            self.polygon_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "circumscribed" || option == "c" {
            pending.circumscribed = true;
            self.polygon_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "inscribed" || option == "i" {
            pending.circumscribed = false;
            self.polygon_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Sides=") {
            let Some(sides) = value
                .parse::<usize>()
                .ok()
                .filter(|sides| (3..=1024).contains(sides))
            else {
                self.polygon_pending = Some(pending);
                return Some(Err(Error::Invalid("sides must be 3..1024")));
            };
            pending.sides = sides;
            self.polygon_pending = Some(pending);
            return Some(Ok(()));
        }
        if text.trim().is_empty() && pending.radius.is_some() {
            self.polygon_pending = Some(pending);
            return Some(self.commit_polygon());
        }
        self.polygon_pending = Some(pending);
        Some(Err(Error::Invalid(
            "Polygon needs center, radius, Sides= or Enter",
        )))
    }

    pub fn accept_polygon_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.polygon_pending.take() else {
            return Err(Error::Missing("polygon command"));
        };
        if pending.center.is_none() {
            pending.center = Some(point);
            self.polygon_pending = Some(pending);
            return Ok(());
        }
        let Some(center) = pending.center else {
            self.polygon_pending = Some(pending);
            return Err(Error::Invalid("polygon center"));
        };
        let radius = distance(center, point);
        if radius <= 1e-9 {
            self.polygon_pending = Some(pending);
            return Err(Error::Invalid("polygon radius must be positive"));
        }
        pending.radius = Some(radius);
        let plane = self.input_plane();
        let delta = [
            point[0] - center[0],
            point[1] - center[1],
            point[2] - center[2],
        ];
        let x = delta[0] * plane.x[0] + delta[1] * plane.x[1] + delta[2] * plane.x[2];
        let y = delta[0] * plane.y[0] + delta[1] * plane.y[1] + delta[2] * plane.y[2];
        pending.start_angle = y.atan2(x);
        self.polygon_pending = Some(pending);
        Ok(())
    }

    fn commit_polygon(&mut self) -> Result<(), Error> {
        let Some(pending) = self.polygon_pending.take() else {
            return Err(Error::Missing("polygon command"));
        };
        let Some(center) = pending.center else {
            return Err(Error::Invalid("polygon center"));
        };
        let Some(radius) = pending.radius else {
            return Err(Error::Invalid("polygon radius"));
        };
        let corner = if pending.circumscribed {
            radius / (std::f64::consts::PI / pending.sides as f64).cos()
        } else {
            radius
        };
        let plane = self.input_plane();
        let mut points = Vec::with_capacity(pending.sides);
        for index in 0..pending.sides {
            let angle =
                pending.start_angle + std::f64::consts::TAU * index as f64 / pending.sides as f64;
            let (sin, cos) = angle.sin_cos();
            points.push([
                center[0] + plane.x[0] * corner * cos + plane.y[0] * corner * sin,
                center[1] + plane.x[1] * corner * cos + plane.y[1] * corner * sin,
                center[2] + plane.x[2] * corner * cos + plane.y[2] * corner * sin,
            ]);
        }
        let curve = om9_core::curve_build::polygon(&points)?;
        self.commit_geometry(POLYGON, "Polygon", Geometry::Curve(curve))
    }

    pub fn polygon_preview(&self, point: Point3) -> Vec<[Point3; 2]> {
        let Some(pending) = self.polygon_pending.as_ref() else {
            return Vec::new();
        };
        let Some(center) = pending.center else {
            return Vec::new();
        };
        let radius = pending.radius.unwrap_or_else(|| distance(center, point));
        let start_angle = if pending.radius.is_some() {
            pending.start_angle
        } else {
            let plane = self.input_plane();
            let delta = [
                point[0] - center[0],
                point[1] - center[1],
                point[2] - center[2],
            ];
            let x = delta[0] * plane.x[0] + delta[1] * plane.x[1] + delta[2] * plane.x[2];
            let y = delta[0] * plane.y[0] + delta[1] * plane.y[1] + delta[2] * plane.y[2];
            y.atan2(x)
        };
        if radius <= 1e-9 {
            return Vec::new();
        }
        let corner = if pending.circumscribed {
            radius / (std::f64::consts::PI / pending.sides as f64).cos()
        } else {
            radius
        };
        let plane = self.input_plane();
        let mut points = Vec::with_capacity(pending.sides);
        for index in 0..pending.sides {
            let angle = start_angle + std::f64::consts::TAU * index as f64 / pending.sides as f64;
            let (sin, cos) = angle.sin_cos();
            points.push([
                center[0] + plane.x[0] * corner * cos + plane.y[0] * corner * sin,
                center[1] + plane.x[1] * corner * cos + plane.y[1] * corner * sin,
                center[2] + plane.x[2] * corner * cos + plane.y[2] * corner * sin,
            ]);
        }
        (0..points.len())
            .map(|index| [points[index], points[(index + 1) % points.len()]])
            .collect()
    }

    pub fn polygon_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.polygon_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![
            ("Sides=", "Sides="),
            ("Circumscribed", "Circumscribed"),
            ("Inscribed", "Inscribed"),
        ];
        if pending.center.is_some() {
            tokens.insert(0, ("Undo", "Undo"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
