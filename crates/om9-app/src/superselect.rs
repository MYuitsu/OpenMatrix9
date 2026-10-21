// SPDX-License-Identifier: LGPL-2.1-or-later
//! SuperSelect: choose objects by a TYPED property (OM9-L3-03-14-006).
//!
//! Matrix8 book 1, pages 32-34: "Super Select — This tool determines what types of
//! objects you …". The plan adds the rule that matters here: a gem is a
//! jewellery-semantic gem, not a mesh that merely looks like one, and an
//! open/bad verdict must come from a typed kernel judgement instead of a guess.
use crate::session::Controller;
use om9_core::identity::ObjectId;
use om9_core::model::{Document, Entity, GEM_EXTENSION_SLOT};

/// What SuperSelect is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectFilter {
    /// Objects carrying the `om9.gem` jewellery-semantic extension.
    Gems,
    /// Objects whose geometry fails the model's own validation.
    BadGeometry,
    /// Objects with a typed open-surface verdict.
    OpenGeometry,
    /// Objects whose label repeats an earlier object's label.
    Duplicate,
}

/// Why an object did not match, said plainly instead of silently dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// No `om9.gem` extension: a name or a colour is not a gem.
    NoGemExtension,
    /// The geometry validates; it is not bad.
    GeometryValid,
    /// No typed open/closed verdict exists for this geometry, so nothing is
    /// claimed either way (the same rule as Object Info's unavailable size).
    NoOpenVerdict(&'static str),
    /// The label is the first of its kind.
    UniqueLabel,
}

/// What one SuperSelect gesture found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SuperselectOutcome {
    pub matched: Vec<ObjectId>,
    pub skipped: Vec<(ObjectId, SkipReason)>,
}

impl SuperselectOutcome {
    pub fn matched_count(&self) -> usize {
        self.matched.len()
    }
}

/// The typed judgement of one object for one filter.
pub fn classify(
    document: &Document,
    entity: &Entity,
    filter: SelectFilter,
    limits: &om9_core::budget::Limits,
) -> Option<SkipReason> {
    match filter {
        SelectFilter::Gems => {
            let semantic = entity
                .extension(GEM_EXTENSION_SLOT)
                .is_some_and(|extension| {
                    extension.version >= 1 && !extension.payload.trim().is_empty()
                });
            if semantic {
                None
            } else {
                Some(SkipReason::NoGemExtension)
            }
        }
        SelectFilter::BadGeometry => {
            if entity.geometry.validate(limits).is_err() {
                None
            } else {
                Some(SkipReason::GeometryValid)
            }
        }
        SelectFilter::OpenGeometry => Some(SkipReason::NoOpenVerdict(
            "no typed open/closed verdict is recorded for this geometry yet",
        )),
        SelectFilter::Duplicate => {
            let key = (entity.label.to_lowercase(), entity.geometry.kind().tag());
            let earlier = document.entities.values().any(|other| {
                other.id < entity.id
                    && (other.label.to_lowercase(), other.geometry.kind().tag()) == key
            });
            if earlier {
                None
            } else {
                Some(SkipReason::UniqueLabel)
            }
        }
    }
}

impl Controller {
    /// Read-only: what the filter would match.
    pub fn superselect(&self, filter: SelectFilter) -> SuperselectOutcome {
        let limits = *self.coordinator().limits();
        let mut outcome = SuperselectOutcome::default();
        for entity in self.document().entities.values() {
            match classify(self.document(), entity, filter, &limits) {
                None => outcome.matched.push(entity.id),
                Some(reason) => outcome.skipped.push((entity.id, reason)),
            }
        }
        outcome
    }

    /// Apply the filter to the selection (replacing it). Returns how many objects
    /// are selected; the document is never written.
    pub fn apply_superselect(&mut self, filter: SelectFilter) -> usize {
        let outcome = self.superselect(filter);
        let _ = self.select_objects(&outcome.matched);
        outcome.matched_count()
    }
}
