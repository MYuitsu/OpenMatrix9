//! Quick Curve Blend (`Blend`, OM9-CURVE-011): G1 bridge between two curves.

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const BLEND: &str = "OM9-CURVE-011";
pub const BLEND_CRV: &str = "OM9-CURVE-012";
pub const MATCH: &str = "OM9-CURVE-052";

#[derive(Debug, Clone)]
pub struct BlendPending {
    pub feature: &'static str,
    pub first: Curve,
    pub second: Curve,
    pub scale1: f64,
    pub scale2: f64,
}

impl BlendPending {
    pub fn prompt(&self) -> &'static str {
        "Scale1=, Scale2= or Enter to blend"
    }
}

impl Controller {
    pub fn dispatch_blend(&mut self) -> Result<(), Error> {
        self.dispatch_blend_as(BLEND)
    }

    pub fn dispatch_match(&mut self) -> Result<(), Error> {
        self.dispatch_blend_as(MATCH)
    }

    fn dispatch_blend_as(&mut self, feature: &'static str) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
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
        self.blend_pending = Some(BlendPending {
            feature,
            first: curves[0].clone(),
            second: curves[1].clone(),
            scale1: 1.0,
            scale2: 1.0,
        });
        Ok(())
    }

    pub fn submit_blend_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.blend_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Scale1=") {
            let Some(scale) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.blend_pending = Some(pending);
                return Some(Err(Error::Invalid("Scale1 must be positive")));
            };
            pending.scale1 = scale;
            self.blend_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Scale2=") {
            let Some(scale) = value.parse::<f64>().ok().filter(|value| value.is_finite() && *value > 0.0) else {
                self.blend_pending = Some(pending);
                return Some(Err(Error::Invalid("Scale2 must be positive")));
            };
            pending.scale2 = scale;
            self.blend_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.blend_pending = Some(pending);
            return Some(Err(Error::Invalid("Blend accepts Scale1=, Scale2=, Enter or Cancel")));
        }
        let first = match om9_core::surface_kernel::sample_curve(
            &pending.first,
            16,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let second = match om9_core::surface_kernel::sample_curve(
            &pending.second,
            16,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        if first.len() < 2 || second.len() < 2 {
            return Some(Err(Error::Invalid("blend source samples")));
        }
        let a = first[first.len() - 1];
        let before = first[first.len() - 2];
        let b = second[0];
        let after = second[1];
        let length = distance(a, b).max(1e-6);
        let tangent_a = unit([a[0] - before[0], a[1] - before[1], a[2] - before[2]]).unwrap_or([1.0, 0.0, 0.0]);
        let tangent_b = unit([after[0] - b[0], after[1] - b[1], after[2] - b[2]]).unwrap_or([1.0, 0.0, 0.0]);
        let points = [
            a,
            [
                a[0] + tangent_a[0] * length / 3.0 * pending.scale1,
                a[1] + tangent_a[1] * length / 3.0 * pending.scale1,
                a[2] + tangent_a[2] * length / 3.0 * pending.scale1,
            ],
            [
                b[0] - tangent_b[0] * length / 3.0 * pending.scale2,
                b[1] - tangent_b[1] * length / 3.0 * pending.scale2,
                b[2] - tangent_b[2] * length / 3.0 * pending.scale2,
            ],
            b,
        ];
        let curve = match om9_core::curve_build::interpolate(
            &points,
            3,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        let label = if pending.feature == MATCH { "Match" } else { "Blend" };
        Some(self.commit_geometry(
            pending.feature,
            label,
            Geometry::Curve(Curve::Nurbs(curve)),
        ))
    }

    pub fn blend_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.blend_pending.is_some() {
            vec![
                ("Scale1=", "Scale1="),
                ("Scale2=", "Scale2="),
                ("Cancel", "Cancel"),
            ]
        } else {
            Vec::new()
        }
    }
}

fn unit(vector: Point3) -> Option<Point3> {
    let length = (vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2)).sqrt();
    if length <= 1e-12 {
        return None;
    }
    Some([vector[0] / length, vector[1] / length, vector[2] / length])
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
