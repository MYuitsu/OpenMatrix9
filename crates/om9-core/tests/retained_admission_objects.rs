// SPDX-License-Identifier: LGPL-2.1-or-later
//! Authored canonical records exercise ownership/accounting, not OCCT validity.
use om9_core::{
    Error,
    brep::{Brep, BrepDisplay, BrepTopology, CanonicalBrep},
    budget::{Limits, Usage},
    geometry::{Geometry, Transform},
    model::{Document, Entity},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use std::time::Instant;

fn source(index: usize, limits: &Limits) -> CanonicalBrep {
    Brep::from_native(
        format!("CASCADE Topology authored accounting record {index}"),
        BrepDisplay {
            source_sha256: [0; 32],
            deflection_mm: 0.1,
            points: vec![[0.; 3], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2, 0]],
            segments: vec![],
            vertices: vec![],
            topology: BrepTopology {
                faces: vec![uuid::Uuid::from_u128(index as u128 + 1)],
                edges: vec![],
                vertex_ids: vec![],
                face_edges: vec![],
            }
            .into(),
        },
        limits,
    )
    .unwrap()
    .into_canonical_and_representation()
    .0
}

fn history(count: usize) -> TransactionCoordinator {
    let limits = Limits {
        max_objects: 1_000_000,
        max_bytes: 16 * 1024 * 1024 * 1024,
        ..Limits::default()
    };
    let document = Document::new("retained object admission");
    let id = document.id;
    let mut c =
        TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    for index in 0..count {
        let entity = Entity::new("body", Geometry::Brep(source(index, &limits)), None);
        let prepared = c
            .prepare(TransactionRequest::new(
                "insert",
                vec![Mutation::Insert {
                    document: id,
                    entity,
                }],
            ))
            .unwrap();
        c.commit(prepared).unwrap();
    }
    assert_eq!(c.undo_len(), count);
    assert_eq!(c.redo_len(), 0);
    c
}

#[test]
fn duplicate_document_bound_keeps_real_object_byte_guards_and_history() {
    let count = 32;
    let mut c = history(count);
    let epoch = c.epoch();
    let world = c.world().clone();
    let ordinary = c.retained_usage().unwrap();
    assert_eq!(ordinary.objects, (count * (count + 1) / 2) as u64);
    // Exact owners fit, whereas counting before/after Arcs twice does not.
    let remaining_objects = ordinary.objects + count as u64;
    let held = Usage {
        objects: c.limits().max_objects - remaining_objects,
        ..Usage::default()
    };
    let bounded = c.retained_usage_with_held_usage(held).unwrap();
    assert_eq!(bounded.bytes, ordinary.bytes);
    assert_eq!(bounded.objects, ordinary.objects + held.objects);
    let over = Usage {
        objects: c.limits().max_objects - ordinary.objects + 1,
        ..Usage::default()
    };
    assert!(matches!(
        c.retained_usage_with_held_usage(over),
        Err(Error::Budget("objects"))
    ));
    assert!(matches!(
        c.retained_usage_with_held_usage(Usage {
            bytes: c.limits().max_bytes,
            ..Usage::default()
        }),
        Err(Error::Budget("bytes"))
    ));
    assert_eq!(c.epoch(), epoch);
    assert_eq!(c.world(), &world);
    assert_eq!(c.undo_len(), count);
    assert_eq!(c.redo_len(), 0);
    c.undo().unwrap();
    assert_eq!(
        c.world().documents.values().next().unwrap().entities.len(),
        count - 1
    );
    c.redo().unwrap();
    assert_eq!(c.world(), &world);
}

#[test]
fn newest_snapshot_lookup_keeps_old_sources_and_independent_equal_allocations_charged() {
    let mut c = history(8);
    let document = *c.world().documents.keys().next().unwrap();
    let ids = c.world().documents[&document]
        .entities
        .keys()
        .copied()
        .collect::<Vec<_>>();
    let mut captures = Vec::with_capacity(ids.len());
    c.capture_brep_sources_into(document, &ids, &mut captures)
        .unwrap();
    let source_bytes = captures
        .iter()
        .map(|source| source.canonical_storage_bytes().unwrap())
        .sum::<u64>();
    let duplicates = captures.clone();
    let mut mutations = ids
        .iter()
        .map(|id| Mutation::Delete {
            source: c.world().documents[&document].stamp(*id).unwrap(),
        })
        .collect::<Vec<_>>();
    // Equal source bytes/topology still have independent owning allocations.
    let replacement = Entity::new(
        "independent equal source",
        Geometry::Brep(source(0, c.limits())),
        None,
    );
    mutations.push(Mutation::Insert {
        document,
        entity: replacement,
    });
    let prepared = c
        .prepare(TransactionRequest::new("replace", mutations))
        .unwrap();
    c.commit(prepared).unwrap();
    assert_eq!(c.world().documents[&document].entities.len(), 1);
    c.retained_usage().unwrap();
    c.undo().unwrap();
    assert_eq!(c.world().documents[&document].entities.len(), 8);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&document].entities.len(), 1);
    let output = *c.world().documents[&document]
        .entities
        .keys()
        .next()
        .unwrap();
    let prepared = c
        .prepare(TransactionRequest::new(
            "move result",
            vec![Mutation::Placement {
                source: c.world().documents[&document].stamp(output).unwrap(),
                placement: Transform::translation([1., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(prepared).unwrap();
    while c.undo_len() > 1 {
        assert!(c.discard_oldest_undo().unwrap());
    }
    assert_eq!(c.undo_len(), 1);
    assert_eq!(c.redo_len(), 0);
    let detached = c.retained_usage().unwrap().bytes;
    drop(duplicates);
    assert_eq!(c.retained_usage().unwrap().bytes, detached);
    assert!(
        captures
            .iter()
            .all(|source| source.source().starts_with("CASCADE Topology authored"))
    );
    drop(captures);
    assert_eq!(detached - c.retained_usage().unwrap().bytes, source_bytes);
}

#[test]
#[ignore = "manual release timing probe; no time-based acceptance assertion"]
fn profile_512_retained_versions_at_200k_object_ceiling() {
    let started = Instant::now();
    let c = history(512);
    let setup_s = started.elapsed().as_secs_f64();
    let start = Instant::now();
    let ordinary = c.retained_usage().unwrap();
    let ordinary_ms = start.elapsed().as_secs_f64() * 1000.;
    let held = Usage {
        objects: c.limits().max_objects - 200_000,
        ..Usage::default()
    };
    let start = Instant::now();
    let bounded = c.retained_usage_with_held_usage(held).unwrap();
    let bounded_ms = start.elapsed().as_secs_f64() * 1000.;
    assert_eq!(ordinary.objects, 131328);
    assert_eq!(bounded.objects, ordinary.objects + held.objects);
    assert_eq!(bounded.bytes, ordinary.bytes);
    assert_eq!(c.undo_len(), 512);
    let evidence = serde_json::json!({"fixture":"authored canonical ownership only","undo":512,
        "retained_object_slots":ordinary.objects,"remaining_max_objects":200000,"setup_s":setup_s,
        "ordinary_ms":ordinary_ms,"bounded_ms":bounded_ms,"retained_bytes":ordinary.bytes});
    println!("{evidence}");
    if let Some(path) = std::env::var_os("OM9_RETAINED_PROBE_RESULT") {
        std::fs::write(path, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
    }
}
