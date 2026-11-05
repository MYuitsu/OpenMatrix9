// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned OCCT source plus a revision-bound, disposable display/topology snapshot.
//! Native indices never serve as persistent IDs. Replacing this snapshot creates new UUIDs;
//! publication/source stamps invalidate previous picks instead of guessing topology continuity.
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Point3, finite},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brep {
    source: Arc<String>,
    source_sha256: [u8; 32],
    display_sha256: [u8; 32],
    pub display: Arc<BrepDisplay>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrepDisplay {
    pub source_sha256: [u8; 32],
    pub deflection_mm: f64,
    pub points: Vec<Point3>,
    /// Three display vertex offsets and a snapshot-local face offset.
    pub triangles: Vec<[u32; 4]>,
    /// Two display vertex offsets and a snapshot-local edge offset.
    pub segments: Vec<[u32; 3]>,
    /// A display vertex offset for each actual topological vertex.
    pub vertices: Vec<u32>,
    pub faces: Vec<Uuid>,
    pub edges: Vec<Uuid>,
    pub vertex_ids: Vec<Uuid>,
    /// Face/edge adjacency, retaining closed boundaries and inner wires.
    pub face_edges: Vec<[u32; 2]>,
}
impl Brep {
    /// Adapter publication boundary. The kernel validates the shape before calling this constructor.
    /// Portable validation checks ownership/bounds and source binding, not OCCT validity.
    pub fn from_native(source: String, mut display: BrepDisplay, limits: &Limits) -> Result<Self> {
        let digest = Sha256::digest(source.as_bytes()).into();
        display.source_sha256 = digest;
        let display_sha256 = crate::retained::fingerprint(&display)?;
        let shape = Self {
            source: Arc::new(source),
            source_sha256: digest,
            display_sha256,
            display: Arc::new(display),
        };
        shape.validate(limits)?;
        Ok(shape)
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub(crate) fn source_allocation_key(&self) -> usize {
        Arc::as_ptr(&self.source) as usize
    }
    pub(crate) fn source_allocation_bytes(&self) -> Result<u64> {
        Usage::array::<u8>(self.source.capacity())?.bytes
            .checked_add(Usage::array::<usize>(2)?.bytes)
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<String>() as u64))
            .ok_or(Error::Overflow("BRep source allocation"))
    }
    pub(crate) fn display_allocation_key(&self) -> usize {
        Arc::as_ptr(&self.display) as usize
    }
    pub(crate) fn display_allocation_bytes(&self) -> Result<u64> {
        let d = &self.display;
        let sizes = [
            Usage::array::<BrepDisplay>(1)?.bytes,
            Usage::array::<usize>(2)?.bytes,
            Usage::array::<Point3>(d.points.capacity())?.bytes,
            Usage::array::<[u32; 4]>(d.triangles.capacity())?.bytes,
            Usage::array::<[u32; 3]>(d.segments.capacity())?.bytes,
            Usage::array::<u32>(d.vertices.capacity())?.bytes,
            Usage::array::<[u32; 2]>(d.face_edges.capacity())?.bytes,
            Usage::array::<Uuid>(d.faces.capacity())?.bytes,
            Usage::array::<Uuid>(d.edges.capacity())?.bytes,
            Usage::array::<Uuid>(d.vertex_ids.capacity())?.bytes,
        ];
        sizes.into_iter().try_fold(0u64, |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("BRep display allocation")))
    }
    /// Inventory owned buffers without hashing, topology checks, or scratch allocation.
    /// Callers must still validate untrusted data before publication.
    pub fn storage_usage(&self, limits: &Limits) -> Result<Usage> {
        let d = &self.display;
        let mut usage = Usage { bytes: self.source_allocation_bytes()?, ..Usage::default() };
        usage.add(Usage { bytes: self.display_allocation_bytes()?, ..Usage::default() }, limits)?;
        usage.points = d.points.len() as u64;
        usage.nodes = d
            .faces
            .len()
            .checked_add(d.edges.len())
            .and_then(|n| n.checked_add(d.vertices.len()))
            .ok_or(Error::Overflow("BRep topology count"))? as u64;
        usage.check(limits)?;
        Ok(usage)
    }
    pub fn validate(&self, limits: &Limits) -> Result<Usage> {
        let d = &self.display;
        let digest: [u8; 32] = Sha256::digest(self.source.as_bytes()).into();
        if self.source.is_empty()
            || !self.source.contains("CASCADE Topology")
            || digest != self.source_sha256
            || digest != d.source_sha256
            || crate::retained::fingerprint(d)? != self.display_sha256
        {
            return Err(Error::Invalid("BRep source/cache binding"));
        }
        if !d.deflection_mm.is_finite()
            || d.deflection_mm <= 0.
            || d.faces.is_empty()
            || d.triangles.is_empty()
            || d.vertices.len() != d.vertex_ids.len()
        {
            return Err(Error::Invalid("BRep display topology"));
        }
        let mut usage = self.storage_usage(limits)?;
        for p in &d.points {
            finite(*p)?;
        }
        for [a, b, c, f] in &d.triangles {
            if [a, b, c].iter().any(|i| **i as usize >= d.points.len())
                || *f as usize >= d.faces.len()
                || a == b
                || b == c
                || a == c
            {
                return Err(Error::Invalid("BRep triangle topology"));
            }
        }
        if d.segments.iter().any(|[a, b, e]| {
            *a as usize >= d.points.len()
                || *b as usize >= d.points.len()
                || *e as usize >= d.edges.len()
        }) || d.vertices.iter().any(|i| *i as usize >= d.points.len())
            || d.face_edges
                .iter()
                .any(|[f, e]| *f as usize >= d.faces.len() || *e as usize >= d.edges.len())
        {
            return Err(Error::Invalid("BRep boundary topology"));
        }
        // Sorting UUIDs is bounded by the admitted topology budget; no native handle is retained.
        let mut ids = Vec::new();
        let count = d.faces.len() + d.edges.len() + d.vertex_ids.len();
        usage.add(Usage::array::<Uuid>(count)?, limits)?;
        ids.try_reserve_exact(count)
            .map_err(|_| Error::Budget("BRep topology identities"))?;
        for keys in [&d.faces, &d.edges, &d.vertex_ids] {
            ids.extend_from_slice(keys);
        }
        ids.sort_unstable();
        if ids.iter().any(Uuid::is_nil) || ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::Invalid("BRep topology identity"));
        }
        usage.bytes -= Usage::array::<Uuid>(count)?.bytes;
        Ok(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_matches_validation_and_does_not_waive_scratch_budget() {
        let limits = Limits::default();
        let shape = Brep::from_native(
            "CASCADE Topology fixture".into(),
            BrepDisplay {
                source_sha256: [0; 32],
                deflection_mm: 0.01,
                points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                triangles: vec![[0, 1, 2, 0]],
                segments: vec![],
                vertices: vec![],
                faces: vec![Uuid::from_u128(1)],
                edges: vec![],
                vertex_ids: vec![],
                face_edges: vec![],
            },
            &limits,
        )
        .unwrap();
        let cloned = shape.clone();
        assert_eq!(shape.source().as_ptr(), cloned.source().as_ptr(), "immutable source should be shared across clones");
        assert!(Arc::ptr_eq(&shape.display, &cloned.display));
        let serialized = serde_json::to_vec(&shape).unwrap();
        let packet: serde_json::Value = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(packet["source"].as_str(), Some(shape.source()));
        assert_eq!(packet["display"], serde_json::to_value(shape.display.as_ref()).unwrap());
        let restored: Brep = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(restored, shape);
        restored.validate(&limits).unwrap();
        let mut changed = cloned;
        Arc::make_mut(&mut changed.display).points[0][0] = 42.;
        assert!(!Arc::ptr_eq(&shape.display, &changed.display));
        assert!(changed.validate(&limits).is_err(), "modified snapshot must fail source binding");
        assert_eq!(shape.display.points[0][0], 0.);
        let usage = shape.storage_usage(&limits).unwrap();
        assert_eq!(usage, shape.validate(&limits).unwrap());
        let tight = Limits {
            max_bytes: usage.bytes,
            ..limits
        };
        assert!(shape.storage_usage(&tight).is_ok());
        assert!(matches!(shape.validate(&tight), Err(Error::Budget(_))));
    }
}
