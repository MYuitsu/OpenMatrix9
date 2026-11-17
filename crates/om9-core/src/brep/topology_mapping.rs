// SPDX-License-Identifier: LGPL-2.1-or-later
//! Conservative matching of *attested* source topology manifests.
//!
//! This module does not generate or certify native descriptors. A native
//! adapter must bind each descriptor to the exact indexed topology used to
//! create the mesh, and include placement and orientation in its digest. Until
//! that exists, a manifest must not be fabricated from mesh triangles or v4 IDs.

use crate::{Error, Result, budget::{Limits, Usage}, brep::BrepTopology};
use uuid::Uuid;

/// A source-bound, kernel-generated SHA-256 descriptor for one subshape.
pub type Descriptor = [u8; 32];

/// An oriented incidence of two indexed topology elements. Repeated entries
/// are significant (e.g. two uses of the same seam edge).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Incidence {
    pub parent: u32,
    pub child: u32,
    pub orientation: u8,
}

/// Borrowed manifest. Its buffers, source and provenance must already be
/// accounted and validated by the caller. `kernel` must identify the exact
/// descriptor algorithm and kernel build, not just a broad version family.
pub struct Manifest<'a> {
    pub source_sha256: [u8; 32],
    pub kernel: &'a str,
    pub faces: &'a [Descriptor],
    pub edges: &'a [Descriptor],
    pub vertices: &'a [Descriptor],
    pub face_edges: &'a [Incidence],
    pub edge_vertices: &'a [Incidence],
}

/// Maps old snapshot indices to their corresponding new snapshot indices.
pub struct IndexMapping {
    faces: Vec<u32>,
    edges: Vec<u32>,
    vertices: Vec<u32>,
}

impl IndexMapping {
    /// Own an adapter-supplied mapping. This does not attest provenance;
    /// `reorder_topology_admitted` checks bounds, bijection and adjacency.
    pub fn from_reference_indices(faces: Vec<u32>, edges: Vec<u32>, vertices: Vec<u32>) -> Self {
        Self { faces, edges, vertices }
    }
    pub fn faces(&self) -> &[u32] { &self.faces }
    pub fn edges(&self) -> &[u32] { &self.edges }
    pub fn vertices(&self) -> &[u32] { &self.vertices }
    pub fn storage_bytes(&self) -> Result<u64> {
        [Usage::array::<u32>(self.faces.capacity())?.bytes, Usage::array::<u32>(self.edges.capacity())?.bytes,
            Usage::array::<u32>(self.vertices.capacity())?.bytes].into_iter()
            .try_fold(0u64, |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("topology mapping")))
    }
    /// Reassign persistent IDs to the new snapshot order. Both manifests must
    /// have been attested by the native adapter before using this result for a
    /// document or picking; `IndexMapping` alone does not attest provenance.
    /// `limits` covers the returned vectors while the original topology and
    /// mapping are live, including the caller's other retained allocations.
    pub fn reorder_topology(&self, old: &BrepTopology, new_face_edges: &[[u32; 2]], limits: &Limits) -> Result<BrepTopology> {
        self.reorder_topology_admitted(old, new_face_edges, limits, &mut |_, _| Ok(()))
    }
    pub fn reorder_topology_admitted(&self, old: &BrepTopology, new_face_edges: &[[u32; 2]], limits: &Limits,
        admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<BrepTopology> {
        let mut usage = Usage::default();
        let faces = reorder_ids(&old.faces, &self.faces, &mut usage, limits, admit)?;
        let edges = reorder_ids(&old.edges, &self.edges, &mut usage, limits, admit)?;
        let vertex_ids = reorder_ids(&old.vertex_ids, &self.vertices, &mut usage, limits, admit)?;
        let mut face_edges = array::<[u32; 2]>(new_face_edges.len(), &mut usage, limits, admit)?;
        for edge in new_face_edges {
            if edge[0] as usize >= faces.len() || edge[1] as usize >= edges.len() {
                return Err(Error::Invalid("topology face-edge index"));
            }
            face_edges.push(*edge);
        }
        if old.face_edges.len() != face_edges.len() { return Err(Error::Invalid("topology face-edge count")); }
        let mut mapped = array::<[u32; 2]>(old.face_edges.len(), &mut usage, limits, admit)?;
        let mut expected = array::<[u32; 2]>(face_edges.len(), &mut usage, limits, admit)?;
        for [face, edge] in &old.face_edges {
            mapped.push([
                *self.faces.get(*face as usize).ok_or(Error::Invalid("topology face index"))?,
                *self.edges.get(*edge as usize).ok_or(Error::Invalid("topology edge index"))?,
            ]);
        }
        expected.extend_from_slice(&face_edges);
        mapped.sort_unstable();
        expected.sort_unstable();
        if mapped != expected { return Err(Error::Invalid("topology face-edge mismatch")); }
        usage.bytes = usage.bytes.checked_sub(Usage::array::<[u32; 2]>(mapped.capacity())?.bytes)
            .and_then(|v| v.checked_sub(Usage::array::<[u32; 2]>(expected.capacity()).ok()?.bytes))
            .ok_or(Error::Overflow("topology reorder usage"))?;
        drop(mapped); drop(expected);
        admit(usage.bytes, 0)?;
        Ok(BrepTopology { faces, edges, vertex_ids, face_edges })
    }
}

fn reorder_ids(ids: &[Uuid], mapping: &[u32], usage: &mut Usage, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Vec<Uuid>> {
    if ids.len() != mapping.len() { return Err(Error::Invalid("topology ID count")); }
    let mut output = array::<Uuid>(ids.len(), usage, limits, admit)?;
    output.resize(ids.len(), Uuid::nil());
    for (id, index) in ids.iter().zip(mapping) {
        if id.is_nil() { return Err(Error::Invalid("topology ID")); }
        let slot = output.get_mut(*index as usize).ok_or(Error::Invalid("topology mapping index"))?;
        if !slot.is_nil() { return Err(Error::Invalid("ambiguous topology mapping")); }
        *slot = *id;
    }
    Ok(output)
}

fn array<T>(count: usize, usage: &mut Usage, limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Vec<T>> {
    let requested = Usage::array::<T>(count)?;
    let before = usage.bytes;
    usage.add(requested, limits)?;
    admit(before, requested.bytes)?;
    let mut output = Vec::new();
    output.try_reserve_exact(count).map_err(|_| Error::Budget("topology mapping allocation"))?;
    // The allocator may supply more capacity than requested.
    let extra = output.capacity().checked_sub(count).ok_or(Error::Overflow("topology mapping capacity"))?;
    usage.add(Usage::array::<T>(extra)?, limits)?;
    admit(usage.bytes, 0)?;
    Ok(output)
}

fn indices(old: &[Descriptor], new: &[Descriptor], usage: &mut Usage, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Vec<u32>> {
    if old.len() != new.len() || old.len() > u32::MAX as usize {
        return Err(Error::Invalid("topology descriptor count"));
    }
    let mut sorted = array::<(Descriptor, u32)>(new.len(), usage, limits, admit)?;
    for (index, key) in new.iter().enumerate() {
        if *key == [0; 32] { return Err(Error::Invalid("topology descriptor")); }
        sorted.push((*key, index as u32));
    }
    sorted.sort_unstable_by_key(|entry| entry.0);
    if sorted.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(Error::Invalid("ambiguous topology descriptor"));
    }
    let mut mapping = array::<u32>(old.len(), usage, limits, admit)?;
    // Each candidate index must be used once. A byte marker avoids another
    // u32 array and a second sort while keeping bounded scratch accounting.
    let mut seen = array::<u8>(new.len(), usage, limits, admit)?;
    seen.resize(new.len(), 0);
    for key in old {
        let index = sorted.binary_search_by_key(key, |entry| entry.0)
            .map_err(|_| Error::Invalid("topology descriptor mismatch"))?;
        let candidate = sorted[index].1;
        let marker = &mut seen[candidate as usize];
        if *marker != 0 { return Err(Error::Invalid("ambiguous topology descriptor")); }
        *marker = 1;
        mapping.push(candidate);
    }
    // Scratch allocations above are no longer live. Capacity, rather than len,
    // is subtracted so later admission still covers all remaining allocations.
    usage.bytes = usage.bytes.checked_sub(Usage::array::<(Descriptor, u32)>(sorted.capacity())?.bytes)
        .and_then(|v| v.checked_sub(Usage::array::<u8>(seen.capacity()).ok()?.bytes))
        .ok_or(Error::Overflow("topology mapping usage"))?;
    drop(sorted); drop(seen);
    admit(usage.bytes, 0)?;
    Ok(mapping)
}

fn incidences(old: &[Incidence], new: &[Incidence], parents: &[u32], children: &[u32],
              usage: &mut Usage, limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<()> {
    if old.len() != new.len() { return Err(Error::Invalid("topology incidence count")); }
    let mut mapped = array::<Incidence>(old.len(), usage, limits, admit)?;
    let mut expected = array::<Incidence>(new.len(), usage, limits, admit)?;
    for incidence in old {
        if incidence.orientation > 3 { return Err(Error::Invalid("topology orientation")); }
        mapped.push(Incidence {
            parent: *parents.get(incidence.parent as usize).ok_or(Error::Invalid("topology parent"))?,
            child: *children.get(incidence.child as usize).ok_or(Error::Invalid("topology child"))?,
            orientation: incidence.orientation,
        });
    }
    for incidence in new {
        if incidence.orientation > 3 || incidence.parent as usize >= parents.len() || incidence.child as usize >= children.len() {
            return Err(Error::Invalid("topology incidence"));
        }
        expected.push(*incidence);
    }
    mapped.sort_unstable();
    expected.sort_unstable();
    if mapped != expected { return Err(Error::Invalid("topology incidence mismatch")); }
    usage.bytes = usage.bytes.checked_sub(Usage::array::<Incidence>(mapped.capacity())?.bytes)
        .and_then(|v| v.checked_sub(Usage::array::<Incidence>(expected.capacity()).ok()?.bytes))
        .ok_or(Error::Overflow("topology mapping usage"))?;
    drop(mapped); drop(expected);
    admit(usage.bytes, 0)?;
    Ok(())
}

/// Match two already-attested manifests. Equality of descriptors is necessary
/// but not sufficient: this also checks a bijection and the complete oriented
/// face/edge and edge/vertex incidence multisets. Ambiguity always fails.
///
/// `limits` must be the caller's remaining budget after both manifests and all
/// other live allocations; returned mapping allocations remain charged there.
pub fn match_manifests(old: &Manifest<'_>, new: &Manifest<'_>, limits: &Limits) -> Result<IndexMapping> {
    match_manifests_admitted(old, new, limits, &mut |_, _| Ok(()))
}
pub fn match_manifests_admitted(old: &Manifest<'_>, new: &Manifest<'_>, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<IndexMapping> {
    if old.source_sha256 == [0; 32] || old.source_sha256 != new.source_sha256
        || old.kernel.is_empty() || old.kernel != new.kernel || old.kernel.len() > limits.max_name_bytes {
        return Err(Error::Invalid("topology provenance"));
    }
    if old.faces.is_empty() { return Err(Error::Invalid("topology has no faces")); }
    let nodes = old.faces.len().checked_add(old.edges.len()).and_then(|v| v.checked_add(old.vertices.len()))
        .and_then(|v| v.checked_add(old.face_edges.len())).and_then(|v| v.checked_add(old.edge_vertices.len()))
        .ok_or(Error::Overflow("topology mapping nodes"))?;
    if nodes as u64 > limits.max_nodes { return Err(Error::Budget("topology mapping nodes")); }
    let mut usage = Usage::default();
    let faces = indices(old.faces, new.faces, &mut usage, limits, admit)?;
    let edges = indices(old.edges, new.edges, &mut usage, limits, admit)?;
    let vertices = indices(old.vertices, new.vertices, &mut usage, limits, admit)?;
    incidences(old.face_edges, new.face_edges, &faces, &edges, &mut usage, limits, admit)?;
    incidences(old.edge_vertices, new.edge_vertices, &edges, &vertices, &mut usage, limits, admit)?;
    Ok(IndexMapping { faces, edges, vertices })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u8) -> Descriptor { [n; 32] }
    fn incidence(parent: u32, child: u32, orientation: u8) -> Incidence {
        Incidence { parent, child, orientation }
    }

    #[test]
    fn permuted_topology_maps_ids_only_when_full_oriented_graph_matches() {
        let old = Manifest { source_sha256: key(99), kernel: "occt-8-build-a;descriptor-v1",
            faces: &[key(1), key(2)], edges: &[key(3), key(4)], vertices: &[key(5), key(6)],
            face_edges: &[incidence(0, 0, 0), incidence(1, 0, 1), incidence(1, 1, 0)],
            edge_vertices: &[incidence(0, 0, 0), incidence(0, 1, 1), incidence(1, 1, 0)] };
        let mut new = Manifest { source_sha256: key(99), kernel: old.kernel,
            faces: &[key(2), key(1)], edges: &[key(4), key(3)], vertices: &[key(6), key(5)],
            face_edges: &[incidence(0, 0, 0), incidence(0, 1, 1), incidence(1, 1, 0)],
            edge_vertices: &[incidence(0, 0, 0), incidence(1, 0, 1), incidence(1, 1, 0)] };
        let mapping = match_manifests(&old, &new, &Limits::default()).unwrap();
        assert_eq!(mapping.faces, [1, 0]);
        assert_eq!(mapping.edges, [1, 0]);
        assert_eq!(mapping.vertices, [1, 0]);
        let old_topology = BrepTopology { faces: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            edges: vec![Uuid::from_u128(3), Uuid::from_u128(4)],
            vertex_ids: vec![Uuid::from_u128(5), Uuid::from_u128(6)],
            face_edges: vec![[0, 0], [1, 0], [1, 1]] };
        let result = mapping.reorder_topology(&old_topology, &[[0, 0], [0, 1], [1, 1]], &Limits::default()).unwrap();
        assert_eq!(result.faces, vec![Uuid::from_u128(2), Uuid::from_u128(1)]);
        assert_eq!(result.edges, vec![Uuid::from_u128(4), Uuid::from_u128(3)]);
        assert_eq!(result.vertex_ids, vec![Uuid::from_u128(6), Uuid::from_u128(5)]);
        assert!(mapping.reorder_topology(&old_topology, &[[0, 0], [0, 1], [1, 0]], &Limits::default()).is_err());
        let invalid = IndexMapping { faces: vec![0, 0], edges: vec![0, 1], vertices: vec![0, 1] };
        assert!(invalid.reorder_topology(&old_topology, &old_topology.face_edges, &Limits::default()).is_err());
        let limits = Limits { max_bytes: 1, ..Limits::default() };
        assert!(matches!(mapping.reorder_topology(&old_topology, &old_topology.face_edges, &limits), Err(Error::Budget(_))));
        let wrong_orientation = [incidence(0, 0, 0), incidence(0, 1, 0), incidence(1, 1, 0)];
        new.face_edges = &wrong_orientation;
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
        let correct_faces = [incidence(0, 0, 0), incidence(0, 1, 1), incidence(1, 1, 0)];
        let wrong_vertices = [incidence(0, 1, 0), incidence(1, 0, 1), incidence(1, 1, 0)];
        new.face_edges = &correct_faces;
        new.edge_vertices = &wrong_vertices;
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
    }

    #[test]
    fn ambiguous_missing_foreign_and_underbudget_manifests_are_refused() {
        let old = Manifest { source_sha256: key(99), kernel: "kernel-a", faces: &[key(1), key(2)],
            edges: &[], vertices: &[], face_edges: &[], edge_vertices: &[] };
        let mut new = Manifest { source_sha256: old.source_sha256, kernel: old.kernel, faces: &[key(1), key(1)],
            edges: &[], vertices: &[], face_edges: &[], edge_vertices: &[] };
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
        let missing = [key(1), key(3)];
        new.faces = &missing;
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
        new.faces = old.faces;
        new.kernel = "kernel-b";
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
        new.kernel = old.kernel;
        new.source_sha256 = key(98);
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
        new.source_sha256 = old.source_sha256;
        let limits = Limits { max_bytes: 1, ..Limits::default() };
        assert!(matches!(match_manifests(&old, &new, &limits), Err(Error::Budget(_))));
    }

    #[test]
    fn seam_multiplicity_and_invalid_indices_are_not_discarded() {
        let seam = [incidence(0, 0, 0), incidence(0, 0, 1)];
        let old = Manifest { source_sha256: key(99), kernel: "kernel-a", faces: &[key(1)],
            edges: &[key(2)], vertices: &[], face_edges: &seam, edge_vertices: &[] };
        let mut new = Manifest { source_sha256: old.source_sha256, kernel: old.kernel,
            faces: old.faces, edges: old.edges, vertices: &[], face_edges: &seam, edge_vertices: &[] };
        assert!(match_manifests(&old, &new, &Limits::default()).is_ok());
        let duplicate = [incidence(0, 0, 0), incidence(0, 0, 0)];
        new.face_edges = &duplicate;
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
        let invalid = [incidence(0, 0, 0), incidence(0, 1, 1)];
        new.face_edges = &invalid;
        assert!(match_manifests(&old, &new, &Limits::default()).is_err());
    }
}
