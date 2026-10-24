//! Continue Control Point Curve (`ContinueCurve`, OM9-CURVE-041).

use om9_core::geometry::{Curve, Geometry, NurbsCurve, Point3};

use super::{Controller, Error};

pub const CONTINUE_CURVE: &str = "OM9-CURVE-041";

#[derive(Debug, Clone)]
pub struct ContinueCurvePending {
    pub points: Vec<Point3>,
    pub degree: usize,
}

impl ContinueCurvePending {
    pub fn prompt(&self) -> &'static str {
        "Next control point, or Enter to create"
    }
}

impl Controller {
    pub fn dispatch_continue_curve(&mut self) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("curve selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("curve selection"));
        };
        let seed = match curve {
            Curve::Nurbs(nurbs) => *nurbs
                .poles
                .last()
                .ok_or(Error::Invalid("curve has no poles"))?,
            other => *om9_core::surface_kernel::sample_curve(other, 2, self.coordinator.limits())?
                .last()
                .ok_or(Error::Invalid("curve has no samples"))?,
        };
        self.continue_curve_pending = Some(ContinueCurvePending {
            points: vec![seed],
            degree: 3,
        });
        Ok(())
    }

    pub fn submit_continue_curve_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.continue_curve_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.points.len() > 1 {
                pending.points.pop();
            }
            self.continue_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value.parse::<usize>().ok().filter(|value| (1..=11).contains(value)) else {
                self.continue_curve_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be 1..11")));
            };
            pending.degree = degree;
            self.continue_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.continue_curve_pending = Some(pending);
            return Some(Err(Error::Invalid("ContinueCurve accepts points, Undo or Enter")));
        }
        if pending.points.len() < 2 {
            self.continue_curve_pending = Some(pending);
            return Some(Err(Error::Invalid("need at least one new control point")));
        }
        let count = pending.points.len();
        let degree = pending.degree.min(count - 1);
        let mut knots = vec![0.0; degree + 1];
        let spans = count - degree;
        for span in 1..spans {
            knots.push(span as f64 / spans as f64);
        }
        knots.resize(count + degree + 1, 1.0);
        let curve = NurbsCurve {
            degree,
            poles: pending.points,
            weights: vec![1.0; count],
            knots,
            domain: [0.0, 1.0],
            periodic: false,
            reversed: false,
            seam: None,
        };
        Some(self.commit_geometry(
            CONTINUE_CURVE,
            "ContinueCurve",
            Geometry::Curve(Curve::Nurbs(curve)),
        ))
    }

    pub fn accept_continue_curve_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.continue_curve_pending.take() else {
            return Err(Error::Missing("continue curve command"));
        };
        pending.points.push(point);
        self.continue_curve_pending = Some(pending);
        Ok(())
    }

    pub fn continue_curve_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.continue_curve_pending.is_some() {
            vec![
                ("Degree=", "Degree="),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ]
        } else {
            Vec::new()
        }
    }
}
