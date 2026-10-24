//! Curve intersection (`Intersect`, OM9-CURVE-017).

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const INTERSECT: &str = "OM9-CURVE-017";

#[derive(Debug, Clone)]
pub struct IntersectPending {
    pub first: Curve,
    pub second: Curve,
    pub tolerance: f64,
}

impl IntersectPending {
    pub fn prompt(&self) -> &'static str {
        "Tolerance=, or Enter to intersect"
    }
}

impl Controller {
    pub fn dispatch_intersect(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.intersect_pending = Some(IntersectPending {
            first: curves[0].clone(),
            second: curves[1].clone(),
            tolerance: 1e-6,
        });
        Ok(())
    }

    pub fn submit_intersect_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.intersect_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Tolerance=") {
            let Some(tolerance) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.intersect_pending = Some(pending);
                return Some(Err(Error::Invalid("tolerance must be positive")));
            };
            pending.tolerance = tolerance;
            self.intersect_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.intersect_pending = Some(pending);
            return Some(Err(Error::Invalid("Intersect accepts Tolerance= or Enter")));
        }
        let intersections = match om9_core::curve_surface_ops::curve_curve_intersections(
            &pending.first,
            &pending.second,
            pending.tolerance,
            self.coordinator.limits(),
        ) {
            Ok(intersections) => intersections,
            Err(error) => return Some(Err(error)),
        };
        let mut points: Vec<Point3> = Vec::new();
        for intersection in intersections {
            let point = intersection.point;
            if !points.iter().any(|other| {
                (other[0] - point[0]).abs() < pending.tolerance
                    && (other[1] - point[1]).abs() < pending.tolerance
                    && (other[2] - point[2]).abs() < pending.tolerance
            }) {
                points.push(point);
            }
        }
        if points.is_empty() {
            return Some(Err(Error::Invalid("curves do not intersect")));
        }
        for point in points {
            if let Err(error) = self.commit_geometry(
                INTERSECT,
                "Intersection",
                Geometry::Point(point),
            ) {
                return Some(Err(error));
            }
        }
        Some(Ok(()))
    }

    pub fn intersect_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.intersect_pending.is_some() {
            vec![("Tolerance=", "Tolerance="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
