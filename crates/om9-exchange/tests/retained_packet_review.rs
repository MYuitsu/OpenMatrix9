// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
//! Independent request-encoder checks. Zero source namespaces launch no child.
use om9_core::{
    Error,
    budget::{Limits, Usage},
    geometry::{Cloud, Geometry, Transform},
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
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TEST: Mutex<()> = Mutex::new(());
static CANCEL_AT_ALLOCATION: Mutex<Option<Cancellation>> = Mutex::new(None);
static TARGET_SIZE: AtomicUsize = AtomicUsize::new(0);
static ARMED: AtomicBool = AtomicBool::new(false);
static FIRED: AtomicBool = AtomicBool::new(false);
fn added(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::SeqCst) + bytes;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == TARGET_SIZE.load(Ordering::SeqCst)
            && ARMED.swap(false, Ordering::SeqCst)
        {
            FIRED.store(true, Ordering::SeqCst);
            CANCEL_AT_ALLOCATION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .cancel();
        }
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
fn coordinator(document: Document) -> TransactionCoordinator {
    let l = Limits::default();
    TransactionCoordinator::new(World::new(vec![document], &l).unwrap(), l).unwrap()
}

#[test]
fn nested_member_and_layer_relations_use_output_uuids_with_explicit_current_layer() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let mut d = Document::new("independent nested packet");
    let parent = Layer::new("parent", [1, 2, 3, 255]);
    let mut child = Layer::new("child", [4, 5, 6, 255]);
    child.parent = Some(parent.id);
    let (parent_layer, child_layer) = (parent.id, child.id);
    d.layer_order = vec![parent_layer, child_layer];
    d.layers.insert(parent.id, parent);
    d.layers.insert(child.id, child);
    d.current_layer = Some(child_layer);
    let leaf_member = Entity::new("leaf member", Geometry::Point([1., 2., 3.]), None);
    let leaf_member_id = leaf_member.id;
    let leaf = Definition::new("leaf", vec![leaf_member]);
    let leaf_id = leaf.id;
    let mut nested = Entity::new("nested", Geometry::Instance(leaf_id), Some(parent_layer));
    nested.placement = Transform::translation([7., 8., 9.]);
    let nested_id = nested.id;
    let parent_definition = Definition::new("parent definition", vec![nested]);
    let parent_definition_id = parent_definition.id;
    let mut root = Entity::new(
        "root",
        Geometry::Instance(parent_definition_id),
        Some(child_layer),
    );
    root.placement = Transform([
        [2., 0., 0., 10.],
        [0., 3., 0., 20.],
        [0., 0., -4., 30.],
        [0., 0., 0., 1.],
    ]);
    let root_id = root.id;
    d.definitions.insert(leaf_id, leaf);
    d.definitions
        .insert(parent_definition_id, parent_definition);
    d.entities.insert(root_id, root);
    let id = d.id;
    let c = coordinator(d);
    let before = c.snapshot();
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
    assert!(prepared.native_stats().is_empty());
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let bytes = wire
        .encode_retained_request(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let packet: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let rows = packet["additions"].as_array().unwrap();
    let row = |host| {
        let output = wire.host(host).unwrap();
        rows.iter()
            .find(|row| row["output"]["id"] == output.id.to_string())
            .unwrap()
    };
    let leaf = wire.host(HostRef::Definition(leaf_id)).unwrap();
    let parent = wire
        .host(HostRef::Definition(parent_definition_id))
        .unwrap();
    let nested_host = HostRef::DefinitionMember {
        definition: parent_definition_id,
        object: nested_id,
    };
    let member_host = HostRef::DefinitionMember {
        definition: leaf_id,
        object: leaf_member_id,
    };
    assert_eq!(
        row(HostRef::Object(root_id))["current"]["Entity"]["geometry"]["Instance"],
        parent.id.to_string()
    );
    assert_eq!(
        row(nested_host)["current"]["Entity"]["geometry"]["Instance"],
        leaf.id.to_string()
    );
    assert_eq!(
        row(HostRef::Definition(parent_definition_id))["current"]["Definition"]["members"],
        serde_json::json!([wire.host(nested_host).unwrap().id])
    );
    assert_eq!(
        row(HostRef::Definition(leaf_id))["current"]["Definition"]["members"],
        serde_json::json!([wire.host(member_host).unwrap().id])
    );
    assert_eq!(
        row(HostRef::Layer(child_layer))["current"]["Layer"]["parent"],
        wire.host(HostRef::Layer(parent_layer))
            .unwrap()
            .id
            .to_string()
    );
    assert_eq!(
        row(member_host)["current"]["Entity"]["attributes"]["layer_index"],
        wire.default_layer()
            .unwrap()
            .archive_reference_index
            .unwrap()
    );
    assert_eq!(
        packet["document"]["current_layer"]["policy"],
        "explicit_output"
    );
    assert_eq!(
        packet["document"]["current_layer"]["id"],
        wire.host(HostRef::Layer(child_layer))
            .unwrap()
            .id
            .to_string()
    );
    assert_eq!(
        row(HostRef::Object(root_id))["current"]["Entity"]["placement"][2][2],
        -4.
    );
    for host_id in [
        parent_layer.0,
        child_layer.0,
        leaf_id.0,
        parent_definition_id.0,
        leaf_member_id.0,
        nested_id.0,
        root_id.0,
    ] {
        assert!(
            !std::str::from_utf8(&bytes)
                .unwrap()
                .contains(&host_id.to_string())
        );
    }
    assert_eq!(c.snapshot(), before);
}

#[test]
fn implicit_current_layer_chooses_first_live_table_or_generated_unassigned() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    for populated in [false, true] {
        let mut d = Document::new("implicit layer");
        if populated {
            let layer = Layer::new("first", [0, 0, 0, 255]);
            let root = Entity::new("r", Geometry::Point([0., 0., 0.]), Some(layer.id));
            d.entities.insert(root.id, root);
            d.layer_order.push(layer.id);
            d.layers.insert(layer.id, layer);
        }
        let id = d.id;
        let c = coordinator(d);
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
        let wire = prepared
            .output_wire_plan(&c, &Cancellation::default(), Usage::default())
            .unwrap();
        let packet: serde_json::Value = serde_json::from_slice(
            &wire
                .encode_retained_request(&c, &Cancellation::default(), Usage::default())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            packet["document"]["current_layer"]["policy"],
            "explicit_output"
        );
        assert_eq!(
            packet["document"]["current_layer"]["id"],
            wire.table_order(5)[0].to_string()
        );
        assert_eq!(
            packet["document"]["current_layer"]["archive_reference_index"],
            0
        );
        assert_eq!(wire.default_layer().is_some(), !populated);
    }
}

#[test]
fn original_and_fresh_cancellation_at_packet_allocation_discard_private_encoded_output() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    for original_token in [true, false] {
        let mut d = Document::new("controlled packet cancellation");
        for _ in 0..20 {
            let root = Entity::new("x".repeat(1024), Geometry::Point([0., 0., 0.]), None);
            d.entities.insert(root.id, root);
        }
        let id = d.id;
        let c = coordinator(d);
        let before = c.snapshot();
        let capture = ExportCapture::all(&c, id).unwrap();
        let original = Cancellation::default();
        let fresh = Cancellation::default();
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
            .output_wire_plan(&c, &fresh, Usage::default())
            .unwrap();
        let size = wire
            .encode_retained_request(&c, &fresh, Usage::default())
            .unwrap()
            .len();
        assert!(size > 8192);
        *CANCEL_AT_ALLOCATION.lock().unwrap() = Some(if original_token {
            original.clone()
        } else {
            fresh.clone()
        });
        TARGET_SIZE.store(size, Ordering::SeqCst);
        FIRED.store(false, Ordering::SeqCst);
        ARMED.store(true, Ordering::SeqCst);
        let result = wire.encode_retained_request(&c, &fresh, Usage::default());
        ARMED.store(false, Ordering::SeqCst);
        *CANCEL_AT_ALLOCATION.lock().unwrap() = None;
        assert!(
            FIRED.load(Ordering::SeqCst),
            "packet reserve must trigger the controlled Cancel"
        );
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(c.snapshot(), before);
    }
}

#[test]
fn aggregate_packet_peak_retains_all_prior_owners_and_bounds_refusal_allocations() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let mut d = Document::new("packet peak");
    for _ in 0..64 {
        let root = Entity::new(
            format!("{}\"", "x".repeat(8191)),
            Geometry::Point([1., 2., 3.]),
            None,
        );
        d.entities.insert(root.id, root);
    }
    let id = d.id;
    let c = coordinator(d);
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
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let mut held = c.retained_usage().unwrap();
    held.add(prepared.owned_usage(), c.limits()).unwrap();
    held.add(wire.owned_usage(), c.limits()).unwrap();
    held.add(
        Usage::array::<SourceStamp>(capture.document().entities.len()).unwrap(),
        c.limits(),
    )
    .unwrap();
    let limit = c.limits().max_bytes;
    let mut accepted = 0;
    let mut refused = limit;
    while refused - accepted > 1 {
        let middle = accepted + (refused - accepted) / 2;
        match wire.encode_retained_request(
            &c,
            &Cancellation::default(),
            Usage {
                bytes: middle,
                ..Usage::default()
            },
        ) {
            Ok(_) => accepted = middle,
            Err(Error::Budget(_)) => refused = middle,
            Err(error) => panic!("unexpected admission boundary: {error:?}"),
        }
    }
    for external in [accepted, refused] {
        let baseline = LIVE.load(Ordering::SeqCst);
        PEAK.store(baseline, Ordering::SeqCst);
        let result = wire.encode_retained_request(
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
        if let Ok(packet) = result {
            assert!(packet.capacity() as u64 <= (limit - held.bytes - external) / 2);
            assert_eq!(packet.len(), packet.capacity());
        }
    }
    wire.revalidate(&c, &Cancellation::default()).unwrap();
}

#[test]
fn packet_cap_refuses_a_large_cloud_without_allocating_the_output_buffer() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let mut d = Document::new("counted cloud refusal");
    let root = Entity::new(
        "cloud",
        Geometry::Cloud(Cloud {
            points: vec![[123456789.12345678, 123456789.12345678, 123456789.12345678]; 400_000],
            normals: vec![],
            colors: vec![],
            hidden: vec![],
        }),
        None,
    );
    d.entities.insert(root.id, root);
    let id = d.id;
    let c = coordinator(d);
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
    let wire = prepared
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let baseline = LIVE.load(Ordering::SeqCst);
    PEAK.store(baseline, Ordering::SeqCst);
    let result = wire.encode_retained_request(&c, &Cancellation::default(), Usage::default());
    let peak = PEAK.load(Ordering::SeqCst).saturating_sub(baseline);
    assert!(matches!(
        result,
        Err(Error::Budget("retained request bytes"))
    ));
    assert!(
        peak < 64 * 1024,
        "counter refusal must precede large packet allocation: peak={peak}"
    );
    wire.revalidate(&c, &Cancellation::default()).unwrap();
}
