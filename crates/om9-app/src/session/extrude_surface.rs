//! Extrude (`Extrude`, OM9-SURFACE-002): extrude selected curves upward.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const EXTRUDE: &str = "OM9-SURFACE-002";

#[derive(Debug, Clone)]
pub struct ExtrudePending {
    pub curves: Vec<Curve>,
    pub height: f64,
}

impl ExtrudePending {
    pub fn prompt(&self) -> &'static str {
        "Height=, or Enter to extrude by 10"
    }
}

impl Controller {
    pub fn dispatch_extrude_surface(&mut self) -> Result<(), Error> {
        let mut curves = Vec::new();
        for id in self.selected_ordered() {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selection"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        if curves.is_empty() {
            return Err(Error::Missing("curve selection"));
        }
        self.extrude_surface_pending = Some(ExtrudePending {
            curves,
            height: 10.0,
        });
        Ok(())
    }

    pub fn submit_extrude_surface_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.extrude_surface_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Height=") {
            let Some(height) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value != 0.0)
            else {
                self.extrude_surface_pending = Some(pending);
                return Some(Err(Error::Invalid("height must be non-zero")));
            };
            pending.height = height;
            self.extrude_surface_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.extrude_surface_pending = Some(pending);
            return Some(Err(Error::Invalid("Extrude accepts Height= or Enter")));
        }
        let surfaces = match om9_core::surface_ops::extrude_curves(
            &pending.curves,
            pending.height,
            self.coordinator.limits(),
        ) {
            Ok(surfaces) => surfaces,
            Err(error) => return Some(Err(error)),
        };
        for surface in surfaces {
            if let Err(error) = self.commit_geometry(EXTRUDE, "Extrude", Geometry::Surface(surface))
            {
                return Some(Err(error));
            }
        }
        Some(Ok(()))
    }

    pub fn extrude_surface_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.extrude_surface_pending.is_some() {
            vec![("Height=", "Height="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
