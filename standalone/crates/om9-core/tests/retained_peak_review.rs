// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent allocator measurement of aggregate retained validation admission.
use om9_core::{
    budget::{Limits, Usage},
    geometry::{Cloud, Geometry, Unit},
    identity::{DocumentId, ImportNamespaceId, SourceId},
    model::{Document, Entity},
    resource::{Resource, ResourceKind},
    retained::{
        ImportMode, ImportNamespace, NativeRecordKey, NativeRecordKind, NativeUnits, ResolvedBy,
        RetainedSource, SourceRecord, SourceUnits,
    },
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static TEST_LOCK: Mutex<()> = Mutex::new(());
static PEAK: AtomicUsize = AtomicUsize::new(0);
fn allocated(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::SeqCst) + bytes;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
        unsafe { System.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() {
            LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
            allocated(size);
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn fixture(rows_first: bool, undo_snapshot: bool, limits: Limits) -> TransactionCoordinator {
    let mut records_document = Document::new("qualified records");
    records_document.id = DocumentId(uuid::Uuid::from_u128(if rows_first { 1 } else { 2 }));
    let record_id = records_document.id;
    let source_resource = Resource::from_bytes(
        "authored source",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"structural source fixture only",
        &Limits::default(),
    )
    .unwrap();
    let unit = NativeUnits {
        native_enum: 2,
        name: "Millimeters".into(),
        declared_mm_per_unit: Some(1.),
        millimeters_per_unit: 1.,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.001,
        angular_tolerance_radians: 0.001,
        relative_tolerance: 0.,
    };
    let source = RetainedSource {
        id: SourceId::new(),
        resource: source_resource.id,
        sha256: source_resource.sha256(),
        archive_version: 50,
        units: SourceUnits {
            model: unit.clone(),
            page: unit,
            definitions: Default::default(),
        },
    };
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: Some(ImportMode::Preserve),
        records: (1..=5000)
            .map(|n| {
                SourceRecord::retained(
                    NativeRecordKey {
                        kind: NativeRecordKind::Geometry,
                        id: uuid::Uuid::from_u128(n),
                        native_index: None,
                    },
                    "ON_Brep",
                )
            })
            .collect(),
    };
    records_document
        .resources
        .insert(source_resource.id, source_resource);
    records_document.retained_sources.insert(source.id, source);
    records_document
        .import_namespaces
        .insert(namespace.id, namespace);
    let mut payload_document = Document::new("other retained allocation");
    payload_document.id = DocumentId(uuid::Uuid::from_u128(if rows_first { 2 } else { 1 }));
    let resource = Resource::from_bytes(
        "owned peer payload",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        &vec![3; 3 * 1024 * 1024],
        &Limits::default(),
    )
    .unwrap();
    payload_document.resources.insert(resource.id, resource);
    let mut c = TransactionCoordinator::new(
        World::new(vec![records_document, payload_document], &limits).unwrap(),
        limits,
    )
    .unwrap();
    if undo_snapshot {
        let prepared = c
            .prepare(TransactionRequest::new(
                "context snapshot",
                vec![Mutation::Context {
                    document: record_id,
                    expected_revision: 1,
                    page_unit: Unit::Inch,
                }],
            ))
            .unwrap();
        c.commit(prepared).unwrap();
        c.undo().unwrap();
    }
    c
}

#[test]
fn validation_scratch_must_fit_alongside_other_retained_documents() {
    let _serial = TEST_LOCK.lock().unwrap();
    for rows_first in [true, false] {
        let before_fixture = LIVE.load(Ordering::SeqCst);
        let c = fixture(rows_first, false, Limits::default());
        let owned = c.retained_usage().unwrap();
        let world = c.world().clone();
        drop(c);
        let limits = Limits {
            max_bytes: owned.bytes + 8192,
            ..Limits::default()
        };
        PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
        // The corrected constructor may reject before its first validation scratch;
        // otherwise the same guarantee must hold during retained_usage itself.
        let result = TransactionCoordinator::new(world, limits).and_then(|c| c.retained_usage());
        let measured = PEAK.load(Ordering::SeqCst) - before_fixture;
        assert!(
            measured <= limits.max_bytes as usize,
            "retained validation allocated {measured} bytes with limit {}, result {result:?}",
            limits.max_bytes
        );
        assert!(matches!(result, Err(om9_core::Error::Budget("bytes"))));
    }
}
#[test]
fn held_reservations_cover_both_peer_orders_and_undo_snapshot_owners() {
    let _serial = TEST_LOCK.lock().unwrap();
    for rows_first in [true, false] {
        for history in [false, true] {
            let calibration = fixture(rows_first, history, Limits::default());
            let owned = calibration.retained_usage().unwrap();
            drop(calibration);
            let before_fixture = LIVE.load(Ordering::SeqCst);
            let limits = Limits {
                max_bytes: owned.bytes + 8 * 1024 * 1024,
                ..Limits::default()
            };
            let mut c = fixture(rows_first, history, limits);
            assert_eq!(c.retained_usage().unwrap().bytes, owned.bytes);
            let input = vec![3u8; 8 * 1024 * 1024 - 8192];
            let epoch = c.epoch();
            let redo = c.redo_len();
            PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
            let result = c.retained_usage_with_held_usage(Usage {
                bytes: input.capacity() as u64,
                ..Usage::default()
            });
            let measured = PEAK.load(Ordering::SeqCst) - before_fixture;
            assert!(
                measured <= limits.max_bytes as usize,
                "aggregate peak {measured}, limit {}, rows_first={rows_first}, history={history}",
                limits.max_bytes
            );
            assert!(matches!(result, Err(om9_core::Error::Budget("bytes"))));
            assert_eq!(c.epoch(), epoch);
            assert_eq!(c.redo_len(), redo);
            drop(input);
            if history {
                c.redo().unwrap();
                let id = DocumentId(uuid::Uuid::from_u128(if rows_first { 1 } else { 2 }));
                assert_eq!(c.world().documents[&id].page_unit, Unit::Inch);
            }
        }
    }
}
#[test]
fn mutation_input_remains_reserved_during_initial_coordinator_validation() {
    let _serial = TEST_LOCK.lock().unwrap();
    for rows_first in [true, false] {
        let calibration = fixture(rows_first, true, Limits::default());
        let owned = calibration.retained_usage().unwrap();
        drop(calibration);
        let before_fixture = LIVE.load(Ordering::SeqCst);
        let limits = Limits {
            max_bytes: owned.bytes + 8 * 1024 * 1024,
            ..Limits::default()
        };
        let c = fixture(rows_first, true, limits);
        let id = DocumentId(uuid::Uuid::from_u128(if rows_first { 1 } else { 2 }));
        let input = Entity::new(
            "owned incoming cloud",
            Geometry::Cloud(Cloud {
                points: vec![
                    [1., 2., 3.];
                    (8 * 1024 * 1024 - 8192) / std::mem::size_of::<[f64; 3]>()
                ],
                normals: vec![],
                colors: vec![],
                hidden: vec![],
            }),
            None,
        );
        let request = TransactionRequest::new(
            "reject before scratch",
            vec![Mutation::Insert {
                document: id,
                entity: input,
            }],
        );
        let epoch = c.epoch();
        let redo = c.redo_len();
        PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
        let result = c.prepare(request);
        let measured = PEAK.load(Ordering::SeqCst) - before_fixture;
        assert!(
            measured <= limits.max_bytes as usize,
            "request plus retained peak {measured}, limit {}",
            limits.max_bytes
        );
        assert!(matches!(result, Err(om9_core::Error::Budget("bytes"))));
        assert_eq!(c.epoch(), epoch);
        assert_eq!(c.redo_len(), redo);
    }
}
