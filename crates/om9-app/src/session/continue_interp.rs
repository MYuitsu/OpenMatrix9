//! Continue Interpolated Curve (`ContinueinterpCrv`, OM9-CURVE-035).

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const CONTINUE_INTERP: &str = "OM9-CURVE-035";

#[derive(Debug, Clone)]
pub struct ContinueInterpPending {
    pub points: Vec<Point3>,
    pub degree: usize,
}

impl ContinueInterpPending {
    pub fn prompt(&self) -> &'static str {
        "Next point, or Enter to fit"
    }
}

impl Controller {
    pub fn dispatch_continue_interp(&mut self) -> Result<(), Error> {
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
        let samples = om9_core::surface_kernel::sample_curve(curve, 2, self.coordinator.limits())?;
        let start = *samples.last().ok_or(Error::Invalid("curve samples"))?;
        self.continue_interp_pending = Some(ContinueInterpPending {
            points: vec![start],
            degree: 3,
        });
        Ok(())
    }

    pub fn submit_continue_interp_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.continue_interp_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if option == "undo" || option == "u" {
            if pending.points.len() > 1 {
                pending.points.pop();
            }
            self.continue_interp_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value
                .parse::<usize>()
                .ok()
                .filter(|value| (1..=11).contains(value))
            else {
                self.continue_interp_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be 1..11")));
            };
            pending.degree = degree;
            self.continue_interp_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.continue_interp_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "ContinueinterpCrv accepts points, Undo or Enter",
            )));
        }
        if pending.points.len() < 4 {
            self.continue_interp_pending = Some(pending);
            return Some(Err(Error::Invalid("need three more points")));
        }
        let curve = match om9_core::curve_build::interpolate(
            &pending.points,
            pending.degree,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(
            CONTINUE_INTERP,
            "ContinueinterpCrv",
            Geometry::Curve(Curve::Nurbs(curve)),
        ))
    }

    pub fn accept_continue_interp_point(&mut self, point: Point3) -> Result<(), Error> {
        let Some(mut pending) = self.continue_interp_pending.take() else {
            return Err(Error::Missing("continue interp command"));
        };
        pending.points.push(point);
        self.continue_interp_pending = Some(pending);
        Ok(())
    }

    pub fn continue_interp_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.continue_interp_pending.is_some() {
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
