// SPDX-License-Identifier: LGPL-2.1-or-later
//! One revision-consistent publication for geometry, reports and recipes.
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::{Geometry, GeometryKind},
    history::{EvaluationState, Recipe},
    identity::*,
    model::{Document, Entity},
    request::SourceBinding,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterMap {
    Identity,
    Affine { scale: f64, offset: f64 },
    Unknown,
    Unsupported,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Exactness {
    ExactLocus,
    Approximate { max_deviation_mm: f64 },
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionReport {
    pub from: GeometryKind,
    pub to: GeometryKind,
    pub exactness: Exactness,
    pub parameter_map: ParameterMap,
    pub preserved: Vec<String>,
    pub dropped: Vec<String>,
}
impl ConversionReport {
    pub fn identity(kind: GeometryKind) -> Self {
        Self {
            from: kind,
            to: kind,
            exactness: Exactness::ExactLocus,
            parameter_map: ParameterMap::Identity,
            preserved: vec!["geometry representation and parameterization".into()],
            dropped: vec![],
        }
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if let Exactness::Approximate { max_deviation_mm } = self.exactness
            && (!max_deviation_mm.is_finite() || max_deviation_mm < 0.)
        {
            return Err(Error::Invalid("conversion deviation"));
        }
        if let ParameterMap::Affine { scale, offset } = self.parameter_map
            && (!scale.is_finite() || scale == 0. || !offset.is_finite())
        {
            return Err(Error::Invalid("parameter map"));
        }
        let mut u = Usage::default();
        u.add(Usage::array::<String>(self.preserved.capacity())?, l)?;
        u.add(Usage::array::<String>(self.dropped.capacity())?, l)?;
        for s in self.preserved.iter().chain(&self.dropped) {
            name(s, l)?;
            u.add(
                Usage {
                    bytes: s.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        Ok(u)
    }
    pub fn map_parameter(&self, p: f64) -> Result<f64> {
        if !p.is_finite() {
            return Err(Error::Invalid("conversion parameter"));
        }
        let q = match self.parameter_map {
            ParameterMap::Identity => p,
            ParameterMap::Affine { scale, offset } => scale * p + offset,
            _ => return Err(Error::Unsupported("unproved parameter map")),
        };
        if !q.is_finite() {
            return Err(Error::Invalid("mapped parameter"));
        }
        Ok(q)
    }
}
#[derive(Debug, Clone)]
pub struct PreparedOutput {
    pub entity: Entity,
    pub expected_existing: Option<SourceStamp>,
    pub geometry_snapshot: Geometry,
    pub geometry_result: Geometry,
    pub source_stamps: Vec<SourceStamp>,
    pub conversion: ConversionReport,
}
#[derive(Debug, Clone)]
pub struct PreparedResult {
    pub document: DocumentId,
    pub operation: String,
    pub sources: Vec<SourceBinding>,
    pub outputs: Vec<PreparedOutput>,
    pub recipe: Option<Recipe>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultStatus {
    Current,
    Stale,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedResult {
    pub operation: String,
    pub geometry_revision: u64,
    pub geometry_snapshot: Geometry,
    pub source_stamps: Vec<SourceStamp>,
    pub conversion: ConversionReport,
    pub recipe: Option<HistoryId>,
    pub status: ResultStatus,
}
impl PublishedResult {
    pub fn validate(&self, id: ObjectId, d: &Document, l: &Limits) -> Result<Usage> {
        name(&self.operation, l)?;
        let entity = d.entities.get(&id).ok_or(Error::Missing("result output"))?;
        if self.status == ResultStatus::Current {
            if self.geometry_revision != entity.geometry_revision
                || self.geometry_snapshot != entity.geometry
            {
                return Err(Error::Stale("published geometry/result"));
            }
            for source in &self.source_stamps {
                d.resolve(source)?;
            }
        }
        if let Some(recipe) = self.recipe
            && !d
                .history
                .recipes
                .get(&recipe)
                .is_some_and(|r| r.outputs.contains(&id))
        {
            return Err(Error::Missing("result recipe owner"));
        }
        let mut u = Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.operation.capacity() as u64,
            ..Usage::default()
        };
        u.add(self.geometry_snapshot.validate(l)?, l)?;
        u.add(
            Usage::array::<SourceStamp>(self.source_stamps.capacity())?,
            l,
        )?;
        u.add(self.conversion.validate(l)?, l)?;
        Ok(u)
    }
}
impl PreparedResult {
    pub(crate) fn validate(&self, d: &Document, l: &Limits) -> Result<Usage> {
        if self.document != d.id || self.operation.is_empty() || self.outputs.is_empty() {
            return Err(Error::Invalid("prepared operation/document"));
        }
        name(&self.operation, l)?;
        let stamps: Vec<_> = self.sources.iter().map(|s| s.source).collect();
        let mut roles = BTreeSet::new();
        for input in &self.sources {
            d.resolve(&input.source)?;
            name(&input.role, l)?;
            if input.role.is_empty()
                || !roles.insert((&input.role, input.source, input.selection.clone()))
            {
                return Err(Error::Invalid("prepared duplicate source role/path"));
            }
        }
        let mut u = Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.operation.capacity() as u64,
            outputs: self.outputs.len() as u64,
            ..Usage::default()
        };
        u.add(Usage::array::<SourceBinding>(self.sources.capacity())?, l)?;
        u.add(Usage::array::<PreparedOutput>(self.outputs.capacity())?, l)?;
        for input in &self.sources {
            u.add(input.owned_usage(l)?, l)?;
        }
        let mut ids = BTreeSet::new();
        for output in &self.outputs {
            let entity = &output.entity;
            if entity.id.is_nil()
                || !ids.insert(entity.id)
                || entity.geometry != output.geometry_snapshot
                || entity.geometry != output.geometry_result
                || output.source_stamps != stamps
                || output.conversion.to != entity.geometry.kind()
            {
                return Err(Error::Invalid(
                    "prepared Shape/snapshot/result/report/source consistency",
                ));
            }
            if !d
                .layer_permissions(entity.attributes.layer, false)?
                .editable
            {
                return Err(Error::Permission("prepared output layer"));
            }
            entity.placement.validate()?;
            name(&entity.label, l)?;
            match output.expected_existing {
                Some(stamp) => {
                    let old = d.resolve(&stamp)?;
                    if old.id != entity.id || old.generation != entity.generation {
                        return Err(Error::Stale("prepared output identity"));
                    }
                    if !d.permissions(old.id, false)?.editable {
                        return Err(Error::Permission("prepared output edit"));
                    }
                    if let Some(owner) = d.history.owner(old.id)
                        && self.recipe.as_ref().map(|r| r.id) != Some(owner)
                    {
                        return Err(Error::Permission("recipe-owned output replacement"));
                    }
                }
                None => {
                    if d.entities.contains_key(&entity.id) {
                        return Err(Error::Invalid("new output collides with existing ID"));
                    }
                }
            }
            for g in [
                &entity.geometry,
                &output.geometry_snapshot,
                &output.geometry_result,
            ] {
                u.add(g.validate(l)?, l)?;
            }
            u.add(output.conversion.validate(l)?, l)?;
            u.add(
                Usage::array::<SourceStamp>(output.source_stamps.capacity())?,
                l,
            )?;
        }
        if let Some(recipe) = &self.recipe {
            if recipe.document != d.id
                || recipe.command != self.operation
                || recipe.inputs != self.sources
                || recipe.outputs != self.outputs.iter().map(|o| o.entity.id).collect::<Vec<_>>()
                || recipe.state != EvaluationState::Clean
            {
                return Err(Error::Invalid("prepared recipe/result consistency"));
            }
            // Output existence and graph validation follow private insertion.
            recipe.environment.frame.validate()?;
            recipe.environment.tolerance.validate()?;
        }
        Ok(u)
    }
    pub(crate) fn apply(self, d: &mut Document) -> Result<()> {
        let recipe_id = self.recipe.as_ref().map(|r| r.id);
        for output in self.outputs {
            let mut entity = output.entity;
            if let Some(old) = output.expected_existing {
                entity.geometry_revision = old
                    .geometry_revision
                    .checked_add(1)
                    .ok_or(Error::Overflow("replay geometry revision"))?;
                entity.attributes_revision = old
                    .attributes_revision
                    .checked_add(1)
                    .ok_or(Error::Overflow("replay attributes revision"))?;
                d.invalidate_results(entity.id, false)?;
            }
            d.results.insert(
                entity.id,
                PublishedResult {
                    operation: self.operation.clone(),
                    geometry_revision: entity.geometry_revision,
                    geometry_snapshot: output.geometry_snapshot,
                    source_stamps: output.source_stamps,
                    conversion: output.conversion,
                    recipe: recipe_id,
                    status: ResultStatus::Current,
                },
            );
            d.entities.insert(entity.id, entity);
        }
        if let Some(mut recipe) = self.recipe {
            if let Some(previous) = d.history.recipes.get(&recipe.id) {
                recipe.revision = previous
                    .revision
                    .checked_add(1)
                    .ok_or(Error::Overflow("replay recipe revision"))?;
            }
            d.history.recipes.insert(recipe.id, recipe);
        }
        Ok(())
    }
}
