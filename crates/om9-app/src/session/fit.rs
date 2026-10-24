//! FitCrv command (`Refit to Tolerance`, OM9-CURVE-010).

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const FIT: &str = "OM9-CURVE-010";

#[derive(Debug, Clone)]
pub struct FitPending {
    pub source: Curve,
    pub tolerance: f64,
}

impl FitPending {
    pub fn prompt(&self) -> &'static str {
        "Tolerance, or Enter to refit"
    }
}

impl Controller {
    pub fn dispatch_fit(&mut self) -> Result<(), Error> {
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
        self.fit_pending = Some(FitPending {
            source: curve.clone(),
            tolerance: 0.01,
        });
        Ok(())
    }

    pub fn submit_fit_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.fit_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Tolerance=") {
            let Some(tolerance) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.fit_pending = Some(pending);
                return Some(Err(Error::Invalid("tolerance must be positive")));
            };
            pending.tolerance = tolerance;
            self.fit_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.fit_pending = Some(pending);
            return Some(Err(Error::Invalid("FitCrv accepts Tolerance= or Enter")));
        }
        let samples = match om9_core::surface_kernel::sample_curve(
            &pending.source,
            64,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let mut fitted: Option<Curve> = None;
        for control_points in 4..=32 {
            let curve = match om9_core::curve_build::fit_points(
                &samples,
                3,
                control_points,
                om9_core::curve_build::KnotMode::ChordLength,
                self.coordinator.limits(),
            ) {
                Ok(curve) => curve,
                Err(_) => continue,
            };
            let curve = Curve::Nurbs(curve);
            let Ok(fitted_samples) = om9_core::surface_kernel::sample_curve(
                &curve,
                512,
                self.coordinator.limits(),
            ) else {
                continue;
            };
            let mut error = 0.0_f64;
            for target in &samples {
                let nearest = fitted_samples
                    .iter()
                    .map(|point| distance(*point, *target))
                    .fold(f64::INFINITY, f64::min);
                error = error.max(nearest);
            }
            if error <= pending.tolerance {
                fitted = Some(curve);
                break;
            }
        }
        let Some(curve) = fitted else {
            return Some(Err(Error::Invalid("FitCrv cannot meet the tolerance")));
        };
        Some(self.commit_geometry(FIT, "FitCrv", Geometry::Curve(curve)))
    }

    pub fn fit_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.fit_pending.is_some() {
            vec![("Tolerance=", "Tolerance="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
