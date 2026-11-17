// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in timing probe for one-by-one versus atomic source capture.
//! Distinct shapes have independent topology allocations; results are not
//! directly comparable with the older shared-topology fixture.
use om9_core::{
    brep::{Brep, BrepDisplay, BrepTopology}, budget::Limits, geometry::Geometry,
    model::{Document, Entity}, transaction::{TransactionCoordinator, World},
};
use std::time::Instant;

#[test]
#[ignore = "manual release timing probe; not a correctness gate"]
fn source_capture_scaling() {
    let limits = Limits::default();
    for distinct in [false, true] {
        for count in [24usize, 64, 128] {
            let topology = BrepTopology { faces: vec![uuid::Uuid::from_u128(1)], edges: vec![], vertex_ids: vec![], face_edges: vec![] };
            let shape = |index: usize| Brep::from_native(format!("CASCADE Topology {}", if distinct { index } else { 0 }), BrepDisplay {
                source_sha256: [0; 32], deflection_mm: 0.1, points: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
                triangles: vec![[0,1,2,0]], segments: vec![], vertices: vec![], topology: topology.clone().into(),
            }, &limits).unwrap();
            let shared = shape(0);
            let mut document = Document::new("capture timing");
            let mut ids = Vec::new();
            for index in 0..count {
                let entity = Entity::new("object", Geometry::Brep(if distinct { shape(index) } else { shared.clone() }), None);
                ids.push(entity.id);
                document.entities.insert(entity.id, entity);
            }
            let doc = document.id;
            let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
            let stamps: Vec<_> = ids.iter().map(|id| coordinator.world().documents[&doc].stamp(*id).unwrap()).collect();
            let mut serial = Vec::new();
            let mut batched = Vec::new();
            for run in 0..5 {
                for use_batch in if run % 2 == 0 { [false, true] } else { [true, false] } {
                    let mut output = Vec::with_capacity(count);
                    let start = Instant::now();
                    if use_batch { coordinator.capture_brep_sources_into(doc, &ids, &mut output).unwrap(); }
                    else { for stamp in &stamps { output.push(coordinator.capture_brep_source(stamp).unwrap()); } }
                    let micros = start.elapsed().as_micros();
                    if use_batch { batched.push(micros); } else { serial.push(micros); }
                    assert_eq!(output.len(), count);
                    drop(output);
                }
            }
            serial.sort_unstable(); batched.sort_unstable();
            println!("distinct={distinct} count={count} serial_median_us={} serial_max_us={} batch_median_us={} batch_max_us={}",
                serial[2], serial[4], batched[2], batched[4]);
        }
    }
}
