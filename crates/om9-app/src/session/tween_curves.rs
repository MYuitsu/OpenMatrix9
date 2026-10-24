//! Tween Curves (`TweenCurves`, OM9-CURVE-038): midpoint between two curves.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const TWEEN_CURVES: &str = "OM9-CURVE-038";

#[derive(Debug, Clone)]
pub struct TweenCurvesPending {
    pub first: Curve,
    pub second: Curve,
    pub count: usize,
}

impl TweenCurvesPending {
    pub fn prompt(&self) -> &'static str {
        "Enter to tween the two curves"
    }
}

impl Controller {
    pub fn dispatch_tween_curves(&mut self) -> Result<(), Error> {
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
        self.tween_curves_pending = Some(TweenCurvesPending {
            first: curves[0].clone(),
            second: curves[1].clone(),
            count: 1,
        });
        Ok(())
    }

    pub fn submit_tween_curves_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let pending = self.tween_curves_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Count=") {
            let Some(count) = value.parse::<usize>().ok().filter(|value| (1..=64).contains(value)) else {
                self.tween_curves_pending = Some(pending);
                return Some(Err(Error::Invalid("count must be 1..64")));
            };
            let mut pending = pending;
            pending.count = count;
            self.tween_curves_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.tween_curves_pending = Some(pending);
            return Some(Err(Error::Invalid("TweenCurves accepts Enter or Cancel")));
        }
        let first = match om9_core::surface_kernel::sample_curve(
            &pending.first,
            32,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let second = match om9_core::surface_kernel::sample_curve(
            &pending.second,
            32,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let count = first.len().min(second.len());
        for step in 1..=pending.count {
            let fraction = step as f64 / (pending.count + 1) as f64;
            let mut points = Vec::with_capacity(count);
            for index in 0..count {
                points.push([
                    first[index][0] + (second[index][0] - first[index][0]) * fraction,
                    first[index][1] + (second[index][1] - first[index][1]) * fraction,
                    first[index][2] + (second[index][2] - first[index][2]) * fraction,
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
            if let Err(error) = self.commit_geometry(
                TWEEN_CURVES,
                "TweenCurves",
                Geometry::Curve(Curve::Nurbs(curve)),
            ) {
                return Some(Err(error));
            }
        }
        Some(Ok(()))
    }

    pub fn tween_curves_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.tween_curves_pending.is_some() {
            vec![("Count=", "Count="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
