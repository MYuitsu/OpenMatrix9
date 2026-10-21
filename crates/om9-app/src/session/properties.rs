// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Object Property Editor adapter (OM9-L3-03-14-007).
//!
//! Matrix8 book 1, pages 32-34: Object Properties show what the model knows about
//! a selected object. The plan's rule is that an edit is ONE document command and
//! that Undo restores the previous value, while a field the model derives (the
//! B-rep source, the geometry itself) can never be typed into.
use super::Controller;
use om9_core::identity::SourceStamp;
use om9_core::model::Attributes;
use om9_core::transaction::{Mutation, TransactionRequest};

/// A field the editor shows for the selected object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyField {
    Name,
    Color,
    Visible,
    Hidden,
    Selectable,
    Locked,
    Geometry,
}

/// Whether the field can be edited, and why not when it cannot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldState {
    pub field: PropertyField,
    pub editable: bool,
    /// Empty when editable; the reason shown disabled otherwise.
    pub reason: &'static str,
}

/// The fields the editor offers, in display order.
pub const FIELDS: [FieldState; 7] = [
    FieldState {
        field: PropertyField::Name,
        editable: true,
        reason: "",
    },
    FieldState {
        field: PropertyField::Color,
        editable: true,
        reason: "",
    },
    FieldState {
        field: PropertyField::Visible,
        editable: true,
        reason: "",
    },
    FieldState {
        field: PropertyField::Hidden,
        editable: true,
        reason: "",
    },
    FieldState {
        field: PropertyField::Selectable,
        editable: true,
        reason: "",
    },
    FieldState {
        field: PropertyField::Locked,
        editable: true,
        reason: "",
    },
    FieldState {
        field: PropertyField::Geometry,
        editable: false,
        reason: "derived from the geometry: read-only",
    },
];

/// One edit the editor can apply. There is deliberately **no** variant for a
/// read-only field: the type system is what disables them, not a run-time check
/// that could be forgotten.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyEdit {
    Label(String),
    Color(Option<[u8; 4]>),
    Visible(bool),
    Hidden(bool),
    Selectable(bool),
    Locked(bool),
}

impl PropertyEdit {
    /// Apply this edit to one attributes record.
    pub fn apply(self, attributes: &mut Attributes, label: &mut String) {
        match self {
            PropertyEdit::Label(value) => *label = value,
            PropertyEdit::Color(color) => attributes.color = color,
            PropertyEdit::Visible(value) => attributes.visible = value,
            PropertyEdit::Hidden(value) => attributes.hidden = value,
            PropertyEdit::Selectable(value) => attributes.selectable = value,
            PropertyEdit::Locked(value) => attributes.locked = value,
        }
    }
}

impl Controller {
    /// Set one property on every given object in ONE transaction: one command, one
    /// journal entry, so Undo restores every previous value together.
    pub fn edit_object_property(
        &mut self,
        sources: Vec<SourceStamp>,
        edit: PropertyEdit,
    ) -> om9_core::Result<()> {
        if sources.is_empty() {
            return Ok(());
        }
        let mut mutations = Vec::with_capacity(sources.len());
        for source in sources {
            let entity = self
                .document()
                .resolve(&source)
                .map_err(|_| om9_core::Error::Stale("property edit source"))?;
            let mut attributes = entity.attributes.clone();
            let mut label = entity.label.clone();
            edit.clone().apply(&mut attributes, &mut label);
            if let PropertyEdit::Label(_) = &edit {
                if label != entity.label {
                    mutations.push(Mutation::Label { source, label });
                }
                continue;
            }
            if attributes == entity.attributes {
                continue;
            }
            mutations.push(Mutation::Attributes { source, attributes });
        }
        if mutations.is_empty() {
            return Ok(());
        }
        let old_scene = self
            .scene
            .as_ref()
            .ok_or(om9_core::Error::Missing("property edit scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(om9_core::Error::Stale("property edit scene"));
        }
        let held = self.scene_cache_usage()?;
        let prepared = self.coordinator.prepare_with_held_usage(
            TransactionRequest::new("Object properties", mutations),
            held,
        )?;
        let epoch = self
            .coordinator
            .epoch()
            .checked_add(1)
            .ok_or(om9_core::Error::Overflow("property edit epoch"))?;
        let scene = old_scene.update_owned(
            prepared.candidate_world().documents[&self.document].clone(),
            epoch,
            None,
            self.coordinator.limits(),
        )?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.scene = Some(scene);
        }
        Ok(())
    }
}
