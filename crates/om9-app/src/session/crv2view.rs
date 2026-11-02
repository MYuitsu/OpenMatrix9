//! Curve from 2 Views (`Crv2View`, OM9-CURVE-013) and the Matrix builder variant
//! `Curve 2 Views with History` (`Crv2ViewHistory`, OM9-CURVE-014).
//!
//! Both rows share one construction: lift the two planar view curves onto the world
//! curve and interpolate through the projected stations. The builder row records the two
//! selected view curves as history inputs, so `Controller::replay_history` can rebuild the
//! same curve after a source edit. The menu row and the typed command always reach one
//! feature id, and the history variant never claims a separate geometry service.

use om9_core::budget::Limits;
use om9_core::geometry::{Curve, Geometry};
use om9_core::identity::ObjectId;
use om9_core::request::{Selection, SourceBinding};

use super::{Controller, Error};

pub const CRV2VIEW: &str = "OM9-CURVE-013";
/// `Curve 2 Views with History` — the builder row of the same two-view construction.
pub const CRV2VIEW_HISTORY: &str = "OM9-CURVE-014";

#[derive(Debug, Clone)]
pub struct Crv2ViewPending {
    pub first: Curve,
    pub second: Curve,
    pub ids: Vec<ObjectId>,
    /// Whether the commit must record the two view curves as a history recipe.
    pub history: bool,
}

impl Crv2ViewPending {
    pub fn prompt(&self) -> &'static str {
        if self.history {
            "Enter to build the curve from two views with history"
        } else {
            "Enter to build the curve from two views"
        }
    }
}

/// Build the world curve from the two planar view curves, smaller view span first.
///
/// The two selections arrive in pick order; the curve with the smaller Z span is the
/// front view and supplies X/Y, the other one supplies Z. Keeping this in one function is
/// what lets the plain and history rows share exactly one construction (and one replay).
pub fn build_from_two_views(
    first: &Curve,
    second: &Curve,
    limits: &Limits,
) -> Result<Curve, Error> {
    let first = om9_core::surface_kernel::sample_curve(first, 32, limits)?;
    let second = om9_core::surface_kernel::sample_curve(second, 32, limits)?;
    let count = first.len().min(second.len());
    let mut points = Vec::with_capacity(count);
    for index in 0..count {
        let a = first[index];
        let b = second[index];
        points.push([a[0], a[1], b[2]]);
    }
    let curve = om9_core::curve_build::interpolate(
        &points,
        3,
        om9_core::curve_build::KnotMode::ChordLength,
        limits,
    )?;
    Ok(Curve::Nurbs(curve))
}

fn z_span(curve: &Curve, limits: &Limits) -> f64 {
    om9_core::surface_kernel::sample_curve(curve, 32, limits)
        .map(|samples| {
            let min = samples
                .iter()
                .map(|point| point[2])
                .fold(f64::MAX, f64::min);
            let max = samples
                .iter()
                .map(|point| point[2])
                .fold(f64::MIN, f64::max);
            max - min
        })
        .unwrap_or(0.0)
}

impl Controller {
    pub fn dispatch_crv2view(&mut self, history: bool) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let limits = *self.coordinator.limits();
        let mut picked = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            picked.push((id, curve.clone()));
        }
        picked.sort_by(|a, b| {
            z_span(&a.1, &limits)
                .partial_cmp(&z_span(&b.1, &limits))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let (first_id, first) = picked[0].clone();
        let (second_id, second) = picked[1].clone();
        self.crv2view_pending = Some(Crv2ViewPending {
            first,
            second,
            ids: vec![first_id, second_id],
            history,
        });
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
        let limits = *self.coordinator.limits();
        let curve = match build_from_two_views(&pending.first, &pending.second, &limits) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        if !pending.history {
            return Some(self.commit_geometry(CRV2VIEW, "Crv2View", Geometry::Curve(curve)));
        }
        let entity = om9_core::model::Entity::new(
            "Crv2ViewHistory",
            Geometry::Curve(curve),
            self.document().current_layer,
        );
        let mut inputs = Vec::with_capacity(pending.ids.len());
        for id in &pending.ids {
            let source = match self.document().stamp(*id) {
                Ok(stamp) => stamp,
                Err(error) => return Some(Err(error)),
            };
            inputs.push(SourceBinding {
                role: "view".into(),
                source,
                selection: Selection::Whole,
            });
        }
        Some(self.commit_entity_with_history(
            CRV2VIEW_HISTORY,
            "Crv2ViewHistory",
            entity,
            "Crv2ViewHistory",
            inputs,
        ))
    }

    /// Options exposed while the two-view session waits for its commit. Both rows accept
    /// Enter (commit) and Cancel (write nothing); the history row adds no parameter of its
    /// own, it records the same construction.
    pub fn crv2view_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.crv2view_pending.is_some() {
            vec![("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}
