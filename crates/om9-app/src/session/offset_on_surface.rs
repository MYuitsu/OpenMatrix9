//! Offset Curve on Surface (`OffsetCrvOnSrf`, OM9-CURVE-020).

use om9_core::geometry::{Curve, Geometry, NurbsSurface};

use super::{Controller, Error};

pub const OFFSET_ON_SURFACE: &str = "OM9-CURVE-020";

#[derive(Debug, Clone)]
pub struct OffsetOnSurfacePending {
    pub curve: Curve,
    pub surface: NurbsSurface,
    pub distance: f64,
}

impl OffsetOnSurfacePending {
    pub fn prompt(&self) -> &'static str {
        "Distance=, or Enter to offset by 1"
    }
}

impl Controller {
    pub fn dispatch_offset_on_surface(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
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
        self.offset_on_surface_pending = Some(OffsetOnSurfacePending {
            curve,
            surface,
            distance: 1.0,
        });
        Ok(())
    }

    pub fn submit_offset_on_surface_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.offset_on_surface_pending.take()?;
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
                self.offset_on_surface_pending = Some(pending);
                return Some(Err(Error::Invalid("distance must be non-zero")));
            };
            pending.distance = distance;
            self.offset_on_surface_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.offset_on_surface_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "OffsetCrvOnSrf accepts Distance= or Enter",
            )));
        }
        let samples = match om9_core::surface_kernel::sample_curve(
            &pending.curve,
            48,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let mut parameters = Vec::with_capacity(samples.len());
        for index in 0..samples.len() {
            let projection = match om9_core::curve_surface_ops::closest_point_on_surface(
                &pending.surface,
                samples[index],
                self.coordinator.limits(),
            ) {
                Ok(projection) => projection,
                Err(error) => return Some(Err(error)),
            };
            let normal = match om9_core::surface_kernel::normal(
                &pending.surface,
                projection.parameters[0],
                projection.parameters[1],
                self.coordinator.limits(),
            ) {
                Ok(normal) => normal,
                Err(error) => return Some(Err(error)),
            };
            let previous = projection.point;
            let next = if index + 1 < samples.len() {
                om9_core::curve_surface_ops::closest_point_on_surface(
                    &pending.surface,
                    samples[index + 1],
                    self.coordinator.limits(),
                )
                .map(|projection| projection.point)
                .unwrap_or(previous)
            } else {
                previous
            };
            let tangent = [
                next[0] - previous[0],
                next[1] - previous[1],
                next[2] - previous[2],
            ];
            let side = [
                tangent[1] * normal[2] - tangent[2] * normal[1],
                tangent[2] * normal[0] - tangent[0] * normal[2],
                tangent[0] * normal[1] - tangent[1] * normal[0],
            ];
            let length = (side[0].powi(2) + side[1].powi(2) + side[2].powi(2)).sqrt();
            if length <= 1e-12 {
                parameters.push(projection.parameters);
                continue;
            }
            let target = [
                projection.point[0] + side[0] / length * pending.distance,
                projection.point[1] + side[1] / length * pending.distance,
                projection.point[2] + side[2] / length * pending.distance,
            ];
            let shifted = match om9_core::curve_surface_ops::closest_point_on_surface(
                &pending.surface,
                target,
                self.coordinator.limits(),
            ) {
                Ok(projection) => projection,
                Err(error) => return Some(Err(error)),
            };
            parameters.push(shifted.parameters);
        }
        let curve = match om9_core::curve_surface_ops::sketch_on_surface(
            &pending.surface,
            &parameters,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(OFFSET_ON_SURFACE, "OffsetCrvOnSrf", Geometry::Curve(curve)))
    }

    pub fn offset_on_surface_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.offset_on_surface_pending.is_some() {
            vec![("Distance=", "Distance="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
