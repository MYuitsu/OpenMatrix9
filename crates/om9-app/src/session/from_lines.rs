// SPDX-License-Identifier: LGPL-2.1-or-later
//! Prepare the From Lines preview from a frozen document snapshot. Geometry and
//! tolerances remain in world document units through picking and cage creation.
use om9_addon_geom::subd::from_lines::{FaceLayout, LineNetwork};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    curve_ops::linear_path::placed_vertices,
    geometry::Geometry,
    identity::ObjectId,
    model::Document,
};
use std::{collections::BTreeSet, sync::Arc};

mod runtime;
pub use om9_core::cage::from_lines::CageRecord;
pub(super) use runtime::FromLines;
pub(super) use runtime::face_subset;
pub(crate) const CODE: &str = "tsFromLines";

pub fn prepare(
    document: &Document,
    selected: &[ObjectId],
    tolerance: f64,
    max_auto_face: usize,
    limits: &Limits,
    work: &mut impl FnMut(usize) -> Result<()>,
) -> Result<FaceLayout> {
    if selected.is_empty() {
        return Err(Error::Missing("select lines or polylines"));
    }
    if selected.len() as u64 > limits.max_objects {
        return Err(Error::Budget("From Lines inputs"));
    }
    let mut paths = Vec::new();
    let mut total = 0usize;
    let mut seen = BTreeSet::new();
    for &id in selected {
        if !seen.insert(id) {
            continue;
        }
        let entity = document
            .entities
            .get(&id)
            .ok_or(Error::Stale("From Lines source"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Unsupported("From Lines requires linear curves"));
        };
        let stored = Usage::array::<[f64; 3]>(total)?;
        let remaining = Limits {
            max_points: limits.max_points.saturating_sub(total as u64),
            max_bytes: limits.max_bytes.saturating_sub(stored.bytes),
            ..*limits
        };
        let path = placed_vertices(curve, entity.placement, tolerance, &remaining, &mut || {
            work(1)
        })?;
        total = total
            .checked_add(path.len())
            .ok_or(Error::Overflow("From Lines input points"))?;
        paths.push(path);
    }
    let mut interrupted = None;
    let mut check = |units| {
        work(units).map_err(|error| {
            interrupted = Some(error);
            "From Lines preparation interrupted"
        })
    };
    let result = (|| {
        let network = LineNetwork::from_paths(&paths, tolerance, &mut check)?;
        drop(paths);
        let faces = network.automatic_faces(max_auto_face, &mut check)?;
        FaceLayout::new(Arc::new(network), faces, &mut check)
    })();
    result.map_err(|reason| interrupted.unwrap_or(Error::Invalid(reason)))
}
