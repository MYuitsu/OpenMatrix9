// SPDX-License-Identifier: LGPL-2.1-or-later
//! Persisted source polygons and logical From Lines face identity.
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Geometry, Point3},
    model::{Entity, TSPLINE_FROM_LINES_SLOT},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Serialize, Deserialize)]
pub struct CageRecord {
    pub vertices: Vec<Point3>,
    pub polygons: Vec<Vec<u32>>,
    pub polygon_face_ids: Vec<usize>,
    pub display_face_ids: Vec<usize>,
    pub smooth: bool,
}
impl CageRecord {
    pub fn read(entity: &Entity) -> Result<Self> {
        Self::read_with_limits(entity, &Limits::default())
    }
    pub fn read_with_limits(entity: &Entity, limits: &Limits) -> Result<Self> {
        let ext = entity
            .extension(TSPLINE_FROM_LINES_SLOT)
            .ok_or(Error::Missing("From Lines control cage"))?;
        if ext.version != 1 {
            return Err(Error::Unsupported("From Lines cage version"));
        }
        if ext.payload.len() as u64 > limits.max_bytes {
            return Err(Error::Budget("From Lines cage payload"));
        }
        let record: Self = serde_json::from_str(&ext.payload)
            .map_err(|_| Error::Invalid("From Lines control cage"))?;
        let Geometry::Mesh(mesh) = &entity.geometry else {
            return Err(Error::Invalid("From Lines display mesh"));
        };
        let corners = record
            .polygons
            .iter()
            .try_fold(0usize, |a, f| a.checked_add(f.len()))
            .ok_or(Error::Overflow("cage face corners"))?;
        let mut usage = Usage::array::<Point3>(record.vertices.len())?;
        usage.add(Usage::array::<Vec<u32>>(record.polygons.len())?, limits)?;
        usage.add(Usage::array::<u32>(corners)?, limits)?;
        usage.add(
            Usage::array::<usize>(
                record
                    .polygon_face_ids
                    .len()
                    .saturating_add(record.display_face_ids.len()),
            )?,
            limits,
        )?;
        if record.vertices.len() as u64 > limits.max_points
            || record.polygons.len() as u64 > limits.max_nodes
        {
            return Err(Error::Budget("From Lines control polygons"));
        }
        let source: BTreeSet<_> = record.polygon_face_ids.iter().collect();
        if record.polygons.is_empty()
            || source != record.display_face_ids.iter().collect()
            || record.polygon_face_ids.len() != record.polygons.len()
            || record.display_face_ids.len() != mesh.faces.len()
            || record.vertices.iter().flatten().any(|x| !x.is_finite())
            || record.polygons.iter().any(|f| {
                f.len() < 3
                    || f.iter().any(|&v| v as usize >= record.vertices.len())
                    || f.iter().collect::<BTreeSet<_>>().len() != f.len()
            })
        {
            return Err(Error::Invalid("From Lines cage topology"));
        }
        Ok(record)
    }
}
