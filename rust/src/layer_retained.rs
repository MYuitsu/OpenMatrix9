//! Geometry-free metadata overlay for an existing detached native archive.
//! Native dependency/instance geometry remains outside this portable snapshot.
use crate::{
    layer_clipboard::{GeometryObjectBinding, bind_geometry_objects},
    layer_exchange::{TransferScope, plan_receive},
    layer_state::{LayerError, LayerSnapshotV1, validate_layers},
};
/// Partition only portable object metadata; every namespace keeps the palette.
/// Original IDs, own state and origin stay unchanged (this is not receive/append).
pub fn subset_retained_snapshot(
    source: &LayerSnapshotV1,
    ids: &[String],
) -> Result<LayerSnapshotV1, LayerError> {
    use std::collections::HashSet;
    validate_layers(source)?;
    if ids.len() > crate::layer_state::MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    let wanted: HashSet<&str> = ids.iter().map(String::as_str).collect();
    if wanted.len() != ids.len() {
        return Err(LayerError::DuplicateId);
    }
    let objects: Vec<_> = source
        .objects
        .iter()
        .filter(|row| wanted.contains(row.id.as_str()))
        .cloned()
        .collect();
    if objects.len() != wanted.len() {
        return Err(LayerError::MissingObject);
    }
    let result = LayerSnapshotV1 {
        document_id: source.document_id.clone(),
        generation: source.generation,
        layers: source.layers.clone(),
        objects,
        active_layer: source.active_layer.clone(),
    };
    validate_layers(&result)?;
    Ok(result)
}

pub fn plan_retained_overlay(
    source: &LayerSnapshotV1,
    destination_palette: &LayerSnapshotV1,
    bindings: &[GeometryObjectBinding<'_>],
) -> Result<LayerSnapshotV1, LayerError> {
    // Untouched native objects (including definition members using ByParent)
    // are not imported/appended into the portable metadata result.
    if !destination_palette.objects.is_empty() {
        return Err(LayerError::InvalidSnapshot);
    }
    let bound = bind_geometry_objects(source, bindings)?;
    let plan = plan_receive(
        source,
        destination_palette,
        TransferScope::Selected(Vec::new()),
    )?;
    let mut result = plan.after;
    result.objects = bound.objects;
    for object in &mut result.objects {
        object.layer_id = plan
            .layer_mapping
            .get(&object.layer_id)
            .ok_or(LayerError::MissingLayer)?
            .clone();
    }
    result.document_id = source.document_id.clone();
    result.generation = source.generation;
    validate_layers(&result)?;
    Ok(result)
}
