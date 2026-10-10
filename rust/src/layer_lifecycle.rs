//! Geometry-free document observation and conservative legacy migration.
use crate::{layer_document::default_document, layer_exchange::LayerApplyPlan, layer_state::*};
use std::collections::{BTreeMap, HashSet};

/// Native adapters provide IDs only after verifying persisted helper identity
/// and absence of model payload. This is semantic inventory cleanup, not model
/// deletion: no geometry guard is bypassed for any unverified object. Keep the
/// complete palette, active layer, generation and all other object rows exact.
pub fn filter_verified_metadata(
    state: &LayerSnapshotV1,
    verified_ids: &[ObjectId],
) -> Result<LayerSnapshotV1, LayerError> {
    validate_layers(state)?;
    if verified_ids.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    let mut ids = HashSet::new();
    let mut charge = 0usize;
    for id in verified_ids {
        if !valid_identity(id) {
            return Err(LayerError::InvalidSnapshot);
        }
        charge = charge
            .checked_add(id.len() + std::mem::size_of::<ObjectId>())
            .filter(|n| *n <= MAX_METADATA_BYTES)
            .ok_or(LayerError::LimitExceeded)?;
        if !ids.insert(id) {
            return Err(LayerError::DuplicateId);
        }
    }
    let mut after = state.clone();
    after.objects.retain(|row| !ids.contains(&row.id));
    Ok(after)
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn expanded_parent_paths_are_budgeted_before_building_full_snapshot() {
        let facts = [LegacyObject {
            id: "old".into(),
            path: Some("a".repeat(500) + "::" + &"b".repeat(500)),
            rgb: [1, 2, 3],
            locked: false,
            visible: true,
        }];
        let base = default_document("doc").unwrap().metadata_bytes().unwrap();
        assert_eq!(
            migrate_legacy_with_budget("doc", &facts, base + 1500),
            Err(LayerError::LimitExceeded)
        );
        assert!(migrate_legacy_with_budget("doc", &facts, base + 4000).is_ok());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyObject {
    pub id: ObjectId,
    pub path: Option<String>,
    pub rgb: [u8; 3],
    pub locked: bool,
    pub visible: bool,
}
pub fn migrate_legacy(
    document: &str,
    objects: &[LegacyObject],
) -> Result<LayerSnapshotV1, LayerError> {
    migrate_legacy_with_budget(document, objects, MAX_METADATA_BYTES)
}
fn charge(total: &mut usize, amount: usize, budget: usize) -> Result<(), LayerError> {
    *total = total
        .checked_add(amount)
        .filter(|n| *n <= budget)
        .ok_or(LayerError::LimitExceeded)?;
    Ok(())
}
fn migrate_legacy_with_budget(
    document: &str,
    objects: &[LegacyObject],
    budget: usize,
) -> Result<LayerSnapshotV1, LayerError> {
    if objects.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    let mut state = default_document(document)?;
    append_legacy_objects(&mut state, objects, budget)?;
    validate_layers(&state)?;
    Ok(state)
}
fn append_legacy_objects(
    state: &mut LayerSnapshotV1,
    objects: &[LegacyObject],
    budget: usize,
) -> Result<(), LayerError> {
    if state
        .objects
        .len()
        .checked_add(objects.len())
        .is_none_or(|n| n > MAX_OBJECTS)
    {
        return Err(LayerError::LimitExceeded);
    }
    let mut total = state.metadata_bytes()?;
    charge(&mut total, 0, budget)?;
    let mut paths: BTreeMap<_, _> = state
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| (path_key(&l.path_components), i))
        .collect();
    let mut ids: HashSet<_> = state.objects.iter().map(|o| o.id.clone()).collect();
    let mut layer_ids: HashSet<_> = state.layers.iter().map(|l| l.source_id.clone()).collect();
    for object in objects {
        if !valid_identity(&object.id) {
            return Err(LayerError::InvalidSnapshot);
        }
        if !ids.insert(object.id.clone()) {
            return Err(LayerError::DuplicateId);
        }
        let layer_id = if let Some(path) = &object.path {
            if path.len() > MAX_DEPTH * 1026 {
                return Err(LayerError::LimitExceeded);
            }
            let parts: Vec<String> = path.split("::").map(str::to_owned).collect();
            if parts.len() > MAX_DEPTH || parts.iter().any(|p| !valid_component(p)) {
                return Err(LayerError::InvalidPath);
            }
            let mut parent = None;
            for depth in 1..=parts.len() {
                let components = &parts[..depth];
                let key = path_key(components);
                let position = if let Some(&position) = paths.get(&key) {
                    if state.layers[position].path_components != components {
                        return Err(LayerError::AmbiguousPath);
                    }
                    position
                } else {
                    if state.layers.len() >= MAX_LAYERS {
                        return Err(LayerError::LimitExceeded);
                    }
                    let position = state.layers.len();
                    let mut ordinal = position + 1;
                    let source_id = loop {
                        let candidate = format!("om9-legacy-layer-{ordinal}");
                        if layer_ids.insert(candidate.clone()) {
                            break candidate;
                        }
                        ordinal = ordinal.checked_add(1).ok_or(LayerError::LimitExceeded)?;
                    };
                    let path_bytes = components.iter().try_fold(0usize, |n, c| {
                        n.checked_add(c.len() + std::mem::size_of::<String>())
                            .ok_or(LayerError::LimitExceeded)
                    })?;
                    let cost = std::mem::size_of::<LayerRow>()
                        + source_id.len()
                        + parent.as_ref().map_or(0, String::len)
                        + parts[depth - 1].len()
                        + path_bytes;
                    charge(&mut total, cost, budget)?;
                    state.layers.push(LayerRow {
                        source_id,
                        parent_id: parent.clone(),
                        name: parts[depth - 1].clone(),
                        path_components: components.to_vec(),
                        rgb: object.rgb,
                        locked: false,
                        visible: true,
                        persistent_locked: None,
                        persistent_visible: None,
                    });
                    paths.insert(key, position);
                    position
                };
                parent = Some(state.layers[position].source_id.clone());
            }
            parent.ok_or(LayerError::InvalidPath)?
        } else {
            state.active_layer.clone().ok_or(LayerError::MissingLayer)?
        };
        // Legacy flat lock/color cannot prove inherited state. Keep the observed
        // own restriction and actual color; never invent a native provenance.
        charge(
            &mut total,
            std::mem::size_of::<ObjectLayerRow>() + object.id.len() + layer_id.len(),
            budget,
        )?;
        state.objects.push(ObjectLayerRow {
            id: object.id.clone(),
            layer_id,
            locked: object.locked,
            visible: object.visible,
            color_source: ColorSource::ByObject,
            rgb: object.rgb,
        });
    }
    Ok(())
}

/// Reconcile native objects whose creation/provenance was not observed by OM9.
/// Known canonical object rows and the complete custom palette always win over
/// flattened native projections. Unknown facts remain conservative ByObject.
pub fn plan_reconcile_legacy(
    state: &LayerSnapshotV1,
    facts: &[LegacyObject],
) -> Result<LayerApplyPlan, LayerError> {
    validate_layers(state)?;
    if facts.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    let mut live = HashSet::with_capacity(facts.len());
    for fact in facts {
        if !valid_identity(&fact.id) {
            return Err(LayerError::InvalidSnapshot);
        }
        if !live.insert(&fact.id) {
            return Err(LayerError::DuplicateId);
        }
    }
    let removed: Vec<_> = state
        .objects
        .iter()
        .filter(|o| !live.contains(&o.id))
        .map(|o| o.id.clone())
        .collect();
    if !removed.is_empty() {
        can_mutate(state, &removed, &LayerOperation::Delete)?;
    }
    let known: HashSet<_> = state.objects.iter().map(|o| &o.id).collect();
    let unknown: Vec<_> = facts
        .iter()
        .filter(|f| !known.contains(&f.id))
        .cloned()
        .collect();
    let mut after = state.clone();
    after.objects.retain(|o| live.contains(&o.id));
    append_legacy_objects(&mut after, &unknown, MAX_METADATA_BYTES)?;
    if &after != state {
        after.generation = state
            .generation
            .checked_add(1)
            .ok_or(LayerError::LimitExceeded)?;
    }
    validate_layers(&after)?;
    Ok(LayerApplyPlan {
        before: state.clone(),
        after,
        layer_mapping: BTreeMap::new(),
        object_mapping: BTreeMap::new(),
    })
}
pub fn plan_observed(
    state: &LayerSnapshotV1,
    live_ids: &[ObjectId],
) -> Result<LayerApplyPlan, LayerError> {
    let index = LayerIndex::new(state)?;
    if live_ids.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    let mut live = HashSet::with_capacity(live_ids.len());
    for id in live_ids {
        if !valid_identity(id) {
            return Err(LayerError::InvalidSnapshot);
        }
        if !live.insert(id) {
            return Err(LayerError::DuplicateId);
        }
    }
    let known: HashSet<_> = state.objects.iter().map(|o| &o.id).collect();
    let removed: Vec<_> = state
        .objects
        .iter()
        .filter(|o| !live.contains(&o.id))
        .map(|o| o.id.clone())
        .collect();
    if !removed.is_empty() {
        can_mutate(state, &removed, &LayerOperation::Delete)?;
    }
    let added: Vec<_> = live_ids.iter().filter(|id| !known.contains(id)).collect();
    let new_layer = if added.is_empty() {
        None
    } else {
        let id = state
            .active_layer
            .as_ref()
            .ok_or(LayerError::MissingLayer)?;
        let flags = index.layer_effective(id)?;
        if flags.locked {
            return Err(LayerError::Locked);
        }
        if !flags.visible {
            return Err(LayerError::Hidden);
        }
        Some((id.clone(), flags.rgb))
    };
    drop(index);
    let mut after = state.clone();
    after.objects.retain(|o| live.contains(&o.id));
    if let Some((layer_id, rgb)) = new_layer {
        for id in added {
            after.objects.push(ObjectLayerRow {
                id: id.clone(),
                layer_id: layer_id.clone(),
                locked: false,
                visible: true,
                color_source: ColorSource::ByLayer,
                rgb,
            });
        }
    }
    if &after != state {
        after.generation = state
            .generation
            .checked_add(1)
            .ok_or(LayerError::LimitExceeded)?;
    }
    validate_layers(&after)?;
    Ok(LayerApplyPlan {
        before: state.clone(),
        after,
        layer_mapping: BTreeMap::new(),
        object_mapping: BTreeMap::new(),
    })
}
