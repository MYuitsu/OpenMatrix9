//! Offset Normal to Surface (`Offset Normal`, OM9-CURVE-057).

use om9_core::geometry::{Curve, Geometry, NurbsSurface};

use super::{Controller, Error};

pub const OFFSET_NORMAL: &str = "OM9-CURVE-057";

#[derive(Debug, Clone)]
pub struct OffsetNormalPending {
    pub curve: Curve,
    pub surface: NurbsSurface,
    pub distance: f64,
}

impl OffsetNormalPending {
    pub fn prompt(&self) -> &'static str {
        "Distance=, or Enter to offset by 1"
    }
}

impl Controller {
    pub fn dispatch_offset_normal(&mut self) -> Result<(), Error> {
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
        self.offset_normal_pending = Some(OffsetNormalPending {
            curve,
            surface,
            distance: 1.0,
        });
        Ok(())
    }

    pub fn submit_offset_normal_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.offset_normal_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Distance=") {
            let Some(distance) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value != 0.0) else {
                self.offset_normal_pending = Some(pending);
                return Some(Err(Error::Invalid("distance must be non-zero")));
            };
            pending.distance = distance;
            self.offset_normal_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.offset_normal_pending = Some(pending);
            return Some(Err(Error::Invalid("Offset Normal accepts Distance= or Enter")));
        }
        let samples = match om9_core::surface_kernel::sample_curve(
            &pending.curve,
            48,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let mut points = Vec::with_capacity(samples.len());
        for point in samples {
            let projection = match om9_core::curve_surface_ops::closest_point_on_surface(
                &pending.surface,
                point,
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
            points.push([
                projection.point[0] + normal[0] * pending.distance,
                projection.point[1] + normal[1] * pending.distance,
                projection.point[2] + normal[2] * pending.distance,
            ]);
        }
        let curve = match om9_core::curve_build::interpolate(
            &points,
            3,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(
            OFFSET_NORMAL,
            "Offset Normal",
            Geometry::Curve(Curve::Nurbs(curve)),
        ))
    }

    pub fn offset_normal_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.offset_normal_pending.is_some() {
            vec![("Distance=", "Distance="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
