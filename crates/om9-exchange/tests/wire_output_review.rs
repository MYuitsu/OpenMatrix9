// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
//! Independent wire-map tests; zero source namespaces start no native child.
use om9_core::{
    Error,
    budget::{Limits, Usage},
    geometry::Geometry,
    identity::SourceStamp,
    model::{Definition, Document, Entity, Layer},
    retained::HostRef,
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{ExportCapture, NativeExchange};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    io::Read,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use uuid::Uuid;

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TEST: Mutex<()> = Mutex::new(());
fn added(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::SeqCst) + bytes;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc(layout) };
        if !result.is_null() {
            added(layout.size());
        }
        result
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, old: Layout, new: usize) -> *mut u8 {
        let result = unsafe { System.realloc(ptr, old, new) };
        if !result.is_null() {
            if new >= old.size() {
                added(new - old.size());
            } else {
                LIVE.fetch_sub(old.size() - new, Ordering::SeqCst);
            }
        }
        result
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn exchange() -> NativeExchange {
    let path = std::env::current_exe().unwrap();
    let mut file = std::fs::File::open(&path).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let read = file.read(&mut buffer).unwrap();
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    NativeExchange::new(
        NativeProgram::admit(&path, hash.finalize().into()).unwrap(),
        ProcessLimits::default(),
    )
    .unwrap()
}
fn coordinator(document: Document, limits: Limits) -> TransactionCoordinator {
    TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap()
}

#[test]
fn additions_follow_host_layer_order_and_definition_members_have_no_native_indices() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let mut d = Document::new("d");
    let mut parent = Layer::new("parent", [0, 0, 0, 255]);
    let child = Layer::new("child", [1, 2, 3, 255]);
    parent.id.0 = Uuid::from_u128(200);
    let mut child = child;
    child.id.0 = Uuid::from_u128(100);
    child.parent = Some(parent.id);
    d.layer_order = vec![parent.id, child.id];
    let parent_id = parent.id;
    let child_id = child.id;
    d.layers.insert(parent.id, parent);
    d.layers.insert(child.id, child);
    let member = Entity::new("member", Geometry::Point([1., 2., 3.]), Some(child_id));
    let member_id = member.id;
    let def = Definition::new("def", vec![member]);
    let def_id = def.id;
    let root = Entity::new("root", Geometry::Instance(def_id), Some(parent_id));
    let root_id = root.id;
    d.definitions.insert(def_id, def);
    d.entities.insert(root_id, root);
    let id = d.id;
    let c = coordinator(d, Limits::default());
    let capture = ExportCapture::all(&c, id).unwrap();
    let original = Cancellation::default();
    let prepared = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &std::env::temp_dir(),
            &original,
            Usage::default(),
        )
        .unwrap();
    assert!(prepared.native_stats().is_empty());
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let parent = wire.host(HostRef::Layer(parent_id)).unwrap();
    let child = wire.host(HostRef::Layer(child_id)).unwrap();
    assert_eq!(wire.table_order(5), &[parent.id, child.id]);
    for (row, ordinal) in [(parent, 0), (child, 1)] {
        assert_eq!(row.serialized_index, Some(ordinal));
        assert_eq!(row.archive_reference_index, Some(ordinal));
        assert_eq!(row.sdk_destination_index, Some(ordinal));
    }
    let definition = wire.host(HostRef::Definition(def_id)).unwrap();
    assert_eq!(definition.serialized_index, None);
    assert_eq!(definition.archive_reference_index, Some(0));
    assert_eq!(definition.sdk_destination_index, Some(0));
    let member = wire
        .host(HostRef::DefinitionMember {
            definition: def_id,
            object: member_id,
        })
        .unwrap();
    let root = wire.host(HostRef::Object(root_id)).unwrap();
    for row in [root, member] {
        assert_eq!(row.native_component_type, 12);
        assert_eq!(row.serialized_index, None);
        assert_eq!(row.archive_reference_index, None);
        assert_eq!(row.sdk_destination_index, None);
    }
    assert_eq!(wire.table_order(11), &[definition.id]);
    assert_eq!(wire.table_order(12).len(), 2);
    let mut ids = vec![parent.id, child.id, definition.id, member.id, root.id];
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 5);
    assert!(!ids.contains(&parent_id.0) && !ids.contains(&child_id.0));
    assert!(wire.default_layer().is_none());
}

#[test]
fn generated_layer_avoids_occupied_name_has_no_host_binding_and_keeps_original_cancel() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let mut d = Document::new("d");
    let layer = Layer::new("om9 unassigned", [0, 0, 0, 255]);
    let layer_id = layer.id;
    d.layer_order.push(layer_id);
    d.layers.insert(layer_id, layer);
    let root = Entity::new("root", Geometry::Point([1., 0., 0.]), None);
    d.entities.insert(root.id, root);
    let id = d.id;
    let mut c = coordinator(d, Limits::default());
    let capture = ExportCapture::all(&c, id).unwrap();
    let original = Cancellation::default();
    let prepared = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &std::env::temp_dir(),
            &original,
            Usage::default(),
        )
        .unwrap();
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let default = wire.default_layer().unwrap();
    assert!(
        !wire
            .default_layer_name()
            .unwrap()
            .eq_ignore_ascii_case("om9 unassigned")
    );
    assert_eq!(default.serialized_index, Some(1));
    assert_eq!(default.archive_reference_index, Some(1));
    assert_eq!(default.sdk_destination_index, Some(1));
    assert!(
        wire.host(HostRef::Layer(om9_core::identity::LayerId(default.id)))
            .is_none()
    );
    assert_eq!(wire.table_order(5)[1], default.id);
    original.cancel();
    assert!(matches!(
        wire.revalidate(&c, &Cancellation::default()),
        Err(Error::Canceled)
    ));
    assert!(matches!(
        prepared.output_wire_plan(&c, &Cancellation::default(), Usage::default()),
        Err(Error::Canceled)
    ));
    let fresh = Cancellation::default();
    let clean = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &std::env::temp_dir(),
            &fresh,
            Usage::default(),
        )
        .unwrap();
    let clean_wire = clean
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    c.close_document(id).unwrap();
    assert!(clean_wire.revalidate(&c, &Cancellation::default()).is_err());
}

#[test]
fn generated_native_layer_name_obeys_the_configured_name_limit() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    for (max_name_bytes, occupied_base) in [(8, false), (14, true)] {
        let mut d = Document::new("d");
        if occupied_base {
            let layer = Layer::new("OM9 Unassigned", [0, 0, 0, 255]);
            d.layer_order.push(layer.id);
            d.layers.insert(layer.id, layer);
            let root = Entity::new("r", Geometry::Point([0., 0., 0.]), None);
            d.entities.insert(root.id, root);
        }
        let id = d.id;
        let c = coordinator(
            d,
            Limits {
                max_name_bytes,
                ..Limits::default()
            },
        );
        let capture = ExportCapture::all(&c, id).unwrap();
        let prepared = exchange()
            .prepare_merge_inputs(
                &c,
                &capture,
                None,
                &std::env::temp_dir(),
                &Cancellation::default(),
                Usage::default(),
            )
            .unwrap();
        let result = prepared.output_wire_plan(&c, &Cancellation::default(), Usage::default());
        assert!(
            matches!(result, Err(Error::Budget("name bytes"))),
            "generated name must be admitted before ownership growth: max={max_name_bytes}, collision={occupied_base}",
        );
    }
}

#[test]
fn wire_map_peak_includes_retained_coordinator_prepared_inputs_stamps_and_external_owner() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let mut d = Document::new("d");
    for _ in 0..256 {
        let layer = Layer::new("l", [0, 0, 0, 255]);
        d.layer_order.push(layer.id);
        d.layers.insert(layer.id, layer);
        let member = Entity::new("m", Geometry::Point([1., 2., 3.]), None);
        let definition = Definition::new("def", vec![member]);
        let root = Entity::new("r", Geometry::Instance(definition.id), None);
        d.entities.insert(root.id, root);
        d.definitions.insert(definition.id, definition);
    }
    let id = d.id;
    let c = coordinator(d, Limits::default());
    let capture = ExportCapture::all(&c, id).unwrap();
    let prepared = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &std::env::temp_dir(),
            &Cancellation::default(),
            Usage::default(),
        )
        .unwrap();
    let limit = c.limits().max_bytes;
    let mut held = c.retained_usage().unwrap();
    held.add(prepared.owned_usage(), c.limits()).unwrap();
    held.add(
        Usage::array::<SourceStamp>(capture.document().entities.len()).unwrap(),
        c.limits(),
    )
    .unwrap();
    let mut accepted = 0;
    let mut refused = limit;
    while refused - accepted > 1 {
        let middle = accepted + (refused - accepted) / 2;
        match prepared.output_wire_plan(
            &c,
            &Cancellation::default(),
            Usage {
                bytes: middle,
                ..Usage::default()
            },
        ) {
            Ok(_) => accepted = middle,
            Err(Error::Budget(_)) => refused = middle,
            Err(error) => panic!("unexpected boundary error: {error:?}"),
        }
    }
    for external in [accepted, refused] {
        let baseline = LIVE.load(Ordering::SeqCst);
        PEAK.store(baseline, Ordering::SeqCst);
        let result = prepared.output_wire_plan(
            &c,
            &Cancellation::default(),
            Usage {
                bytes: external,
                ..Usage::default()
            },
        );
        let peak = PEAK.load(Ordering::SeqCst).saturating_sub(baseline) as u64;
        assert!(
            held.bytes + external + peak <= limit,
            "held={}, external={external}, new_peak={peak}, limit={limit}, admitted={}",
            held.bytes,
            result.is_ok()
        );
        assert_eq!(result.is_ok(), external == accepted);
    }
    prepared.revalidate(&c, &Cancellation::default()).unwrap();
}
