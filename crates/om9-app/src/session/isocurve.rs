//! Extract Isocurve (`ExtractIsocurve`, OM9-CURVE-019).

use om9_core::geometry::{Curve, Geometry};
use om9_core::surface_kernel::SurfaceIso;

use super::{Controller, Error};

pub const ISOCURVE: &str = "OM9-CURVE-019";

#[derive(Debug, Clone)]
pub struct IsoPending {
    pub surface: om9_core::geometry::NurbsSurface,
    pub iso: SurfaceIso,
    pub value: f64,
}

impl IsoPending {
    pub fn prompt(&self) -> &'static str {
        "U= or V= parameter, or Enter to extract"
    }
}

impl Controller {
    pub fn dispatch_isocurve(&mut self) -> Result<(), Error> {
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
        self.isocurve_pending = Some(IsoPending {
            surface: surface.clone(),
            iso: SurfaceIso::U,
            value: 0.5,
        });
        Ok(())
    }

    pub fn submit_isocurve_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.isocurve_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("U=") {
            let Some(value) = value.parse::<f64>().ok().filter(|value| value.is_finite()) else {
                self.isocurve_pending = Some(pending);
                return Some(Err(Error::Invalid("isocurve parameter")));
            };
            pending.iso = SurfaceIso::U;
            pending.value = value;
            self.isocurve_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("V=") {
            let Some(value) = value.parse::<f64>().ok().filter(|value| value.is_finite()) else {
                self.isocurve_pending = Some(pending);
                return Some(Err(Error::Invalid("isocurve parameter")));
            };
            pending.iso = SurfaceIso::V;
            pending.value = value;
            self.isocurve_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.isocurve_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "ExtractIsocurve accepts U=, V= or Enter",
            )));
        }
        let curve = match om9_core::surface_kernel::isocurve(
            &pending.surface,
            pending.iso,
            pending.value,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(
            ISOCURVE,
            "ExtractIsocurve",
            Geometry::Curve(Curve::Nurbs(curve)),
        ))
    }

    pub fn isocurve_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.isocurve_pending.is_some() {
            vec![("U=", "U="), ("V=", "V="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
