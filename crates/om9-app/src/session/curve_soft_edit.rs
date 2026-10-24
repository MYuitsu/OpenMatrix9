//! Curve Soft Edit (`CurveSoftEdit`, OM9-CURVE-055): move one edit point.

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const CURVE_SOFT_EDIT: &str = "OM9-CURVE-055";

#[derive(Debug, Clone)]
pub struct CurveSoftEditPending {
    pub curve: Curve,
    pub index: usize,
    pub target: Point3,
    pub has_target: bool,
}

impl CurveSoftEditPending {
    pub fn prompt(&self) -> &'static str {
        "Index=, Point=x,y,z, or Enter to apply"
    }
}

impl Controller {
    pub fn dispatch_curve_soft_edit(&mut self) -> Result<(), Error> {
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
        self.curve_soft_edit_pending = Some(CurveSoftEditPending {
            curve: curve.clone(),
            index: 0,
            target: [0.0, 0.0, 0.0],
            has_target: false,
        });
        Ok(())
    }

    pub fn submit_curve_soft_edit_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.curve_soft_edit_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Index=") {
            let Some(index) = value.parse::<usize>().ok() else {
                self.curve_soft_edit_pending = Some(pending);
                return Some(Err(Error::Invalid("edit index")));
            };
            pending.index = index;
            self.curve_soft_edit_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Point=") {
            let parts: Vec<f64> = value
                .split(',')
                .filter_map(|part| part.trim().parse::<f64>().ok())
                .collect();
            if parts.len() != 3 || parts.iter().any(|value| !value.is_finite()) {
                self.curve_soft_edit_pending = Some(pending);
                return Some(Err(Error::Invalid("Point=x,y,z")));
            }
            pending.target = [parts[0], parts[1], parts[2]];
            pending.has_target = true;
            self.curve_soft_edit_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.curve_soft_edit_pending = Some(pending);
            return Some(Err(Error::Invalid("CurveSoftEdit accepts Index=, Point= or Enter")));
        }
        if !pending.has_target {
            self.curve_soft_edit_pending = Some(pending);
            return Some(Err(Error::Invalid("Point= is required")));
        }
        let curve = match om9_core::curve_ops::move_edit_point(
            &pending.curve,
            pending.index,
            pending.target,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(CURVE_SOFT_EDIT, "CurveSoftEdit", Geometry::Curve(curve)))
    }

    pub fn curve_soft_edit_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.curve_soft_edit_pending.is_some() {
            vec![
                ("Index=", "Index="),
                ("Point=", "Point="),
                ("Cancel", "Cancel"),
            ]
        } else {
            Vec::new()
        }
    }
}
