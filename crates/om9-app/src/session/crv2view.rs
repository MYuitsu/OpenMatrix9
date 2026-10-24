//! Curve from 2 Views (`Crv2View`, OM9-CURVE-013).

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const CRV2VIEW: &str = "OM9-CURVE-013";

#[derive(Debug, Clone)]
pub struct Crv2ViewPending {
    pub first: Curve,
    pub second: Curve,
}

impl Crv2ViewPending {
    pub fn prompt(&self) -> &'static str {
        "Enter to build the curve from two views"
    }
}

impl Controller {
    pub fn dispatch_crv2view(&mut self) -> Result<(), Error> {
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
        let span = |curve: &Curve| {
            om9_core::surface_kernel::sample_curve(curve, 32, self.coordinator.limits())
                .map(|samples| {
                    let min = samples.iter().map(|point| point[2]).fold(f64::MAX, f64::min);
                    let max = samples.iter().map(|point| point[2]).fold(f64::MIN, f64::max);
                    max - min
                })
                .unwrap_or(0.0)
        };
        let (first, second) = if span(&curves[0]) > span(&curves[1]) {
            (curves[1].clone(), curves[0].clone())
        } else {
            (curves[0].clone(), curves[1].clone())
        };
        self.crv2view_pending = Some(Crv2ViewPending { first, second });
        Ok(())
    }

    pub fn submit_crv2view_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let pending = self.crv2view_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.crv2view_pending = Some(pending);
            return Some(Err(Error::Invalid("Crv2View accepts Enter or Cancel")));
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
        let mut points = Vec::with_capacity(count);
        for index in 0..count {
            let a = first[index];
            let b = second[index];
            points.push([a[0], a[1], b[2]]);
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
        Some(self.commit_geometry(CRV2VIEW, "Crv2View", Geometry::Curve(Curve::Nurbs(curve))))
    }

    pub fn crv2view_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.crv2view_pending.is_some() {
            vec![("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
