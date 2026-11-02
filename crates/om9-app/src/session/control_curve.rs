//! Control point curve (`Curve`, OM9-CURVE-042).

use om9_core::geometry::{Curve, Geometry, NurbsCurve, Point3};

use super::{Controller, Error};

pub const CONTROL_CURVE: &str = "OM9-CURVE-042";

#[derive(Debug, Clone, Default)]
pub struct ControlCurvePending {
    pub points: Vec<Point3>,
    pub degree: usize,
}

impl ControlCurvePending {
    pub fn prompt(&self) -> &'static str {
        "Next control point, or Enter to create"
    }
}

impl Controller {
    pub fn dispatch_control_curve(&mut self) -> Result<(), Error> {
        self.control_curve_pending = Some(ControlCurvePending {
            points: Vec::new(),
            degree: 3,
        });
        Ok(())
    }

    pub fn submit_control_curve_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.control_curve_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            pending.points.pop();
            self.control_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value
                .parse::<usize>()
                .ok()
                .filter(|value| (1..=11).contains(value))
            else {
                self.control_curve_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be 1..11")));
            };
            pending.degree = degree;
            self.control_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.control_curve_pending = Some(pending);
            return Some(Err(Error::Invalid("Curve accepts points, Undo or Enter")));
        }
        if pending.points.len() < 2 {
            self.control_curve_pending = Some(pending);
            return Some(Err(Error::Invalid("need at least two control points")));
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
        Some(self.commit_geometry(CONTROL_CURVE, "Curve", Geometry::Curve(Curve::Nurbs(curve))))
    }

    pub fn accept_control_curve_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.control_curve_pending.take() else {
            return Err(Error::Missing("curve command"));
        };
        pending.points.push(point);
        self.control_curve_pending = Some(pending);
        Ok(())
    }

    pub fn control_curve_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.control_curve_pending.is_some() {
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
