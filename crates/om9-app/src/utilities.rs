// SPDX-License-Identifier: LGPL-2.1-or-later
//! Object health utilities for `OM9-L2-03-19`: validity, naked edges and
//! duplicate detection. Exact B-Rep topology comes from OCCT when available.
use std::collections::BTreeMap;

use om9_core::{budget::Limits, geometry::Geometry, identity::ObjectId, model::Document};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthEntry {
    pub id: ObjectId,
    pub label: String,
    pub kind: &'static str,
    pub valid: bool,
    pub naked_edges: usize,
    pub non_manifold_edges: usize,
    pub closed: bool,
    pub duplicate_of: Option<ObjectId>,
}

pub fn mesh_edge_report(mesh: &om9_core::geometry::Mesh) -> (usize, usize) {
    let mut edges: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for face in &mesh.faces {
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edges.entry(key).or_default() += 1;
        }
    }
    let boundary = edges.values().filter(|count| **count == 1).count();
    let non_manifold = edges.values().filter(|count| **count > 2).count();
    (boundary, non_manifold)
}

pub fn health_entries(document: &Document, limits: &Limits) -> Vec<HealthEntry> {
    let mut seen: Vec<(Geometry, ObjectId)> = Vec::new();
    let mut entries = Vec::with_capacity(document.entities.len());
    for entity in document.entities.values() {
        let duplicate_of = seen
            .iter()
            .find(|(geometry, _)| geometry == &entity.geometry)
            .map(|(_, id)| *id);
        if duplicate_of.is_none() {
            seen.push((entity.geometry.clone(), entity.id));
        }
        let (valid, naked_edges, non_manifold_edges, closed) = match &entity.geometry {
            Geometry::Brep(brep) => {
                #[cfg(feature = "occt-brep")]
                {
                    match om9_kernel::editing::geometry_health(brep, limits) {
                        Ok(health) => (health.valid, health.naked_edges, 0, health.closed),
                        Err(_) => (false, 0, 0, false),
                    }
                }
                #[cfg(not(feature = "occt-brep"))]
                {
                    (brep.validate(limits).is_ok(), 0, 0, false)
                }
            }
            Geometry::Mesh(mesh) => {
                let (boundary, non_manifold) = mesh_edge_report(mesh);
                (
                    mesh.validate(limits).is_ok(),
                    boundary,
                    non_manifold,
                    boundary == 0,
                )
            }
            geometry => (geometry.validate(limits).is_ok(), 0, 0, false),
        };
        entries.push(HealthEntry {
            id: entity.id,
            label: entity.label.clone(),
            kind: entity.geometry.kind().tag(),
            valid,
            naked_edges,
            non_manifold_edges,
            closed,
            duplicate_of,
        });
    }
    entries
}

pub fn bad_object_ids(entries: &[HealthEntry]) -> Vec<ObjectId> {
    entries
        .iter()
        .filter(|entry| !entry.valid)
        .map(|entry| entry.id)
        .collect()
}

pub fn naked_edge_ids(entries: &[HealthEntry]) -> Vec<ObjectId> {
    entries
        .iter()
        .filter(|entry| entry.naked_edges > 0)
        .map(|entry| entry.id)
        .collect()
}

pub fn summary(entries: &[HealthEntry]) -> String {
    let bad = entries.iter().filter(|entry| !entry.valid).count();
    let open = entries.iter().filter(|entry| entry.naked_edges > 0).count();
    let duplicate = entries
        .iter()
        .filter(|entry| entry.duplicate_of.is_some())
        .count();
    format!(
        "Health: {} objects, {} invalid, {} open, {} duplicate",
        entries.len(),
        bad,
        open,
        duplicate
    )
}
