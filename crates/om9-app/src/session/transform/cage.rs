// SPDX-License-Identifier: LGPL-2.1-or-later
//! Cage rows: Create Cage (`OM9-TRANSFORM-041`), Cage Edit (`OM9-TRANSFORM-020`) and
//! ReleaseFromCage (`OM9-TRANSFORM-021`). The mathematical owner is `om9_core::cage`
//! (OM9-L3-04-08-005).
//!
//! A cage is an ordinary document object: a mesh box whose eight vertices are the cage corners,
//! plus one semantic extension that keeps the corners the cage was created with. A caged object
//! carries a back pointer to its cage, so membership has exactly one owner, and every command is
//! one transaction with one Undo entry.
use super::*;
use om9_core::budget::Limits;
use om9_core::cage as cage_kernel;
use om9_core::model::{Entity, Extension};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Matrix `Transform > Create Cage` (OM9-TRANSFORM-041).
pub const CAGE: &str = "OM9-TRANSFORM-041";
/// Matrix `Transform > Cage Edit` (OM9-TRANSFORM-020).
pub const CAGE_EDIT: &str = "OM9-TRANSFORM-020";
/// Matrix `Transform > ReleaseFromCage` (OM9-TRANSFORM-021).
pub const RELEASE_CAGE: &str = "OM9-TRANSFORM-022";

/// The one extension slot these rows own on an entity.
const CAGE_SLOT: &str = om9_core::model::CAGE_EXTENSION_SLOT;
const CAGE_LABEL: &str = "Cage";

/// A row that reads or writes the cage extension.
pub fn handles(feature: &str) -> bool {
    matches!(feature, CAGE | CAGE_EDIT | RELEASE_CAGE)
}

/// The documented input sequence: only Cage Edit asks, because a cage edit is one corner and one
/// offset and the rest of the cage is already document state.
pub fn fields(feature: &str) -> &'static [Field] {
    const EDIT_FIELDS: &[Field] = &[
        Field::Number("Cage corner to move (1-8): "),
        Field::Point("Corner offset: "),
    ];
    const NO_FIELDS: &[Field] = &[];
    match feature {
        CAGE_EDIT => EDIT_FIELDS,
        _ => NO_FIELDS,
    }
}

/// Validate one typed number of a cage row; `None` when the row is not ours.
pub(super) fn check_number(feature: &str, index: usize, value: f64) -> Option<Result<()>> {
    match (feature, index) {
        (CAGE_EDIT, 0) => Some(corner(value).map(|_| ())),
        _ => None,
    }
}

fn corner(value: f64) -> Result<usize> {
    if !value.is_finite() || value.fract() != 0.0 || !(1.0..=8.0).contains(&value) {
        return Err(Error::Invalid("cage corner between 1 and 8"));
    }
    Ok(value as usize)
}

fn rest_extension(corners: [Point3; 8]) -> Result<Extension> {
    let payload = serde_json::json!({ "rest": corners });
    Extension::new(
        1,
        serde_json::to_string(&payload).map_err(|_| Error::Invalid("cage rest"))?,
    )
}

fn link_extension(cage: ObjectId) -> Result<Extension> {
    let payload = serde_json::json!({ "cage": cage.0.to_string() });
    Extension::new(
        1,
        serde_json::to_string(&payload).map_err(|_| Error::Invalid("cage link"))?,
    )
}

fn slot(entity: &Entity) -> Option<serde_json::Value> {
    let extension = entity.extensions.get(CAGE_SLOT)?;
    serde_json::from_str(&extension.payload).ok()
}

/// The corners the cage was created with, when `entity` is a cage.
fn rest_of(entity: &Entity) -> Result<Option<[Point3; 8]>> {
    let Some(value) = slot(entity) else {
        return Ok(None);
    };
    let Some(rest) = value.get("rest") else {
        return Ok(None);
    };
    let corners: Vec<Point3> =
        serde_json::from_value(rest.clone()).map_err(|_| Error::Invalid("cage rest payload"))?;
    if corners.len() != 8 {
        return Err(Error::Invalid("cage rest payload"));
    }
    let mut found = [[0.0; 3]; 8];
    found.copy_from_slice(&corners);
    Ok(Some(found))
}

fn is_cage(entity: &Entity) -> bool {
    rest_of(entity).ok().flatten().is_some()
}

/// The cage an object is attached to, when `entity` carries the back pointer.
fn link_of(entity: &Entity) -> Option<ObjectId> {
    let value = slot(entity)?;
    let text = value.get("cage")?.as_str()?;
    Uuid::parse_str(text).ok().map(ObjectId)
}

fn detached(entity: &Entity) -> BTreeMap<String, Extension> {
    let mut extensions = entity.extensions.clone();
    extensions.remove(CAGE_SLOT);
    extensions
}

fn selection(controller: &Controller) -> Vec<ObjectId> {
    let mut ids: Vec<ObjectId> = controller.selection.iter().copied().collect();
    ids.sort();
    ids
}

fn entity_of(controller: &Controller, id: ObjectId) -> Result<&Entity> {
    controller
        .document()
        .entities
        .get(&id)
        .ok_or(Error::Missing("cage object"))
}

/// The world-space corners of the cage mesh, so a moved cage object still presents its corners.
fn world_corners(controller: &Controller, id: ObjectId) -> Result<[Point3; 8]> {
    let entity = entity_of(controller, id)?;
    let Geometry::Mesh(mesh) = &entity.geometry else {
        return Err(Error::Invalid("cage mesh"));
    };
    let local = cage_kernel::mesh_corners(mesh)?;
    let placement = entity.placement;
    let mut corners = [[0.0; 3]; 8];
    for (slot, corner) in corners.iter_mut().zip(local) {
        *slot = placement.point(corner)?;
    }
    Ok(corners)
}

fn attached(controller: &Controller, cage: ObjectId) -> Result<Vec<ObjectId>> {
    let mut found = Vec::new();
    for (id, entity) in &controller.document().entities {
        if link_of(entity) == Some(cage) {
            found.push(*id);
        }
    }
    Ok(found)
}

/// The cage a Cage Edit works on: a selected cage, or the single cage the selection is attached
/// to. Two cages in one selection is ambiguous and refused.
fn resolve_cage(controller: &Controller) -> Result<(ObjectId, [Point3; 8])> {
    let ids = selection(controller);
    if ids.is_empty() {
        return Err(Error::Missing(
            "select a cage or the objects attached to one",
        ));
    }
    let mut cages = Vec::new();
    let mut links = Vec::new();
    for id in &ids {
        let entity = entity_of(controller, *id)?;
        if is_cage(entity) {
            cages.push(*id);
        } else if let Some(link) = link_of(entity) {
            links.push(link);
        }
    }
    if cages.len() > 1 {
        return Err(Error::Invalid("one cage at a time"));
    }
    links.sort();
    links.dedup();
    let cage = match cages.first() {
        Some(id) => *id,
        None => match links.len() {
            0 => {
                return Err(Error::Missing(
                    "select a cage or the objects attached to one",
                ));
            }
            1 => links[0],
            _ => return Err(Error::Invalid("the selection spans more than one cage")),
        },
    };
    Ok((cage, world_corners(controller, cage)?))
}

/// The union of the world-space bounds of the selection, refusing anything without points.
fn selection_bounds(
    controller: &Controller,
    ids: &[ObjectId],
    limits: &Limits,
) -> Result<om9_core::utilities::Bounds> {
    let mut result: Option<om9_core::utilities::Bounds> = None;
    for id in ids {
        let entity = entity_of(controller, *id)?;
        let world = om9_core::top11::bake_geometry(&entity.geometry, entity.placement, limits)?;
        let bounds =
            om9_core::utilities::bounds(&world).ok_or(Error::Missing("cage object bounds"))?;
        result = Some(match result {
            Some(mut current) => {
                for axis in 0..3 {
                    current.min[axis] = current.min[axis].min(bounds.min[axis]);
                    current.max[axis] = current.max[axis].max(bounds.max[axis]);
                }
                current
            }
            None => bounds,
        });
    }
    result.ok_or(Error::Missing("objects to cage"))
}

/// Refuse a cage run before its first question is asked.
pub(super) fn validate(controller: &Controller, feature: &str) -> Result<()> {
    match feature {
        CAGE => {
            let targets = controller.transform_targets()?;
            if targets.is_empty() {
                return Err(Error::Missing("objects to cage"));
            }
            for id in targets {
                if link_of(entity_of(controller, id)?).is_some() {
                    return Err(Error::Invalid("object is already attached to a cage"));
                }
            }
            Ok(())
        }
        CAGE_EDIT => resolve_cage(controller).map(|_| ()),
        RELEASE_CAGE => {
            if selection(controller).is_empty() {
                return Err(Error::Missing("objects attached to a cage"));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// `Create Cage`: one mesh box around the selection plus one back pointer per caged object.
pub(super) fn create(controller: &mut Controller, name: &'static str) -> Result<()> {
    let limits = *controller.coordinator.limits();
    let targets = controller.transform_targets()?;
    if targets.is_empty() {
        return Err(Error::Missing("objects to cage"));
    }
    let bounds = selection_bounds(controller, &targets, &limits)?;
    if bounds.size().iter().any(|value| *value <= 0.0) {
        return Err(Error::Invalid("cage needs a box with volume"));
    }
    let corners = cage_kernel::box_corners(bounds.min, bounds.max);
    let layer = controller.document().current_layer;
    let mut entity = Entity::new(
        CAGE_LABEL,
        Geometry::Mesh(cage_kernel::box_mesh(corners)),
        layer,
    );
    entity
        .extensions
        .insert(CAGE_SLOT.to_string(), rest_extension(corners)?);
    let cage = entity.id;
    let document = controller.document().id;
    let mut mutations = Vec::with_capacity(targets.len() + 1);
    mutations.push(Mutation::Insert { document, entity });
    for id in targets {
        let entity = entity_of(controller, id)?;
        if link_of(entity).is_some() {
            return Err(Error::Invalid("object is already attached to a cage"));
        }
        let mut extensions = entity.extensions.clone();
        extensions.insert(CAGE_SLOT.to_string(), link_extension(cage)?);
        mutations.push(Mutation::Extensions {
            source: controller.document().stamp(id)?,
            extensions,
        });
    }
    controller.commit_utility_mutations(name, mutations)
}

/// `Cage Edit`: move one corner and carry the attached objects with the cage's own field.
pub(super) fn edit(
    controller: &mut Controller,
    name: &'static str,
    corner: usize,
    offset: Point3,
) -> Result<()> {
    let limits = *controller.coordinator.limits();
    let (cage, live) = resolve_cage(controller)?;
    let before = cage_kernel::cage_from_corners(live, &limits)?;
    let after = before.with_corner(corner, offset)?;
    let targets = attached(controller, cage)?;
    let mut mutations = Vec::with_capacity(targets.len() + 1);
    for id in targets {
        let (geometry, placement, source) = {
            let entity = entity_of(controller, id)?;
            (
                entity.geometry.clone(),
                entity.placement,
                controller.document().stamp(id)?,
            )
        };
        let moved =
            om9_core::deform::apply_placed_map(&geometry, &placement, &limits, &mut |point| {
                cage_kernel::displacement(&before, &after, point, &limits)
            })?;
        mutations.push(Mutation::Geometry {
            source,
            geometry: moved,
        });
    }
    let (placement, source) = {
        let entity = entity_of(controller, cage)?;
        (entity.placement, controller.document().stamp(cage)?)
    };
    let inverse = om9_core::top11::inverse_affine(placement)?;
    let mut local = [[0.0; 3]; 8];
    for (slot, value) in local.iter_mut().zip(after.corners()?) {
        *slot = inverse.point(value)?;
    }
    mutations.push(Mutation::Geometry {
        source,
        geometry: Geometry::Mesh(cage_kernel::box_mesh(local)),
    });
    controller.commit_utility_mutations(name, mutations)
}

/// `ReleaseFromCage`: drop the back pointer from the selection, or from every object attached to
/// a selected cage. The geometry keeps its current shape.
pub(super) fn release(controller: &mut Controller, name: &'static str) -> Result<()> {
    let ids = selection(controller);
    let mut seen: BTreeSet<ObjectId> = BTreeSet::new();
    let mut mutations = Vec::new();
    for id in ids {
        if is_cage(entity_of(controller, id)?) {
            for target in attached(controller, id)? {
                if !seen.insert(target) {
                    continue;
                }
                let entity = entity_of(controller, target)?;
                mutations.push(Mutation::Extensions {
                    source: controller.document().stamp(target)?,
                    extensions: detached(entity),
                });
            }
        } else if link_of(entity_of(controller, id)?).is_some() && seen.insert(id) {
            let entity = entity_of(controller, id)?;
            mutations.push(Mutation::Extensions {
                source: controller.document().stamp(id)?,
                extensions: detached(entity),
            });
        }
    }
    if mutations.is_empty() {
        return Err(Error::Invalid("selection is not attached to a cage"));
    }
    controller.commit_utility_mutations(name, mutations)
}
