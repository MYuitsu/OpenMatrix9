// SPDX-License-Identifier: LGPL-2.1-or-later
//! Associative dependency maintenance over `HistoryGraph` (M04 `OM9-L2-01-04`, wave W7).
//!
//! Ported from the legacy bridge (`rust/src/{builder_history,history,prepared_result,
//! reference_path}.rs`). The legacy prepared-result cache and the Qt builder sessions stay
//! behind; what is portable — deciding which recipes went stale, propagating dirtiness
//! through the output→input edges, planning the recompute in topological order, and the
//! detach/freeze lifecycle — is re-implemented on the target's own graph.
//!
//! State labels are part of the contract, so a caller can show why a feature is dirty:
//! `Dirty("source changed")`, `Dirty("downstream of dirty recipe")`,
//! `Unresolved("source missing")`, `Unresolved("source changed")`.
use crate::{
    Error, Result,
    history::{EvaluationState, HistoryGraph},
    identity::{DocumentId, HistoryId},
    model::Document,
};
use std::collections::{BTreeMap, BTreeSet};

/// A recipe that needs recompute, with the reason its state was set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleRecipe {
    pub id: HistoryId,
    pub reason: String,
}

/// Recipes whose recorded inputs no longer match the document: a changed source, a deleted
/// source, or a state the graph already records. Detached recipes are never stale.
pub fn stale_recipes(document: &Document, graph: &HistoryGraph) -> Vec<StaleRecipe> {
    let mut stale = Vec::new();
    for recipe in graph.recipes.values() {
        if recipe.state == EvaluationState::Detached {
            continue;
        }
        let mut reason: Option<String> = None;
        for input in &recipe.inputs {
            match document.resolve(&input.source) {
                Ok(_) => {}
                // A missing object and a stale stamp are different answers: one means the
                // source is gone, the other means it changed under the recipe.
                Err(Error::Missing(_)) => {
                    reason = Some("source missing".into());
                    break;
                }
                Err(Error::Stale(_)) => {
                    reason = Some("source changed".into());
                    break;
                }
                Err(_) => {
                    reason = Some("source unreadable".into());
                    break;
                }
            }
        }
        if let Some(reason) = reason {
            stale.push(StaleRecipe {
                id: recipe.id,
                reason,
            });
        }
    }
    stale.sort_by_key(|stale| stale.id);
    stale
}

/// Record the staleness in the graph: stale recipes become `Dirty`/`Unresolved`, then every
/// recipe that consumes one of their outputs becomes `Dirty("downstream of dirty recipe")` in
/// turn. Returns the ids whose state changed, in topological order.
pub fn mark_stale(document: &Document, graph: &mut HistoryGraph) -> Result<Vec<HistoryId>> {
    let stale = stale_recipes(document, graph);
    if stale.is_empty() {
        return Ok(Vec::new());
    }
    let own_stale: BTreeSet<HistoryId> = stale.iter().map(|entry| entry.id).collect();
    let mut changed = BTreeSet::new();
    for entry in &stale {
        let target = if entry.reason == "source missing" {
            EvaluationState::Unresolved(entry.reason.clone())
        } else {
            EvaluationState::Dirty(entry.reason.clone())
        };
        if let Some(recipe) = graph.recipes.get_mut(&entry.id)
            && recipe.state != target
        {
            // Only a real transition counts as a change, so a second pass over an unchanged
            // document reports nothing.
            recipe.state = target;
            changed.insert(entry.id);
        }
    }
    // Propagate through the output → input edges until nothing new is reached.
    loop {
        let dirty_outputs: BTreeSet<_> = graph
            .recipes
            .values()
            .filter(|recipe| {
                changed.contains(&recipe.id)
                    && matches!(
                        recipe.state,
                        EvaluationState::Dirty(_) | EvaluationState::Unresolved(_)
                    )
            })
            .flat_map(|recipe| recipe.outputs.iter().copied())
            .collect();
        let mut progressed = false;
        for recipe in graph.recipes.values_mut() {
            if changed.contains(&recipe.id)
                || recipe.state == EvaluationState::Detached
                // A recipe whose own source is missing keeps its `Unresolved` answer.
                || own_stale.contains(&recipe.id)
            {
                continue;
            }
            if recipe
                .inputs
                .iter()
                .any(|input| dirty_outputs.contains(&input.source.object))
                && recipe.state != EvaluationState::Dirty("downstream of dirty recipe".into())
            {
                recipe.state = EvaluationState::Dirty("downstream of dirty recipe".into());
                changed.insert(recipe.id);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    // Report in execution order so a caller can recompute without sorting again.
    let order = graph.execution_order()?;
    Ok(order
        .into_iter()
        .filter(|id| changed.contains(id))
        .collect())
}

/// The recompute plan: every recipe the graph marks `Dirty` or `Pending`, in topological
/// order. `Unresolved` and `Failed` recipes are not planned — they cannot run until the
/// caller changes their inputs or acknowledges the failure.
pub fn plan_recompute(graph: &HistoryGraph) -> Result<Vec<HistoryId>> {
    let order = graph.execution_order()?;
    Ok(order
        .into_iter()
        .filter(|id| {
            graph.recipes.get(id).is_some_and(|recipe| {
                matches!(
                    recipe.state,
                    EvaluationState::Dirty(_) | EvaluationState::Pending
                )
            })
        })
        .collect())
}

/// Record a successful recompute: the recipe is `Clean` again and its revision advances.
pub fn mark_clean(graph: &mut HistoryGraph, id: HistoryId) -> Result<()> {
    let recipe = graph
        .recipes
        .get_mut(&id)
        .ok_or(Error::Missing("history recipe"))?;
    if recipe.state == EvaluationState::Detached {
        return Err(Error::Invalid("detached recipe recompute"));
    }
    recipe.state = EvaluationState::Clean;
    recipe.revision = recipe
        .revision
        .checked_add(1)
        .ok_or(Error::Overflow("recipe revision"))?;
    Ok(())
}

/// Record a failed recompute: the recipe keeps its failure message instead of pretending to
/// be clean (the rollback half of `L3-007` is the transaction's job).
pub fn mark_failed(
    graph: &mut HistoryGraph,
    id: HistoryId,
    reason: impl Into<String>,
) -> Result<()> {
    let recipe = graph
        .recipes
        .get_mut(&id)
        .ok_or(Error::Missing("history recipe"))?;
    let reason = reason.into();
    if reason.is_empty() {
        return Err(Error::Invalid("history failure reason"));
    }
    recipe.state = EvaluationState::Failed(reason);
    Ok(())
}

/// Detach a recipe: its outputs stay in the document as plain objects, and the graph no
/// longer owns or updates them (`L3-008` policy for "keep the result, drop the history").
pub fn detach(graph: &mut HistoryGraph, id: HistoryId) -> Result<()> {
    let recipe = graph
        .recipes
        .get_mut(&id)
        .ok_or(Error::Missing("history recipe"))?;
    if recipe.state == EvaluationState::Detached {
        return Ok(());
    }
    recipe.state = EvaluationState::Detached;
    recipe.policy.record = false;
    recipe.policy.update = false;
    Ok(())
}

/// Freeze or thaw a recipe. A frozen recipe keeps its outputs but refuses recompute, which is
/// what `HistoryPolicy::lock` means for the graph.
pub fn freeze(graph: &mut HistoryGraph, id: HistoryId, frozen: bool) -> Result<()> {
    let recipe = graph
        .recipes
        .get_mut(&id)
        .ok_or(Error::Missing("history recipe"))?;
    if recipe.state == EvaluationState::Detached && frozen {
        return Err(Error::Invalid("detached recipe freeze"));
    }
    recipe.policy.lock = frozen;
    Ok(())
}

/// Count the recipes by state, for a history panel that must not re-derive the categories.
pub fn state_counts(graph: &HistoryGraph) -> BTreeMap<&'static str, usize> {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for recipe in graph.recipes.values() {
        let key = match &recipe.state {
            EvaluationState::Clean => "clean",
            EvaluationState::Dirty(_) => "dirty",
            EvaluationState::Pending => "pending",
            EvaluationState::Unresolved(_) => "unresolved",
            EvaluationState::Failed(_) => "failed",
            EvaluationState::Detached => "detached",
        };
        *counts.entry(key).or_default() += 1;
    }
    counts
}

/// A recipe that lives in a document that is not the one the caller holds is a programming
/// error, not a user-visible state.
pub fn ensure_document(graph: &HistoryGraph, document: DocumentId) -> Result<()> {
    if graph
        .recipes
        .values()
        .any(|recipe| recipe.document != document)
    {
        return Err(Error::Invalid("recipe document mismatch"));
    }
    Ok(())
}
