//! Host-owned creation of a joined curve and its associative recipe.
use om9_core::{
    Error, Result,
    geometry::Geometry,
    model::Entity,
    request::{Options, Selection, SourceBinding, Value},
    units::LengthMm,
};

use super::Controller;

impl Controller {
    /// Create the result and recipe atomically. User-facing routes remain disabled until
    /// source-edit propagation is integrated with the host transaction lifecycle.
    pub fn create_join_history(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let ids = self.selected_ordered();
        let tolerance = self.document().effective_tolerance()?.absolute_mm;
        let mut inputs = Vec::new();
        let mut entities = Vec::new();
        for id in ids {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("join source"))?;
            inputs.push(SourceBinding {
                role: "curve".into(),
                source: self.document().stamp(id)?,
                selection: Selection::Whole,
            });
            entities.push(entity);
        }
        let curve =
            om9_core::top11::join_curve_entities(&entities, tolerance, self.coordinator.limits())?;
        let entity = Entity::new(
            "JoinHistory",
            Geometry::Curve(curve),
            self.document().current_layer,
        );
        let mut parameters = Options::new();
        parameters.insert(
            "tolerance_mm".into(),
            Value::length(LengthMm::try_new(tolerance)?),
        );
        self.commit_entity_with_history_inner(
            "gvJoinHistory",
            "JoinHistory",
            entity,
            om9_core::producers::JOIN_HISTORY_COMMAND,
            inputs,
            None,
            parameters,
            true,
        )
    }
}
