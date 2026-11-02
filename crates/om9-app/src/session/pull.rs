//! Pullback (`Pull`, OM9-CURVE-015) through the core curve/surface owner.

use om9_core::geometry::{Curve, Geometry, NurbsSurface};

use super::{Controller, Error};

pub const PULL: &str = "OM9-CURVE-015";

#[derive(Debug, Clone)]
pub struct PullPending {
    pub curve: Curve,
    pub surface: NurbsSurface,
    pub tolerance: f64,
}

impl PullPending {
    pub fn prompt(&self) -> &'static str {
        "Tolerance=, or Enter to pull the curve"
    }
}

impl Controller {
    pub fn dispatch_pull(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("curve and surface selection"));
        }
        let mut curve = None;
        let mut surface = None;
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selection"))?;
            match &entity.geometry {
                Geometry::Curve(value) => curve = Some(value.clone()),
                Geometry::Surface(value) => surface = Some(value.clone()),
                _ => return Err(Error::Invalid("curve and surface selection")),
            }
        }
        let (Some(curve), Some(surface)) = (curve, surface) else {
            return Err(Error::Invalid("curve and surface selection"));
        };
        self.pull_pending = Some(PullPending {
            curve,
            surface,
            tolerance: 1e9,
        });
        Ok(())
    }

    pub fn submit_pull_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.pull_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Tolerance=") {
            let Some(tolerance) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.pull_pending = Some(pending);
                return Some(Err(Error::Invalid("tolerance must be positive")));
            };
            pending.tolerance = tolerance;
            self.pull_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.pull_pending = Some(pending);
            return Some(Err(Error::Invalid("Pull accepts Tolerance= or Enter")));
        }
        let pullback = match om9_core::curve_surface_ops::pullback_curve(
            &pending.curve,
            &pending.surface,
            64,
            pending.tolerance,
            self.coordinator.limits(),
        ) {
            Ok(pullback) => pullback,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(PULL, "Pull", Geometry::Curve(Curve::Nurbs(pullback.curve))))
    }

    pub fn pull_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.pull_pending.is_some() {
            vec![("Tolerance=", "Tolerance="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
