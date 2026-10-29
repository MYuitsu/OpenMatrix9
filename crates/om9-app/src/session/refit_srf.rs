// SPDX-License-Identifier: LGPL-2.1-or-later
//! RefitSrf command (`Refit to Tolerance` for surfaces, OM9-SURFACE-142).
//!
//! The Matrix/Rhino `RefitSrf` replaces a surface by the smallest regular degree/grid
//! that stays inside a tolerance. The source is read from the selection and never
//! mutated here; the refit is inserted as one transaction, exactly like `FitCrv` does
//! for curves. The measured deviation is a sampling-level bound reported by
//! `om9_core::surface_ops::refit_to_tolerance`.

use om9_core::geometry::{Geometry, NurbsSurface};

use super::{Controller, Error};

pub const REFIT: &str = "OM9-SURFACE-142";

#[derive(Debug, Clone)]
pub struct RefitSrfPending {
    pub source: NurbsSurface,
    pub tolerance: f64,
}

impl RefitSrfPending {
    pub fn prompt(&self) -> &'static str {
        "Tolerance, or Enter to refit"
    }
}

impl Controller {
    pub fn dispatch_refit_srf(&mut self) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        self.refit_srf_pending = Some(RefitSrfPending {
            source: surface.clone(),
            tolerance: 0.01,
        });
        Ok(())
    }

    pub fn submit_refit_srf_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.refit_srf_pending.take()?;
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
                self.refit_srf_pending = Some(pending);
                return Some(Err(Error::Invalid("tolerance must be positive")));
            };
            pending.tolerance = tolerance;
            self.refit_srf_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.refit_srf_pending = Some(pending);
            return Some(Err(Error::Invalid("RefitSrf accepts Tolerance= or Enter")));
        }
        let refit = match om9_core::surface_ops::refit_to_tolerance(
            &pending.source,
            pending.tolerance,
            self.coordinator.limits(),
        ) {
            Ok(refit) => refit,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(REFIT, "RefitSrf", Geometry::Surface(refit.surface)))
    }

    pub fn refit_srf_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.refit_srf_pending.is_some() {
            vec![("Tolerance=", "Tolerance="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
