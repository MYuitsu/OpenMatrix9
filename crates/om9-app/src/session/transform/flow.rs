// SPDX-License-Identifier: LGPL-2.1-or-later
//! Smart Flow rows: `SmartFlow` (`OM9-TRANSFORM-011`), `SmartFlowRigid` (`OM9-TRANSFORM-012`),
//! `SmartFlowBreak` (`OM9-TRANSFORM-014`) and `SmartFlowPullDownCurves`
//! (`OM9-TRANSFORM-015`). The owner is `om9_core::deform` with the M08 module
//! `OM9-L2-04-05` (Smart Flow history).
//!
//! A Smart Flow command keeps a **flow link** on every object it touches: the base surface, the
//! target surface, whether the transport is rigid, its revision and its state
//! `ACTIVE`/`BROKEN`. Breaking keeps the geometry and stops the relation; running the row again
//! re-flows the object and bumps the revision.
use super::*;
use om9_core::deform;
use om9_core::geometry::NurbsSurface;
use om9_core::model::{Entity, Extension};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Matrix `Transform > Smart Flow` (OM9-TRANSFORM-011).
pub const SMART_FLOW: &str = "OM9-TRANSFORM-011";
/// Matrix `Transform > Smart Flow Rigid` (OM9-TRANSFORM-012).
pub const SMART_FLOW_RIGID: &str = "OM9-TRANSFORM-012";
/// Matrix `Transform > Smart Flow Break` (OM9-TRANSFORM-014).
pub const SMART_FLOW_BREAK: &str = "OM9-TRANSFORM-014";
/// Matrix `Transform > Smart Flow Pull Down Curves` (OM9-TRANSFORM-015).
pub const SMART_FLOW_PULLDOWN: &str = "OM9-TRANSFORM-015";

const FLOW_SLOT: &str = om9_core::model::FLOW_EXTENSION_SLOT;
const ACTIVE: &str = "ACTIVE";
const BROKEN: &str = "BROKEN";

/// A row that reads or writes the flow link.
pub fn handles(feature: &str) -> bool {
    matches!(
        feature,
        SMART_FLOW | SMART_FLOW_RIGID | SMART_FLOW_BREAK | SMART_FLOW_PULLDOWN
    )
}

/// The documented input sequence: the three flow rows name the base surface by a picked point,
/// because the selection is a set and has no pick order; Break asks nothing.
pub fn fields(feature: &str) -> &'static [Field] {
    const FRAME_FIELDS: &[Field] = &[Field::Point("Point on the base surface: ")];
    const NO_FIELDS: &[Field] = &[];
    match feature {
        SMART_FLOW | SMART_FLOW_RIGID | SMART_FLOW_PULLDOWN => FRAME_FIELDS,
        _ => NO_FIELDS,
    }
}

/// No Smart Flow row takes a typed number.
pub(super) fn check_number(_feature: &str, _index: usize, _value: f64) -> Option<Result<()>> {
    None
}

/// The selected surfaces (with their ids) and the objects to flow.
type Frames = (Vec<(ObjectId, NurbsSurface)>, Vec<ObjectId>);

/// The base frame, the target frame and the objects to flow.
type ResolvedFrames = (
    (ObjectId, NurbsSurface),
    (ObjectId, NurbsSurface),
    Vec<ObjectId>,
);

/// One recorded flow link.
struct Link {
    base: ObjectId,
    target: ObjectId,
    rigid: bool,
    revision: u64,
    state: String,
}

fn link_extension(link: &Link) -> Result<Extension> {
    let payload = serde_json::json!({
        "base": link.base.0.to_string(),
        "target": link.target.0.to_string(),
        "rigid": link.rigid,
        "revision": link.revision,
        "state": link.state,
    });
    Extension::new(
        1,
        serde_json::to_string(&payload).map_err(|_| Error::Invalid("flow link"))?,
    )
}

fn slot(entity: &Entity) -> Option<serde_json::Value> {
    let extension = entity.extensions.get(FLOW_SLOT)?;
    serde_json::from_str(&extension.payload).ok()
}

fn link_of(entity: &Entity) -> Option<Link> {
    let value = slot(entity)?;
    let base = Uuid::parse_str(value.get("base")?.as_str()?).ok()?;
    let target = Uuid::parse_str(value.get("target")?.as_str()?).ok()?;
    Some(Link {
        base: ObjectId(base),
        target: ObjectId(target),
        rigid: value.get("rigid").and_then(|value| value.as_bool())?,
        revision: value.get("revision").and_then(|value| value.as_u64())?,
        state: value.get("state")?.as_str()?.to_string(),
    })
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
        .ok_or(Error::Missing("flow object"))
}

/// The two selected surfaces and the objects to flow. Exactly two surfaces are the frames, in
/// the order the document stores them; the picked point decides which one is the base.
fn frames(controller: &Controller) -> Result<Frames> {
    let ids = selection(controller);
    if ids.is_empty() {
        return Err(Error::Missing("flow selection"));
    }
    let mut surfaces = Vec::new();
    let mut targets = Vec::new();
    for id in ids {
        let entity = entity_of(controller, id)?;
        match &entity.geometry {
            Geometry::Surface(surface) => surfaces.push((id, surface.clone())),
            _ => targets.push(id),
        }
    }
    if surfaces.len() != 2 {
        return Err(Error::Missing(
            "select the base surface, the target surface and the objects together",
        ));
    }
    if targets.is_empty() {
        return Err(Error::Missing(
            "select the objects to flow with the two surfaces",
        ));
    }
    Ok((surfaces, targets))
}

/// The frames with the base first: the surface nearest the picked point is the base.
fn resolve_frames(
    controller: &Controller,
    picked: Point3,
    limits: &Limits,
) -> Result<ResolvedFrames> {
    let (mut surfaces, targets) = frames(controller)?;
    let first =
        om9_core::curve_surface_ops::closest_point_on_surface(&surfaces[0].1, picked, limits)?
            .distance;
    let second =
        om9_core::curve_surface_ops::closest_point_on_surface(&surfaces[1].1, picked, limits)?
            .distance;
    if second < first {
        surfaces.swap(0, 1);
    }
    let mut surfaces = surfaces.into_iter();
    let base = surfaces.next().ok_or(Error::Missing("base surface"))?;
    let target = surfaces.next().ok_or(Error::Missing("target surface"))?;
    Ok((base, target, targets))
}

/// Refuse a Smart Flow run before its first question is asked.
pub(super) fn validate(controller: &Controller, feature: &str) -> Result<()> {
    match feature {
        SMART_FLOW | SMART_FLOW_RIGID | SMART_FLOW_PULLDOWN => {
            frames(controller)?;
            Ok(())
        }
        SMART_FLOW_BREAK => {
            if linked(controller).is_empty() {
                return Err(Error::Missing("selected objects with a Smart Flow link"));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn linked(controller: &Controller) -> Vec<ObjectId> {
    selection(controller)
        .into_iter()
        .filter(|id| {
            controller
                .document()
                .entities
                .get(id)
                .and_then(link_of)
                .is_some()
        })
        .collect()
}

/// `Smart Flow` and `Smart Flow Rigid`: flow the selection from the base surface to the target
/// surface and record the link. The rigid row moves the object frame instead of its geometry.
pub(super) fn flow(
    controller: &mut Controller,
    name: &'static str,
    picked: Point3,
    rigid: bool,
) -> Result<()> {
    let limits = *controller.coordinator.limits();
    let ((base_id, base), (target_id, target), targets) =
        resolve_frames(controller, picked, &limits)?;
    let mut mutations = Vec::with_capacity(targets.len() * 2);
    for id in targets {
        let (geometry, placement, extensions, source, revision) = {
            let entity = entity_of(controller, id)?;
            (
                entity.geometry.clone(),
                entity.placement,
                entity.extensions.clone(),
                controller.document().stamp(id)?,
                link_of(entity).map_or(0, |link| link.revision),
            )
        };
        if rigid {
            let moved = deform::rigid_flow(&base, &target, &placement, &limits)?;
            mutations.push(Mutation::Placement {
                source,
                placement: moved,
            });
        } else {
            let moved = deform::flow_along_surface(&base, &target, &geometry, &placement, &limits)?;
            mutations.push(Mutation::Geometry {
                source,
                geometry: moved,
            });
        }
        let mut extensions: BTreeMap<String, Extension> = extensions;
        extensions.insert(
            FLOW_SLOT.to_string(),
            link_extension(&Link {
                base: base_id,
                target: target_id,
                rigid,
                revision: revision + 1,
                state: ACTIVE.to_string(),
            })?,
        );
        mutations.push(Mutation::Extensions {
            source: controller.document().stamp(id)?,
            extensions,
        });
    }
    controller.commit_utility_mutations(name, mutations)
}

/// `Smart Flow Break`: mark the selected links `BROKEN` and keep the geometry where it is.
pub(super) fn break_flow(controller: &mut Controller, name: &'static str) -> Result<()> {
    let mut mutations = Vec::new();
    for id in linked(controller) {
        let (extensions, source, link) = {
            let entity = entity_of(controller, id)?;
            let link = link_of(entity).ok_or(Error::Missing("flow link"))?;
            (
                entity.extensions.clone(),
                controller.document().stamp(id)?,
                link,
            )
        };
        if link.state == BROKEN {
            continue;
        }
        let mut extensions = extensions;
        extensions.insert(
            FLOW_SLOT.to_string(),
            link_extension(&Link {
                revision: link.revision + 1,
                state: BROKEN.to_string(),
                ..link
            })?,
        );
        mutations.push(Mutation::Extensions { source, extensions });
    }
    if mutations.is_empty() {
        return Err(Error::Invalid("selection has no live Smart Flow link"));
    }
    controller.commit_utility_mutations(name, mutations)
}

/// `Smart Flow Pull Down Curves`: pull the selected curves from the target surface down onto the
/// base surface at the same parameters.
pub(super) fn pulldown(
    controller: &mut Controller,
    name: &'static str,
    picked: Point3,
) -> Result<()> {
    let limits = *controller.coordinator.limits();
    let ((_, base), (_, target), targets) = resolve_frames(controller, picked, &limits)?;
    let mut mutations = Vec::with_capacity(targets.len());
    for id in targets {
        let (geometry, placement, source) = {
            let entity = entity_of(controller, id)?;
            (
                entity.geometry.clone(),
                entity.placement,
                controller.document().stamp(id)?,
            )
        };
        let moved = deform::pulldown_from_target(&base, &target, &geometry, &placement, &limits)?;
        mutations.push(Mutation::Geometry {
            source,
            geometry: moved,
        });
    }
    controller.commit_utility_mutations(name, mutations)
}
