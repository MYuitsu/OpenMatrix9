//! Adjust Closed Curve Seam (`CrvSeam`, OM9-CURVE-031).

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const CRV_SEAM: &str = "OM9-CURVE-031";

#[derive(Debug, Clone)]
pub struct CrvSeamPending {
    pub curve: Curve,
    pub fraction: f64,
}

impl CrvSeamPending {
    pub fn prompt(&self) -> &'static str {
        "Seam=0..1, or Enter to move the seam"
    }
}

impl Controller {
    pub fn dispatch_crv_seam(&mut self) -> Result<(), Error> {
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
        self.crv_seam_pending = Some(CrvSeamPending {
            curve: curve.clone(),
            fraction: 0.5,
        });
        Ok(())
    }

    pub fn submit_crv_seam_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.crv_seam_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Seam=") {
            let Some(fraction) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
            else {
                self.crv_seam_pending = Some(pending);
                return Some(Err(Error::Invalid("seam fraction must be 0..1")));
            };
            pending.fraction = fraction;
            self.crv_seam_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.crv_seam_pending = Some(pending);
            return Some(Err(Error::Invalid("CrvSeam accepts Seam= or Enter")));
        }
        let curve = match om9_core::curve_ops::set_seam(
            &pending.curve,
            pending.fraction,
            1e-6,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(CRV_SEAM, "CrvSeam", Geometry::Curve(curve)))
    }

    pub fn crv_seam_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.crv_seam_pending.is_some() {
            vec![("Seam=", "Seam="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
