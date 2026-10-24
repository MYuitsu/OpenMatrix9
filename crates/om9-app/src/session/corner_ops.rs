//! Curve Fillet/Chamfer (`Fillet`, `Chamfer`, OM9-CURVE-021/022) for straight lines.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const FILLET: &str = "OM9-CURVE-021";
pub const CHAMFER: &str = "OM9-CURVE-022";

#[derive(Debug, Clone)]
pub struct CornerPending {
    pub feature: &'static str,
    pub first: Curve,
    pub second: Curve,
    pub radius: f64,
    pub distance: f64,
}

impl CornerPending {
    pub fn prompt(&self) -> &'static str {
        if self.feature == FILLET {
            "Radius=, or Enter for radius 1"
        } else {
            "Distance=, or Enter for distance 1"
        }
    }
}

impl Controller {
    pub fn dispatch_corner(&mut self, feature: &'static str) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two line selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        self.corner_pending = Some(CornerPending {
            feature,
            first: curves[0].clone(),
            second: curves[1].clone(),
            radius: 1.0,
            distance: 1.0,
        });
        Ok(())
    }

    pub fn submit_corner_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.corner_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Radius=") {
            let Some(radius) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.corner_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            };
            pending.radius = radius;
            self.corner_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Distance=") {
            let Some(distance) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.corner_pending = Some(pending);
                return Some(Err(Error::Invalid("distance must be positive")));
            };
            pending.distance = distance;
            self.corner_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.corner_pending = Some(pending);
            return Some(Err(Error::Invalid("Fillet/Chamfer accept a number or Enter")));
        }
        let plane = self.input_plane();
        let normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        if pending.feature == FILLET {
            let fillet = match om9_core::curve_modify::fillet_lines(
                &pending.first,
                &pending.second,
                pending.radius,
                normal,
                1e-7,
                self.coordinator.limits(),
            ) {
                Ok(fillet) => fillet,
                Err(error) => return Some(Err(error)),
            };
            Some(self.commit_geometry(FILLET, "Fillet", Geometry::Curve(fillet.arc)))
        } else {
            let chamfer = match om9_core::curve_modify::chamfer_lines(
                &pending.first,
                &pending.second,
                pending.distance,
                pending.distance,
                1e-7,
                self.coordinator.limits(),
            ) {
                Ok(chamfer) => chamfer,
                Err(error) => return Some(Err(error)),
            };
            Some(self.commit_geometry(CHAMFER, "Chamfer", Geometry::Curve(chamfer.line)))
        }
    }

    pub fn corner_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.corner_pending.as_ref() else {
            return Vec::new();
        };
        if pending.feature == FILLET {
            vec![("Radius=", "Radius="), ("Cancel", "Cancel")]
        } else {
            vec![("Distance=", "Distance="), ("Cancel", "Cancel")]
        }
    }
}
