//! Pure receive plan: source palette wins, object transfers append, no host mutation.
use crate::layer_state::*;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferScope {
    Selected(Vec<ObjectId>),
    Session,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerApplyPlan {
    pub before: LayerSnapshotV1,
    pub after: LayerSnapshotV1,
    pub layer_mapping: BTreeMap<LayerId, LayerId>,
    pub object_mapping: BTreeMap<ObjectId, ObjectId>,
}
impl LayerApplyPlan {
    /// Equality also rejects changes from adapters that forgot to bump generation.
    pub fn check_destination(&self, current: &LayerSnapshotV1) -> Result<(), LayerError> {
        if current == &self.before {
            Ok(())
        } else {
            Err(LayerError::Stale)
        }
    }
}
fn fresh_id(
    used: &mut HashSet<String>,
    prefix: &str,
    next: &mut u64,
) -> Result<String, LayerError> {
    loop {
        *next = next.checked_add(1).ok_or(LayerError::LimitExceeded)?;
        let id = format!("{prefix}{next}");
        if used.insert(id.clone()) {
            return Ok(id);
        }
    }
}
pub(crate) fn choose_active(
    snapshot: &mut LayerSnapshotV1,
    source_active: Option<&LayerId>,
) -> Result<(), LayerError> {
    let index = LayerIndex::new(snapshot)?;
    let usable = |id: &str| index.layer_effective(id).is_ok_and(|s| s.selectable());
    let selected = source_active
        .filter(|id| usable(id))
        .cloned()
        .or_else(|| {
            snapshot
                .active_layer
                .as_ref()
                .filter(|id| usable(id))
                .cloned()
        })
        .or_else(|| {
            snapshot
                .layers
                .iter()
                .filter(|l| usable(&l.source_id))
                .min_by_key(|l| path_key(&l.path_components))
                .map(|l| l.source_id.clone())
        });
    drop(index);
    if let Some(id) = selected {
        snapshot.active_layer = Some(id);
        return Ok(());
    }
    let paths: HashSet<_> = snapshot
        .layers
        .iter()
        .map(|l| path_key(&l.path_components))
        .collect();
    let mut name = "OM9 Transfer Work".to_string();
    let mut suffix = 1_u64;
    while paths.contains(&path_key(&[name.clone()])) {
        suffix = suffix.checked_add(1).ok_or(LayerError::LimitExceeded)?;
        name = format!("OM9 Transfer Work {suffix}");
    }
    let mut used = snapshot
        .layers
        .iter()
        .map(|l| l.source_id.clone())
        .collect();
    let id = fresh_id(&mut used, "om9-layer-", &mut 0)?;
    snapshot.layers.push(LayerRow {
        source_id: id.clone(),
        parent_id: None,
        name: name.clone(),
        path_components: vec![name],
        rgb: [180, 180, 180],
        locked: false,
        visible: true,
        persistent_locked: None,
        persistent_visible: None,
    });
    snapshot.active_layer = Some(id);
    Ok(())
}
pub fn plan_receive(
    source: &LayerSnapshotV1,
    destination: &LayerSnapshotV1,
    scope: TransferScope,
) -> Result<LayerApplyPlan, LayerError> {
    let src_index = LayerIndex::new(source)?;
    validate_layers(destination)?;
    let selected = match scope {
        TransferScope::Session => source.objects.iter().map(|o| o.id.clone()).collect(),
        TransferScope::Selected(ids) => ids,
    };
    if selected.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    let mut seen = HashSet::with_capacity(selected.len());
    for id in &selected {
        src_index.object_index(id)?;
        if !seen.insert(id) {
            return Err(LayerError::DuplicateId);
        }
    }
    if destination
        .objects
        .len()
        .checked_add(selected.len())
        .is_none_or(|n| n > MAX_OBJECTS)
    {
        return Err(LayerError::LimitExceeded);
    }
    let generation = destination
        .generation
        .checked_add(1)
        .ok_or(LayerError::LimitExceeded)?;
    let mut after = destination.clone();
    after.generation = generation;
    let mut paths: HashMap<_, _> = after
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| (path_key(&l.path_components), i))
        .collect();
    let mut layer_ids: HashSet<String> = after.layers.iter().map(|l| l.source_id.clone()).collect();
    // Reserve all incoming IDs before allocating collision IDs.
    layer_ids.extend(source.layers.iter().map(|l| l.source_id.clone()));
    let mut sequence = 0;
    let mut layer_mapping = BTreeMap::new();
    let mut ordered: Vec<_> = source.layers.iter().collect();
    ordered.sort_by_key(|l| l.path_components.len());
    for source_layer in ordered {
        let key = path_key(&source_layer.path_components);
        let existing = paths.get(&key).copied();
        let id = if let Some(i) = existing {
            after.layers[i].source_id.clone()
        } else if after
            .layers
            .iter()
            .any(|l| l.source_id == source_layer.source_id)
        {
            fresh_id(&mut layer_ids, "om9-layer-", &mut sequence)?
        } else {
            source_layer.source_id.clone()
        };
        let mut row = source_layer.clone();
        row.source_id = id.clone();
        row.parent_id = source_layer
            .parent_id
            .as_ref()
            .map(|p| {
                layer_mapping
                    .get(p)
                    .cloned()
                    .ok_or(LayerError::MissingLayer)
            })
            .transpose()?;
        if let Some(i) = existing {
            after.layers[i] = row;
        } else {
            if after.layers.len() >= MAX_LAYERS {
                return Err(LayerError::LimitExceeded);
            }
            paths.insert(key, after.layers.len());
            after.layers.push(row);
        }
        layer_mapping.insert(source_layer.source_id.clone(), id);
    }
    // Keep destination-only child paths consistent with source-wins parent spelling.
    let names: HashMap<_, _> = after
        .layers
        .iter()
        .map(|l| (l.source_id.clone(), l.name.clone()))
        .collect();
    let parents: HashMap<_, _> = after
        .layers
        .iter()
        .map(|l| (l.source_id.clone(), l.parent_id.clone()))
        .collect();
    for l in &mut after.layers {
        let mut parts = vec![l.name.clone()];
        let mut parent = l.parent_id.clone();
        while let Some(id) = parent {
            if parts.len() >= MAX_DEPTH {
                return Err(LayerError::LimitExceeded);
            }
            parts.push(names.get(&id).ok_or(LayerError::MissingLayer)?.clone());
            parent = parents.get(&id).ok_or(LayerError::MissingLayer)?.clone();
        }
        parts.reverse();
        l.path_components = parts;
    }
    let mut object_ids: HashSet<String> = after.objects.iter().map(|o| o.id.clone()).collect();
    object_ids.extend(source.objects.iter().map(|o| o.id.clone()));
    let mut object_mapping = BTreeMap::new();
    let mut sequence = 0;
    for id in selected {
        let mut o = source.objects[src_index.object_index(&id)?].clone();
        o.layer_id = layer_mapping
            .get(&o.layer_id)
            .ok_or(LayerError::MissingLayer)?
            .clone();
        // Copy/Paste always adds. Source ID is provenance, never an update key.
        o.id = fresh_id(&mut object_ids, "om9-object-", &mut sequence)?;
        object_mapping.insert(id, o.id.clone());
        after.objects.push(o);
    }
    let source_active = source
        .active_layer
        .as_ref()
        .and_then(|id| layer_mapping.get(id));
    choose_active(&mut after, source_active)?;
    validate_layers(&after)?;
    Ok(LayerApplyPlan {
        before: destination.clone(),
        after,
        layer_mapping,
        object_mapping,
    })
}
