// SPDX-License-Identifier: LGPL-2.1-or-later
//! Dir holds pending flips until the user accepts them with Enter.
use super::{CommandContext, Controller};
use om9_core::{
    Error, Result, curve_ops,
    geometry::{Curve, Geometry, Point3},
    identity::ObjectId,
    transaction::{Mutation, TransactionRequest},
};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(crate) enum ArrowLocation {
    Fraction(f64),
    Hover(Point3),
}

pub(crate) struct Session {
    pub(super) context: CommandContext,
    pub(crate) targets: BTreeSet<ObjectId>,
    flipped: BTreeSet<ObjectId>,
    collecting: bool,
}
impl Session {
    pub(crate) fn prompt(&self) -> &'static str {
        if self.collecting {
            "Select curves. Press Enter when done: "
        } else if self.targets.len() == 1 {
            "Press Enter when done ( Flip ): "
        } else {
            "Select object to flip direction. Press Enter when done ( FlipAll ): "
        }
    }
    pub(super) fn option(&self) -> Option<&'static str> {
        if self.collecting {
            None
        } else if self.targets.len() == 1 {
            Some("Flip")
        } else {
            Some("FlipAll")
        }
    }
    pub(crate) fn start_fraction(&self, id: ObjectId) -> f64 {
        if self.flipped.contains(&id) { 1. } else { 0. }
    }

    fn toggle(&mut self, id: ObjectId) {
        if !self.flipped.remove(&id) {
            self.flipped.insert(id);
        }
    }
}
impl Controller {
    fn direction_curve(&self, id: ObjectId) -> Result<(&Curve, om9_core::geometry::Transform)> {
        let doc = self.document();
        let entity = doc
            .entities
            .get(&id)
            .ok_or(Error::Missing("direction curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Unsupported("select an editable curve"));
        };
        if !doc.permissions(id, false)?.editable {
            return Err(Error::Invalid("curve is locked"));
        }
        Ok((curve, entity.placement))
    }
    pub(super) fn curve_direction_unavailable(&self) -> Option<&'static str> {
        self.selected()
            .iter()
            .any(|&id| self.direction_curve(id).is_err())
            .then_some("Select editable curves for Curve Direction")
    }

    pub(super) fn start_curve_direction(&mut self) -> Result<()> {
        let original = self.context();
        for &id in &original.selection {
            self.direction_curve(id)?;
        }
        let targets: BTreeSet<_> = original.selection.iter().copied().collect();
        self.curve_direction = Some(Session {
            collecting: targets.is_empty(),
            targets,
            context: original,
            flipped: BTreeSet::new(),
        });
        Ok(())
    }
    pub(crate) fn direction_pick(&mut self, id: ObjectId) -> Result<()> {
        let session = self
            .curve_direction
            .as_ref()
            .ok_or(Error::Missing("Dir session"))?;
        self.validate_context(&session.context)?;
        self.direction_curve(id)?;
        if session.collecting {
            if !session.targets.contains(&id) {
                let mut selected: Vec<_> = session.targets.iter().copied().collect();
                selected.push(id);
                self.select_objects(&selected)?;
                self.curve_direction.as_mut().unwrap().targets.insert(id);
            }
        } else if session.targets.contains(&id) {
            self.curve_direction.as_mut().unwrap().toggle(id);
        }
        Ok(())
    }
    pub(crate) fn direction_arrow(
        &self,
        id: ObjectId,
        location: ArrowLocation,
    ) -> Result<(Point3, Point3)> {
        let session = self
            .curve_direction
            .as_ref()
            .ok_or(Error::Missing("Dir session"))?;
        self.validate_context(&session.context)?;
        let (curve, placement) = self.direction_curve(id)?;
        let limits = self.coordinator.limits();
        let fraction = match location {
            ArrowLocation::Hover(point) => {
                om9_core::curve_surface_ops::closest_point_on_placed_curve(
                    curve, placement, point, limits,
                )?
                .0
            }
            ArrowLocation::Fraction(fraction) => fraction,
        };
        let point = curve_ops::point_at(curve, fraction, limits)?;
        let tangent = curve_ops::tangent_at(curve, fraction, limits)?;
        let sign = if session.flipped.contains(&id) {
            -1.
        } else {
            1.
        };
        let tip = std::array::from_fn(|i| point[i] + sign * tangent[i]);
        Ok((placement.point(point)?, placement.point(tip)?))
    }
    pub(super) fn direction_input(&mut self, input: &str) -> Result<()> {
        let token = input.trim().trim_start_matches('_');
        if token.eq_ignore_ascii_case("cancel") || token.eq_ignore_ascii_case("esc") {
            self.cancel();
            return Ok(());
        }
        let session = self
            .curve_direction
            .as_ref()
            .ok_or(Error::Missing("Dir session"))?;
        self.validate_context(&session.context)?;
        if session
            .option()
            .is_some_and(|option| token.eq_ignore_ascii_case(option))
        {
            let session = self.curve_direction.as_mut().unwrap();
            for id in &session.targets {
                if !session.flipped.remove(id) {
                    session.flipped.insert(*id);
                }
            }
            return Ok(());
        }
        if !token.is_empty() && !token.eq_ignore_ascii_case("enter") {
            return Err(Error::Invalid(session.prompt()));
        }
        if session.collecting {
            let session = self.curve_direction.as_mut().unwrap();
            if session.targets.is_empty() {
                return Err(Error::Missing("select curves"));
            }
            session.collecting = false;
            return Ok(());
        }
        if !session.flipped.is_empty() {
            let mut mutations = Vec::new();
            mutations
                .try_reserve_exact(session.flipped.len())
                .map_err(|_| Error::Budget("direction edits"))?;
            for &id in &session.flipped {
                let (curve, _) = self.direction_curve(id)?;
                mutations.push(Mutation::Geometry {
                    source: self.document().stamp(id)?,
                    geometry: Geometry::Curve(curve_ops::reverse_curve_for_dir(
                        curve,
                        self.coordinator.limits(),
                    )?),
                });
            }
            if self.source_mutations_update_join_history(&mutations) {
                self.request_join_history_edit_worker(TransactionRequest::new("Dir", mutations))?;
            } else {
                self.commit_utility_mutations("Dir", mutations)?;
            }
        }
        self.curve_direction = None;
        Ok(())
    }
}
