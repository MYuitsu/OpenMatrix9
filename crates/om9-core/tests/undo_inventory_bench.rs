// SPDX-License-Identifier: LGPL-2.1-or-later
//! Manual timing probe for managed Undo inventory; not an acceptance gate.
//! Distinct shapes have independent topology allocations; results are not
//! directly comparable with the older shared-topology fixture.
use om9_core::{brep::{Brep, BrepDisplay, BrepTopology}, budget::Limits,
    geometry::{Geometry, Transform}, model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World}};
use std::time::Instant;

#[test]
#[ignore = "manual release timing probe"]
fn source_inventory_with_undo_history() {
    let limits = Limits::default();
    for distinct in [false, true] {
        for count in [24usize, 64, 128, 256, 512] {
            let topology = BrepTopology { faces: vec![uuid::Uuid::from_u128(1)], edges: vec![], vertex_ids: vec![], face_edges: vec![] };
            let shape = |index: usize| Brep::from_native(format!("CASCADE Topology {}", if distinct { index } else { 0 }), BrepDisplay {
                source_sha256: [0; 32], deflection_mm: 0.1, points: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
                triangles: vec![[0,1,2,0]], segments: vec![], vertices: vec![], topology: topology.clone().into(),
            }, &limits).unwrap();
            let shared = shape(0);
            let mut document = Document::new("inventory timing");
            let mut first = None;
            for index in 0..count {
                let entity = Entity::new("body", Geometry::Brep(if distinct { shape(index) } else { shared.clone() }), None);
                first.get_or_insert(entity.id);
                document.entities.insert(entity.id, entity);
            }
            let id = first.unwrap();
            let doc = document.id;
            let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
            for step in 1..=8 {
                let source = coordinator.world().documents[&doc].stamp(id).unwrap();
                let request = TransactionRequest::new("move", vec![Mutation::Placement {
                    source, placement: Transform::translation([step as f64, 0., 0.]),
                }]);
                let prepared = coordinator.prepare(request).unwrap();
                coordinator.commit(prepared).unwrap();
            }
            let mut samples = Vec::new();
            let mut retained_samples = Vec::new();
            let mut bytes = 0;
            for run in 0..5 {
                for full in if run % 2 == 0 { [true, false] } else { [false, true] } {
                    let start = Instant::now();
                    if full { bytes = coordinator.undo_memory_bytes().unwrap(); }
                    else { std::hint::black_box(coordinator.retained_usage().unwrap()); }
                    let elapsed = start.elapsed().as_micros();
                    if full { samples.push(elapsed); } else { retained_samples.push(elapsed); }
                }
            }
            samples.sort_unstable();
            retained_samples.sort_unstable();
            println!("distinct={distinct} objects={count} undo_steps=8 inventory_bytes={bytes} inventory_median_us={} inventory_max_us={} retained_median_us={} retained_max_us={}",
                samples[2], samples[4], retained_samples[2], retained_samples[4]);
        }
    }
}
