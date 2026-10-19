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
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brep {
    source: String,
    source_sha256: [u8; 32],
    display_sha256: [u8; 32],
    pub display: BrepDisplay,
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
            source,
            source_sha256: digest,
            display_sha256,
            display,
        };
        shape.validate(limits)?;
        Ok(shape)
    }
    pub fn source(&self) -> &str {
        &self.source
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
        let mut usage = Usage::array::<u8>(self.source.capacity())?;
        usage.add(Usage::array::<Point3>(d.points.capacity())?, limits)?;
        usage.add(Usage::array::<[u32; 4]>(d.triangles.capacity())?, limits)?;
        usage.add(Usage::array::<[u32; 3]>(d.segments.capacity())?, limits)?;
        usage.add(Usage::array::<u32>(d.vertices.capacity())?, limits)?;
        usage.add(Usage::array::<[u32; 2]>(d.face_edges.capacity())?, limits)?;
        for keys in [&d.faces, &d.edges, &d.vertex_ids] {
            usage.add(Usage::array::<Uuid>(keys.capacity())?, limits)?;
        }
        usage.points = d.points.len() as u64;
        usage.nodes = (d.faces.len() + d.edges.len() + d.vertices.len()) as u64;
        usage.check(limits)?;
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
