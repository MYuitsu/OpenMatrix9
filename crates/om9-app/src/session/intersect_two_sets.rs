//! Intersect Two Sets (`IntersectTwoSets`, OM9-CURVE-058).

use om9_core::geometry::Geometry;

use super::{Controller, Error};

pub const INTERSECT_TWO_SETS: &str = "OM9-CURVE-058";

impl Controller {
    pub fn dispatch_intersect_two_sets(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() < 4 {
            return Err(Error::Invalid("two sets of at least two curves"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
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
        let split = curves.len() / 2;
        let (first, second) = curves.split_at(split);
        let mut points = Vec::new();
        for a in first {
            for b in second {
                let intersections = om9_core::curve_surface_ops::curve_curve_intersections(
                    a,
                    b,
                    1e-6,
                    self.coordinator.limits(),
                )?;
                for intersection in intersections {
                    if !points.iter().any(|point: &om9_core::geometry::Point3| {
                        (point[0] - intersection.point[0]).abs() < 1e-6
                            && (point[1] - intersection.point[1]).abs() < 1e-6
                            && (point[2] - intersection.point[2]).abs() < 1e-6
                    }) {
                        points.push(intersection.point);
                    }
                }
            }
        }
        if points.is_empty() {
            return Err(Error::Invalid("sets do not intersect"));
        }
        for point in points {
            self.commit_geometry(
                INTERSECT_TWO_SETS,
                "IntersectTwoSets",
                Geometry::Point(point),
            )?;
        }
        Ok(())
    }
}
