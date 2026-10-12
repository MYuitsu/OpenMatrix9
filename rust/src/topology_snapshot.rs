//! Owned, bounded topology observations. Incidence is not a vertex-link,
//! embedding, kernel validity, persistent-name or mutation certificate.
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_RECORDS: usize = 16384;
pub fn counts_allowed(faces: usize, loops: usize, uses: usize) -> bool {
    faces.checked_add(loops).and_then(|n| n.checked_add(uses)).is_some_and(|n| n <= MAX_RECORDS)
}
pub fn domain_valid(a:f64,b:f64)->bool {
    a.is_finite() && b.is_finite() && a < b && (b-a).is_finite()
}

// Native wire/face API requires a flat layout; these contain values only.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub surface_id: u64,
    pub orientation: u32, // 1 forward, 2 reversed
    pub reserved: u32,
    pub first_loop: usize,
    pub loop_count: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loop {
    pub role: u32, // 1 outer, 2 hole
    pub reserved: u32,
    pub first_use: usize,
    pub use_count: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Use {
    pub edge_id: u64,
    pub pcurve_id: u64,
    pub start_vertex: u64,
    pub end_vertex: u64,
    pub orientation: u32,
    pub seam: u32,
    pub degenerate: u32,
    pub reserved: u32,
    pub domain: [f64; 2],
    /// UV samples along this use, reversed when the edge occurrence is reversed.
    pub uv_start: [f64; 2],
    pub uv_end: [f64; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    BudgetExceeded,
    InvalidRange,
    InvalidIdentity,
    InvalidFlags,
    NonFiniteGeometry,
    InvalidDomain,
    DisconnectedLoop,
    InvalidLoopRole,
    InvalidSeam,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub faces: Vec<Face>,
    pub loops: Vec<Loop>,
    pub uses: Vec<Use>,
    pub edge_identity_count: usize,
    pub degenerate_use_count: usize,
    pub edge_manifold: bool,
    pub edge_closed: bool,
    /// Indices into the ordered use array. A component is connectivity, not a wire.
    pub boundary_components: Vec<Vec<usize>>,
}

fn span(first: usize, count: usize, expected: usize, total: usize) -> Result<usize, Error> {
    let end = first.checked_add(count).ok_or(Error::InvalidRange)?;
    if first != expected || count == 0 || end > total { return Err(Error::InvalidRange); }
    Ok(end)
}

impl Snapshot {
    pub fn capture(faces: &[Face], loops: &[Loop], uses: &[Use]) -> Result<Self, Error> {
        if !counts_allowed(faces.len(), loops.len(), uses.len()) { return Err(Error::BudgetExceeded); }
        let mut next_loop = 0;
        for face in faces {
            if face.surface_id == 0 { return Err(Error::InvalidIdentity); }
            if !(1..=2).contains(&face.orientation) || face.reserved != 0 { return Err(Error::InvalidFlags); }
            next_loop = span(face.first_loop, face.loop_count, next_loop, loops.len())?;
        }
        if next_loop != loops.len() { return Err(Error::InvalidRange); }
        let mut next_use = 0;
        for l in loops {
            if l.reserved != 0 { return Err(Error::InvalidFlags); }
            if !(1..=2).contains(&l.role) { return Err(Error::InvalidLoopRole); }
            next_use = span(l.first_use, l.use_count, next_use, uses.len())?;
        }
        if next_use != uses.len() { return Err(Error::InvalidRange); }
        let mut pcurves = BTreeSet::new();
        let mut incidence: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
        for (index, u) in uses.iter().enumerate() {
            if u.edge_id == 0 || u.start_vertex == 0 || u.end_vertex == 0 || u.pcurve_id == 0 || !pcurves.insert(u.pcurve_id) {
                return Err(Error::InvalidIdentity);
            }
            if !(1..=2).contains(&u.orientation) || u.seam > 1 || u.degenerate > 1 || u.reserved != 0 {
                return Err(Error::InvalidFlags);
            }
            if u.uv_start.into_iter().chain(u.uv_end).any(|x| !x.is_finite() || x.abs() > 1e150) {
                return Err(Error::NonFiniteGeometry);
            }
            let [a,b] = u.domain;
            if !domain_valid(a,b) { return Err(Error::InvalidDomain); }
            if u.degenerate == 1 && (u.start_vertex != u.end_vertex || u.seam == 1) { return Err(Error::InvalidIdentity); }
            let entries = incidence.entry(u.edge_id).or_default();
            if let Some(&old) = entries.first() {
                let v = &uses[old];
                let directed_match = if u.orientation == v.orientation {
                    u.start_vertex == v.start_vertex && u.end_vertex == v.end_vertex
                } else {
                    u.start_vertex == v.end_vertex && u.end_vertex == v.start_vertex
                };
                if u.degenerate != v.degenerate || !directed_match { return Err(Error::InvalidIdentity); }
            }
            entries.push(index);
        }
        for face in faces {
            let face_loops = &loops[face.first_loop..face.first_loop+face.loop_count];
            if face_loops.iter().filter(|l| l.role == 1).count() != 1 { return Err(Error::InvalidLoopRole); }
            let mut seams: BTreeMap<u64, Vec<&Use>> = BTreeMap::new();
            for l in face_loops {
                let chain = &uses[l.first_use..l.first_use+l.use_count];
                for (a,b) in chain.iter().zip(chain.iter().cycle().skip(1)) {
                    if a.end_vertex != b.start_vertex { return Err(Error::DisconnectedLoop); }
                }
                for u in chain { seams.entry(u.edge_id).or_default().push(u); }
            }
            for entries in seams.values() {
                if entries.iter().any(|u| u.seam == 1) &&
                   (entries.len() != 2 || entries.iter().any(|u| u.seam != 1) || entries[0].orientation == entries[1].orientation) {
                    return Err(Error::InvalidSeam);
                }
            }
        }
        let boundary: Vec<usize> = incidence.values().filter(|v| v.len() == 1 && uses[v[0]].degenerate == 0).map(|v| v[0]).collect();
        let edge_manifold = incidence.values().all(|v| uses[v[0]].degenerate == 1 || v.len() <= 2);
        let edge_closed = !faces.is_empty() && incidence.values().all(|v| uses[v[0]].degenerate == 1 || v.len() == 2);
        let mut vertices: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
        for &index in &boundary {
            for vertex in [uses[index].start_vertex, uses[index].end_vertex] { vertices.entry(vertex).or_default().push(index); }
        }
        let mut unseen: BTreeSet<usize> = boundary.into_iter().collect();
        let mut components = Vec::new();
        // Consume each vertex adjacency list once, avoiding quadratic stars.
        while let Some(seed) = unseen.pop_first() {
            let mut component = vec![seed];
            let mut cursor = 0;
            while cursor < component.len() {
                let u = uses[component[cursor]];
                for vertex in [u.start_vertex,u.end_vertex] {
                    if let Some(neighbors) = vertices.remove(&vertex) {
                        for neighbor in neighbors { if unseen.remove(&neighbor) { component.push(neighbor); } }
                    }
                }
                cursor += 1;
            }
            component.sort_unstable(); components.push(component);
        }
        components.sort_by_key(|v| v[0]);
        Ok(Self {faces:faces.to_vec(),loops:loops.to_vec(),uses:uses.to_vec(),edge_identity_count:incidence.len(),
                 degenerate_use_count:uses.iter().filter(|u| u.degenerate == 1).count(),edge_manifold,edge_closed,
                 boundary_components:components})
    }

    /// Project inspected rows into the shared descriptor schema. Neither native
    /// shape type nor edge incidence supplies a full solid/manifold certificate.
    pub fn descriptor(&self)->crate::geometry_contract::TopologyDescriptor {
        use crate::geometry_contract::{TopologyDescriptor,Fact,FaceDescriptor,TrimLoop,LoopRole,EdgeUse,Orientation,BoundaryComponent};
        let edge=|u:&Use|EdgeUse {edge_id:u.edge_id,pcurve_id:Fact::Known(u.pcurve_id),seam:Fact::Known(u.seam==1),
            orientation:if u.orientation==1 {Orientation::Forward}else{Orientation::Reversed}};
        TopologyDescriptor {
            faces:Fact::Known(self.faces.iter().map(|f|FaceDescriptor {
                surface_id:Fact::Known(f.surface_id),
                orientation:Fact::Known(if f.orientation==1 {Orientation::Forward}else{Orientation::Reversed}),
                loops:Fact::Known(self.loops[f.first_loop..f.first_loop+f.loop_count].iter().map(|l|TrimLoop {
                    role:if l.role==1 {LoopRole::Outer}else{LoopRole::Hole},
                    edge_uses:self.uses[l.first_use..l.first_use+l.use_count].iter().map(edge).collect(),
                }).collect()),
            }).collect()),
            free_boundary_components:Fact::Known(self.boundary_components.iter().map(|indices|BoundaryComponent {
                edge_uses:indices.iter().map(|&i|edge(&self.uses[i])).collect(),
            }).collect()),
            ..TopologyDescriptor::uninspected()
        }
    }
}
