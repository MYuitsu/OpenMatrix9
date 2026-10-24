//! Rebuild command (`Rebuild`, OM9-CURVE-009) ported from the Qt Curve controller.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const REBUILD: &str = "OM9-CURVE-009";

#[derive(Debug, Clone)]
pub struct RebuildPending {
    pub source: Curve,
    pub degree: usize,
    pub points: usize,
}

impl RebuildPending {
    pub fn prompt(&self) -> &'static str {
        "Degree, Points, or Enter to rebuild"
    }
}

impl Controller {
    pub fn dispatch_rebuild(&mut self) -> Result<(), Error> {
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
        self.rebuild_pending = Some(RebuildPending {
            source: curve.clone(),
            degree: 3,
            points: 8,
        });
        Ok(())
    }

    pub fn submit_rebuild_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.rebuild_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Degree=") {
            let Some(degree) = value.parse::<usize>().ok().filter(|value| *value >= 1 && *value <= 11) else {
                self.rebuild_pending = Some(pending);
                return Some(Err(Error::Invalid("degree must be 1..11")));
            };
            pending.degree = degree;
            self.rebuild_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Points=") {
            let Some(points) = value.parse::<usize>().ok().filter(|value| *value >= 4) else {
                self.rebuild_pending = Some(pending);
                return Some(Err(Error::Invalid("points must be at least four")));
            };
            pending.points = points;
            self.rebuild_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.rebuild_pending = Some(pending);
            return Some(Err(Error::Invalid("Rebuild accepts Degree=, Points= or Enter")));
        }
        let curve = match om9_core::curve_build::rebuild(
            &pending.source,
            pending.degree,
            pending.points,
            om9_core::curve_build::KnotMode::ChordLength,
            self.coordinator.limits(),
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(REBUILD, "Rebuild", Geometry::Curve(Curve::Nurbs(curve))))
    }

    pub fn rebuild_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.rebuild_pending.is_some() {
            vec![
                ("Degree=", "Degree="),
                ("Points=", "Points="),
                ("Cancel", "Cancel"),
            ]
        } else {
            Vec::new()
        }
    }
}
