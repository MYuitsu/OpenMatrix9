//! Explicit native-only compatibility for legacy leaf names containing paths.
//! Canonical snapshots/JSON remain strict: this is never an implicit repair.
use crate::layer_state::*;
use std::collections::{HashMap, HashSet};

pub fn normalize_native_layers(mut state: LayerSnapshotV1) -> Result<LayerSnapshotV1, LayerError> {
    let mut charge = state.metadata_bytes()?;
    if validate_layers(&state).is_ok() {
        return Ok(state);
    }
    let mut ids = HashMap::new();
    for (index, row) in state.layers.iter().enumerate() {
        if ids.insert(row.source_id.clone(), index).is_some() {
            return Err(LayerError::DuplicateId);
        }
    }
    let mut paths = HashMap::new();
    // Establish the actual native ancestry before interpreting embedded separators.
    for (index, row) in state.layers.iter().enumerate() {
        let mut chain = Vec::new();
        let mut current = Some(index);
        while let Some(at) = current {
            if chain.contains(&at) {
                return Err(LayerError::Cycle);
            }
            if chain.len() >= MAX_DEPTH {
                return Err(LayerError::LimitExceeded);
            }
            chain.push(at);
            current = state.layers[at]
                .parent_id
                .as_ref()
                .map(|id| ids.get(id).copied().ok_or(LayerError::MissingLayer))
                .transpose()?;
        }
        let expected: Vec<String> = chain
            .iter()
            .rev()
            .flat_map(|&at| state.layers[at].name.split("::").map(String::from))
            .collect();
        if expected != row.path_components || expected.is_empty() {
            return Err(LayerError::InvalidPath);
        }
        if expected.len() > MAX_DEPTH {
            return Err(LayerError::LimitExceeded);
        }
        if expected
            .iter()
            .any(|name| name.is_empty() || name.len() > 1024 || name.chars().any(char::is_control))
        {
            return Err(LayerError::InvalidPath);
        }
        if paths.insert(path_key(&expected), index).is_some() {
            return Err(LayerError::AmbiguousPath);
        }
    }
    let original = state.layers.len();
    let mut reserved: HashSet<String> = ids.into_keys().collect();
    let mut serial = 0usize;
    for index in 0..original {
        let full = state.layers[index].path_components.clone();
        for depth in 1..full.len() {
            let components = full[..depth].to_vec();
            let key = path_key(&components);
            if paths.contains_key(&key) {
                continue;
            }
            if state.layers.len() >= MAX_LAYERS {
                return Err(LayerError::LimitExceeded);
            }
            let identity = loop {
                let value = format!("OM9-legacy-layer-{serial}");
                serial = serial.checked_add(1).ok_or(LayerError::LimitExceeded)?;
                if reserved.insert(value.clone()) {
                    break value;
                }
            };
            let row = LayerRow {
                source_id: identity,
                parent_id: None,
                name: components.last().unwrap().clone(),
                path_components: components,
                rgb: [180, 180, 180],
                locked: false,
                visible: true,
                persistent_locked: None,
                persistent_visible: None,
            };
            charge = charge
                .checked_add(
                    std::mem::size_of::<LayerRow>()
                        + row.source_id.len()
                        + row.name.len()
                        + row
                            .path_components
                            .iter()
                            .map(|s| std::mem::size_of::<String>() + s.len())
                            .sum::<usize>(),
                )
                .ok_or(LayerError::LimitExceeded)?;
            if charge > MAX_METADATA_BYTES {
                return Err(LayerError::LimitExceeded);
            }
            paths.insert(key, state.layers.len());
            state.layers.push(row);
        }
    }
    let mut order: Vec<usize> = (0..state.layers.len()).collect();
    order.sort_by_key(|&at| state.layers[at].path_components.len());
    for index in order {
        let components = state.layers[index].path_components.clone();
        let name = components.last().unwrap().clone();
        let (parent, path) = if components.len() == 1 {
            (None, vec![name.clone()])
        } else {
            let parent = paths[&path_key(&components[..components.len() - 1])];
            let mut path = state.layers[parent].path_components.clone();
            path.push(name.clone());
            (Some(state.layers[parent].source_id.clone()), path)
        };
        let row = &mut state.layers[index];
        row.name = name;
        row.parent_id = parent;
        row.path_components = path;
    }
    validate_layers(&state)?;
    Ok(state)
}
