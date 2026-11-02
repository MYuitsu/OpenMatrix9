//! Offset curve command (`Offset`, OM9-CURVE-018).

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const OFFSET: &str = "OM9-CURVE-018";

#[derive(Debug, Clone)]
pub struct OffsetPending {
    pub source: Curve,
    pub distance: f64,
}

impl OffsetPending {
    pub fn prompt(&self) -> &'static str {
        "Offset distance, or Distance="
    }
}

impl Controller {
    pub fn dispatch_offset_curve(&mut self) -> Result<(), Error> {
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
        self.offset_curve_pending = Some(OffsetPending {
            source: curve.clone(),
            distance: 1.0,
        });
        Ok(())
    }

    pub fn submit_offset_curve_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.offset_curve_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Distance=") {
            let Some(distance) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value != 0.0)
            else {
                self.offset_curve_pending = Some(pending);
                return Some(Err(Error::Invalid("offset distance must be non-zero")));
            };
            pending.distance = distance;
            self.offset_curve_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.offset_curve_pending = Some(pending);
            return Some(Err(Error::Invalid("Offset accepts Distance= or Enter")));
        }
        let samples = match om9_core::surface_kernel::sample_curve(
            &pending.source,
            64,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        if samples.len() < 3 {
            return Some(Err(Error::Invalid("offset source samples")));
        }
        let plane = self.input_plane();
        let plane_normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let mut offset = Vec::with_capacity(samples.len());
        for index in 0..samples.len() {
            let previous = samples[index.saturating_sub(1)];
            let next = samples[(index + 1).min(samples.len() - 1)];
            let tangent = [
                next[0] - previous[0],
                next[1] - previous[1],
                next[2] - previous[2],
            ];
            let mut normal = [
                tangent[1] * plane_normal[2] - tangent[2] * plane_normal[1],
                tangent[2] * plane_normal[0] - tangent[0] * plane_normal[2],
                tangent[0] * plane_normal[1] - tangent[1] * plane_normal[0],
            ];
            let norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
            if norm <= 1e-12 {
                normal = plane.y;
            } else {
                normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];
            }
            offset.push([
                samples[index][0] + normal[0] * pending.distance,
                samples[index][1] + normal[1] * pending.distance,
                samples[index][2] + normal[2] * pending.distance,
            ]);
        }
        let curve = match om9_core::curve_build::interpolate(
            &offset,
            3,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(OFFSET, "Offset", Geometry::Curve(Curve::Nurbs(curve))))
    }

    pub fn offset_curve_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.offset_curve_pending.is_some() {
            vec![("Distance=", "Distance="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
