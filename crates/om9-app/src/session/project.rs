//! Project command (`Project`, OM9-CURVE-016): project a curve to the world XY plane.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const PROJECT: &str = "OM9-CURVE-016";

#[derive(Debug, Clone)]
pub struct ProjectPending {
    pub source: Curve,
    pub direction: Option<om9_core::geometry::Point3>,
}

impl ProjectPending {
    pub fn prompt(&self) -> &'static str {
        "Direction=x,y,z, or Enter to project to the construction plane"
    }
}

impl Controller {
    pub fn dispatch_project(&mut self) -> Result<(), Error> {
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
        self.project_pending = Some(ProjectPending {
            source: curve.clone(),
            direction: None,
        });
        Ok(())
    }

    pub fn submit_project_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.project_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Direction=") {
            let values: Vec<f64> = value
                .split(',')
                .filter_map(|value| value.trim().parse::<f64>().ok())
                .collect();
            if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
                self.project_pending = Some(pending);
                return Some(Err(Error::Invalid("Direction=x,y,z")));
            }
            if values.iter().all(|value| value.abs() <= 1e-12) {
                self.project_pending = Some(pending);
                return Some(Err(Error::Invalid("projection direction is zero")));
            }
            pending.direction = Some([values[0], values[1], values[2]]);
            self.project_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.project_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "Project accepts Direction=, Enter or Cancel",
            )));
        }
        let samples = match om9_core::surface_kernel::sample_curve(
            &pending.source,
            64,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let plane = self.input_plane();
        let normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let mut projected = Vec::with_capacity(samples.len());
        for point in samples {
            let point = if let Some(direction) = pending.direction {
                let denominator =
                    direction[0] * normal[0] + direction[1] * normal[1] + direction[2] * normal[2];
                if denominator.abs() <= 1e-12 {
                    return Some(Err(Error::Invalid(
                        "projection direction is parallel to the plane",
                    )));
                }
                let offset = [
                    plane.origin[0] - point[0],
                    plane.origin[1] - point[1],
                    plane.origin[2] - point[2],
                ];
                let t = (offset[0] * normal[0] + offset[1] * normal[1] + offset[2] * normal[2])
                    / denominator;
                [
                    point[0] + direction[0] * t,
                    point[1] + direction[1] * t,
                    point[2] + direction[2] * t,
                ]
            } else {
                match self.project_to_plane(point) {
                    Ok(point) => point,
                    Err(error) => return Some(Err(error)),
                }
            };
            projected.push(point);
        }
        let curve = match om9_core::curve_build::interpolate(
            &projected,
            3,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(PROJECT, "Project", Geometry::Curve(Curve::Nurbs(curve))))
    }

    pub fn project_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.project_pending.is_some() {
            vec![("Direction=", "Direction="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
