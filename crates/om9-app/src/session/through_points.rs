//! Curve Through Points (`CurveThroughPt`, OM9-CURVE-026).

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const THROUGH_POINTS: &str = "OM9-CURVE-026";

#[derive(Debug, Clone)]
pub struct ThroughPointsPending {
    pub points: Vec<Point3>,
    pub degree: usize,
    pub closed: bool,
}

impl ThroughPointsPending {
    pub fn prompt(&self) -> &'static str {
        "Enter to interpolate the selected points"
    }
}

impl Controller {
    pub fn dispatch_through_points(&mut self) -> Result<(), Error> {
        let points: Vec<Point3> = self
            .selected_ordered()
            .into_iter()
            .filter_map(|id| {
                let entity = self.document().entities.get(&id)?;
                if let Geometry::Point(point) = entity.geometry {
                    Some(point)
                } else {
                    None
                }
            })
            .collect();
        if points.len() < 4 {
            return Err(Error::Invalid("select at least four points"));
        }
        self.through_points_pending = Some(ThroughPointsPending {
            points,
            degree: 3,
            closed: false,
        });
        Ok(())
    }

    pub fn submit_through_points_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.through_points_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value.parse::<usize>().ok().filter(|value| (1..=11).contains(value)) else {
                self.through_points_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be 1..11")));
            };
            pending.degree = degree;
            self.through_points_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "closed" || option == "close" || option == "c" {
            pending.closed = true;
            self.through_points_pending = Some(pending);
            return Some(Ok(()));
        }
        if option == "open" || option == "o" {
            pending.closed = false;
            self.through_points_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.through_points_pending = Some(pending);
            return Some(Err(Error::Invalid("CurveThroughPt accepts Enter or Cancel")));
        }
        let mut points = pending.points;
        if pending.closed && points.len() >= 4 {
            points.push(points[0]);
        }
        if points.len() <= pending.degree {
            return Some(Err(Error::Invalid("CurveThroughPt needs more points than degree")));
        }
        let curve = match om9_core::curve_build::interpolate(
            &points,
            pending.degree,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(
            THROUGH_POINTS,
            "CurveThroughPt",
            Geometry::Curve(Curve::Nurbs(curve)),
        ))
    }

    pub fn through_points_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.through_points_pending.as_ref() else {
            return Vec::new();
        };
        let mut tokens = vec![("Degree=", "Degree=")];
        if pending.points.len() >= 4 {
            tokens.push(("Closed", "Closed"));
            tokens.push(("Open", "Open"));
        }
        tokens.push(("Cancel", "Cancel"));
        tokens
    }
}
