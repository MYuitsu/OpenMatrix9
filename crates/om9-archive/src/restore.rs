// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    history::{EvaluationState, Recipe},
    identity::{HistoryId, ObjectId, SourceStamp},
    model::Document,
    result::ResultStatus,
};
use std::{collections::BTreeSet, fmt::Write};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReplayKey {
    pub solver: String,
    pub solver_version: String,
    pub schema_version: u32,
    pub command: String,
    pub command_version: u32,
}
pub struct RestoredDocument {
    pub document: Document,
    pub diagnostics: Vec<String>,
}
const DEPENDENCY_REASON: &str = "Unresolved replay dependency";

fn registered(recipe: &Recipe, available: &BTreeSet<ReplayKey>) -> bool {
    // Borrow the recipe strings. Constructing an owned lookup key would copy
    // potentially long strings solely to ask whether a module is available.
    available.iter().any(|key| {
        key.solver == recipe.environment.solver
            && key.solver_version == recipe.environment.solver_version
            && key.schema_version == recipe.environment.schema_version
            && key.command == recipe.command
            && key.command_version == recipe.environment.command_version
    })
}
fn missing_reason_len(recipe: &Recipe) -> Result<usize> {
    let digits = recipe
        .environment
        .command_version
        .checked_ilog10()
        .unwrap_or(0) as usize
        + 1;
    [
        "Missing replay module ".len(),
        recipe.environment.solver.len(),
        1,
        recipe.environment.solver_version.len(),
        " for ".len(),
        recipe.command.len(),
        " version ".len(),
        digits,
    ]
    .into_iter()
    .try_fold(0usize, |n, x| {
        n.checked_add(x)
            .ok_or(Error::Overflow("restore diagnostic bytes"))
    })
}
/// Admit the complete restoration peak before a caller clones its document,
/// and before restoration grows any diagnostic/traversal allocation.
pub(crate) fn admit(
    document: &Document,
    limits: &Limits,
    available: &BTreeSet<ReplayKey>,
) -> Result<usize> {
    document
        .session_generation
        .checked_add(1)
        .ok_or(Error::Overflow("session generation"))?;
    document
        .epoch
        .checked_add(1)
        .ok_or(Error::Overflow("document epoch"))?;
    let mut peak = document.validate(limits)?;
    let mut count = 0usize;
    let mut outputs = 0usize;
    for recipe in document.history.recipes.values() {
        if recipe.state == EvaluationState::Detached {
            continue;
        }
        let length = if !registered(recipe, available) {
            missing_reason_len(recipe)?
        } else if !recipe.inputs.is_empty() {
            DEPENDENCY_REASON.len()
        } else {
            continue;
        };
        if length > limits.max_name_bytes {
            return Err(Error::Budget("name bytes"));
        }
        count = count
            .checked_add(1)
            .ok_or(Error::Overflow("restore diagnostics"))?;
        outputs = outputs
            .checked_add(recipe.outputs.len())
            .ok_or(Error::Overflow("restore outputs"))?;
        let bytes = (length as u64)
            .checked_mul(2)
            .ok_or(Error::Overflow("restore diagnostic copies"))?;
        // Both the recipe state and the caller's diagnostics own the message.
        // The existing state remains charged until replacement drops it.
        peak.add(
            Usage {
                bytes,
                ..Usage::default()
            },
            limits,
        )?;
    }
    peak.add(Usage::array::<String>(count)?, limits)?;
    peak.add(Usage::btree::<ObjectId, ()>(outputs)?, limits)?;
    peak.add(document.history.execution_scratch_usage()?, limits)?;
    peak.add(
        Usage::array::<HistoryId>(document.history.recipes.len())?,
        limits,
    )?;
    peak.add(
        Usage::array::<SourceStamp>(document.entities.len())?,
        limits,
    )?;
    Ok(count)
}
fn message(recipe: &Recipe) -> Result<String> {
    let mut reason = String::new();
    reason
        .try_reserve_exact(missing_reason_len(recipe)?)
        .map_err(|_| Error::Budget("restore diagnostic allocation"))?;
    write!(
        reason,
        "Missing replay module {}@{} for {} version {}",
        recipe.environment.solver,
        recipe.environment.solver_version,
        recipe.command,
        recipe.environment.command_version
    )
    .map_err(|_| Error::Invalid("restore diagnostic formatting"))?;
    Ok(reason)
}
fn copy_message(message: &str) -> Result<String> {
    let mut copy = String::new();
    copy.try_reserve_exact(message.len())
        .map_err(|_| Error::Budget("restore diagnostic copy"))?;
    copy.push_str(message);
    Ok(copy)
}
pub fn restore(
    mut document: Document,
    limits: &Limits,
    available: &BTreeSet<ReplayKey>,
) -> Result<RestoredDocument> {
    let count = admit(&document, limits, available)?;
    document.renew_session(limits)?;
    let mut diagnostics = Vec::new();
    diagnostics
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("restore diagnostic vector"))?;
    let mut blocked = BTreeSet::new();
    let order = document
        .history
        .execution_order_bounded(limits, document.validate(limits)?)?;
    for id in &order {
        let recipe = document
            .history
            .recipes
            .get_mut(id)
            .ok_or(Error::Missing("restore recipe"))?;
        if recipe.state == EvaluationState::Detached {
            continue;
        }
        let reason = if !registered(recipe, available) {
            Some(message(recipe)?)
        } else if recipe
            .inputs
            .iter()
            .any(|input| blocked.contains(&input.source.object))
        {
            Some(copy_message(DEPENDENCY_REASON)?)
        } else {
            None
        };
        if let Some(reason) = reason {
            recipe.state = EvaluationState::Unresolved(copy_message(&reason)?);
            diagnostics.push(reason);
            for id in &recipe.outputs {
                blocked.insert(*id);
                if let Some(result) = document.results.get_mut(id) {
                    result.status = ResultStatus::Stale;
                }
            }
        }
    }
    document.validate(limits)?;
    Ok(RestoredDocument {
        document,
        diagnostics,
    })
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use om9_core::{
        geometry::{Geometry, Transform, Unit},
        history::{ReplayEnvironment, Tolerance},
        model::Entity,
    };
    #[test]
    fn two_diagnostic_owners_are_reserved_before_restoration_grows_them() {
        let mut d = Document::new("restoration admission");
        let output = Entity::new("output", Geometry::Point([1., 2., 3.]), None);
        let id = output.id;
        d.entities.insert(id, output);
        let recipe = Recipe::new(
            d.id,
            "c".repeat(4_000),
            vec![],
            vec![id],
            ReplayEnvironment {
                command_version: 1,
                solver: "s".repeat(4_000),
                solver_version: "v".repeat(4_000),
                schema_version: 1,
                length_unit: Unit::Millimeter,
                frame: Transform::identity(),
                tolerance: Tolerance {
                    absolute_mm: 0.001,
                    relative: 1e-6,
                    angular_rad: 1e-5,
                },
            },
        );
        d.history.recipes.insert(recipe.id, recipe);
        let owned = d.validate(&Limits::default()).unwrap();
        let limits = Limits {
            max_bytes: owned.bytes + 18_000,
            ..Limits::default()
        };
        d.validate(&limits)
            .expect("original document fits without diagnostic copies");
        assert!(matches!(
            admit(&d, &limits, &BTreeSet::new()),
            Err(Error::Budget(_))
        ));
        assert!(
            d.history
                .recipes
                .values()
                .all(|r| r.state == EvaluationState::Clean)
        );
    }
}
