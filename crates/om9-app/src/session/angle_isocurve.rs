//! Angle Isocurve (`AngleIsocurve`, OM9-CURVE-044).

use om9_core::geometry::Geometry;

use super::{Controller, Error};

pub const ANGLE_ISOCURVE: &str = "OM9-CURVE-044";

#[derive(Debug, Clone)]
pub struct AngleIsocurvePending {
    pub surface: om9_core::geometry::NurbsSurface,
    pub angle: f64,
    pub axis: usize,
}

impl AngleIsocurvePending {
    pub fn prompt(&self) -> &'static str {
        "Angle=0..360, Direction=U/V, or Enter to extract"
    }
}

impl Controller {
    pub fn dispatch_angle_isocurve(&mut self) -> Result<(), Error> {
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
        self.angle_isocurve_pending = Some(AngleIsocurvePending {
            surface: surface.clone(),
            angle: 0.0,
            axis: 0,
        });
        Ok(())
    }

    pub fn submit_angle_isocurve_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.angle_isocurve_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Angle=") {
            let Some(angle) = value.parse::<f64>().ok().filter(|value| value.is_finite()) else {
                self.angle_isocurve_pending = Some(pending);
                return Some(Err(Error::Invalid("angle")));
            };
            pending.angle = angle;
            self.angle_isocurve_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Direction=") {
            match value.trim().to_ascii_uppercase().as_str() {
                "U" => pending.axis = 0,
                "V" => pending.axis = 1,
                _ => {
                    self.angle_isocurve_pending = Some(pending);
                    return Some(Err(Error::Invalid("direction must be U or V")));
                }
            }
            self.angle_isocurve_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.angle_isocurve_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "AngleIsocurve accepts Angle=, Direction= or Enter",
            )));
        }
        let turns = (pending.angle / 360.0).rem_euclid(1.0);
        let iso = if pending.axis == 0 {
            om9_core::surface_kernel::SurfaceIso::U
        } else {
            om9_core::surface_kernel::SurfaceIso::V
        };
        let domain = pending.surface.domain[pending.axis];
        let value = domain[0] + turns * (domain[1] - domain[0]);
        let curve = match om9_core::surface_kernel::isocurve(
            &pending.surface,
            iso,
            value,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(
            ANGLE_ISOCURVE,
            "AngleIsocurve",
            Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)),
        ))
    }

    pub fn angle_isocurve_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.angle_isocurve_pending.is_some() {
            vec![
                ("Angle=", "Angle="),
                ("Direction=", "Direction="),
                ("Cancel", "Cancel"),
            ]
        } else {
            Vec::new()
        }
    }
}
