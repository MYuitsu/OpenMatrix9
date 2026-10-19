// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]
//! Independent H-drive input staging checks. No native child is launched.
use om9_core::{
    Error,
    budget::{Limits, Usage},
    geometry::Geometry,
    identity::SourceStamp,
    model::{Document, Entity},
    transaction::{Cancellation, TransactionCoordinator, World},
};
use om9_exchange::{ExportCapture, NativeExchange};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs,
    io::Read,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TEST: Mutex<()> = Mutex::new(());
static CANCEL_PARENT: Mutex<Option<PathBuf>> = Mutex::new(None);
static CANCEL_TOKEN: Mutex<Option<Cancellation>> = Mutex::new(None);
static EXPECTED_BYTES: AtomicUsize = AtomicUsize::new(0);
static OBSERVED_BYTES: AtomicUsize = AtomicUsize::new(0);
static ARMED: AtomicBool = AtomicBool::new(false);
static FIRED: AtomicBool = AtomicBool::new(false);

// Turn the probe off while inspecting the private directory: these test-only
// filesystem calls allocate too. Cancel only after the complete request was
// materialized, before acceptance of its held read owner.
fn cancel_after_materialization() {
    if !ARMED.swap(false, Ordering::SeqCst) {
        return;
    }
    let mut observed = None;
    if let Some(parent) = CANCEL_PARENT.lock().unwrap().as_ref()
        && let Ok(entries) = fs::read_dir(parent)
    {
        for entry in entries.flatten() {
            if let Ok(metadata) = fs::metadata(entry.path().join("request.json"))
                && metadata.is_file()
                && metadata.len() == EXPECTED_BYTES.load(Ordering::SeqCst) as u64
            {
                observed = Some(metadata.len());
                break;
            }
        }
    }
    if let Some(bytes) = observed {
        OBSERVED_BYTES.store(bytes as usize, Ordering::SeqCst);
        FIRED.store(true, Ordering::SeqCst);
        CANCEL_TOKEN.lock().unwrap().as_ref().unwrap().cancel();
    } else {
        ARMED.store(true, Ordering::SeqCst);
    }
}
fn added(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::SeqCst) + bytes;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        cancel_after_materialization();
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
        cancel_after_materialization();
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
    let mut file = fs::File::open(&path).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
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
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let parent = root.join("build/standalone-validation");
        fs::create_dir_all(&parent).unwrap();
        let parent = parent.canonicalize().unwrap();
        assert!(parent.starts_with(&root));
        let prefix = parent.components().next().unwrap();
        assert!(matches!(
            prefix,
            std::path::Component::Prefix(p)
                if matches!(p.kind(), std::path::Prefix::Disk(b'H') | std::path::Prefix::VerbatimDisk(b'H'))
        ));
        let path = parent.join(format!("retained-stage-review-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn empty(&self) -> bool {
        fs::read_dir(&self.0).unwrap().next().is_none()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

#[test]
fn request_sha_matches_guarded_bytes_and_handle_denies_rename_or_replacement() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let temp = Temp::new();
    let mut d = Document::new("guarded actual packet");
    let root = Entity::new(
        "quote \" and Unicode Việt",
        Geometry::Point([1., 2., 3.]),
        None,
    );
    d.entities.insert(root.id, root);
    let id = d.id;
    let c = coordinator(d);
    let capture = ExportCapture::all(&c, id).unwrap();
    let inputs = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &temp.0,
            &Cancellation::default(),
            Usage::default(),
        )
        .unwrap();
    assert!(inputs.native_stats().is_empty());
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let expected = wire
        .encode_retained_request(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let stage = wire
        .stage_retained_request(
            &c,
            &temp.0,
            &Cancellation::default(),
            Usage::array::<u8>(expected.capacity()).unwrap(),
        )
        .unwrap();
    let actual = fs::read(stage.request_path()).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(
        stage.request_sha256(),
        <[u8; 32]>::from(Sha256::digest(&actual))
    );
    assert!(!stage.output_path().exists());
    assert_eq!(stage.request_path().file_name().unwrap(), "request.json");
    let renamed = temp.0.join("renamed-request.json");
    assert!(fs::rename(stage.request_path(), &renamed).is_err());
    assert!(!renamed.exists());
    let replacement = temp.0.join("replacement.json");
    fs::write(
        &replacement,
        b"replacement must remain outside the staged owner",
    )
    .unwrap();
    assert!(fs::rename(&replacement, stage.request_path()).is_err());
    assert!(replacement.exists());
    assert_eq!(fs::read(stage.request_path()).unwrap(), expected);
    stage.revalidate(&c, &Cancellation::default()).unwrap();
    let private = stage.request_path().parent().unwrap().to_owned();
    drop(stage);
    assert!(!private.exists());
    assert_eq!(
        fs::read(&replacement).unwrap(),
        b"replacement must remain outside the staged owner"
    );
    fs::remove_file(replacement).unwrap();
    assert!(temp.empty());
}

#[test]
fn equal_counter_foreign_authority_and_closed_target_reject_without_changing_staged_bytes() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let temp = Temp::new();
    let d = Document::new("stage authority");
    let id = d.id;
    let mut c = coordinator(d);
    let capture = ExportCapture::all(&c, id).unwrap();
    let inputs = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &temp.0,
            &Cancellation::default(),
            Usage::default(),
        )
        .unwrap();
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let stage = wire
        .stage_retained_request(&c, &temp.0, &Cancellation::default(), Usage::default())
        .unwrap();
    let foreign = TransactionCoordinator::new(c.world().clone(), *c.limits()).unwrap();
    assert_eq!(foreign.epoch(), c.epoch());
    assert!(std::sync::Arc::ptr_eq(
        &foreign.world().documents[&id],
        &c.world().documents[&id]
    ));
    assert!(
        stage
            .revalidate(&foreign, &Cancellation::default())
            .is_err()
    );
    stage.revalidate(&c, &Cancellation::default()).unwrap();
    c.close_document(id).unwrap();
    assert!(stage.revalidate(&c, &Cancellation::default()).is_err());
    assert_eq!(
        stage.request_sha256(),
        <[u8; 32]>::from(Sha256::digest(fs::read(stage.request_path()).unwrap()))
    );
    drop(stage);
    assert!(temp.empty());
}

#[test]
fn cancellation_after_request_materialization_closes_created_inputs_and_removes_private_directory()
{
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    for cancel_original in [false, true] {
        let temp = Temp::new();
        let mut d = Document::new("post-write cancellation");
        for _ in 0..20 {
            let root = Entity::new("x".repeat(1024), Geometry::Point([1., 2., 3.]), None);
            d.entities.insert(root.id, root);
        }
        let id = d.id;
        let c = coordinator(d);
        let before = c.snapshot();
        let capture = ExportCapture::all(&c, id).unwrap();
        let original = Cancellation::default();
        let creation = Cancellation::default();
        let inputs = exchange()
            .prepare_merge_inputs(&c, &capture, None, &temp.0, &original, Usage::default())
            .unwrap();
        let wire = inputs
            .output_wire_plan(&c, &Cancellation::default(), Usage::default())
            .unwrap();
        let expected = wire
            .encode_retained_request(&c, &Cancellation::default(), Usage::default())
            .unwrap()
            .len();
        *CANCEL_PARENT.lock().unwrap() = Some(temp.0.clone());
        *CANCEL_TOKEN.lock().unwrap() = Some(if cancel_original {
            original.clone()
        } else {
            creation.clone()
        });
        EXPECTED_BYTES.store(expected, Ordering::SeqCst);
        OBSERVED_BYTES.store(0, Ordering::SeqCst);
        FIRED.store(false, Ordering::SeqCst);
        ARMED.store(true, Ordering::SeqCst);
        let result = wire.stage_retained_request(&c, &temp.0, &creation, Usage::default());
        ARMED.store(false, Ordering::SeqCst);
        *CANCEL_PARENT.lock().unwrap() = None;
        *CANCEL_TOKEN.lock().unwrap() = None;
        assert!(
            FIRED.load(Ordering::SeqCst),
            "Cancel must occur after complete file materialization"
        );
        assert_eq!(OBSERVED_BYTES.load(Ordering::SeqCst), expected);
        assert!(matches!(result, Err(Error::Canceled)));
        assert!(temp.empty());
        assert_eq!(c.snapshot(), before);
    }
}

#[test]
fn stage_packet_allocation_peak_is_bounded_with_all_prior_and_external_owners() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    let temp = Temp::new();
    let mut d = Document::new("stage peak");
    for _ in 0..16 {
        let root = Entity::new(
            format!("{}\"", "x".repeat(4095)),
            Geometry::Point([1., 2., 3.]),
            None,
        );
        d.entities.insert(root.id, root);
    }
    let id = d.id;
    let c = coordinator(d);
    let capture = ExportCapture::all(&c, id).unwrap();
    let inputs = exchange()
        .prepare_merge_inputs(
            &c,
            &capture,
            None,
            &temp.0,
            &Cancellation::default(),
            Usage::default(),
        )
        .unwrap();
    let wire = inputs
        .output_wire_plan(&c, &Cancellation::default(), Usage::default())
        .unwrap();
    let mut held = c.retained_usage().unwrap();
    held.add(inputs.owned_usage(), c.limits()).unwrap();
    held.add(wire.owned_usage(), c.limits()).unwrap();
    held.add(
        Usage::array::<SourceStamp>(capture.document().entities.len()).unwrap(),
        c.limits(),
    )
    .unwrap();
    let base = Usage::array::<u8>(temp.0.capacity()).unwrap();
    held.add(base, c.limits()).unwrap();
    let limit = c.limits().max_bytes;
    let mut accepted = 0;
    let mut refused = limit - base.bytes;
    while refused - accepted > 1 {
        let middle = accepted + (refused - accepted) / 2;
        let mut external = base;
        external
            .add(
                Usage {
                    bytes: middle,
                    ..Usage::default()
                },
                c.limits(),
            )
            .unwrap();
        match wire.stage_retained_request(&c, &temp.0, &Cancellation::default(), external) {
            Ok(_) => accepted = middle,
            Err(Error::Budget(_)) => refused = middle,
            Err(error) => panic!("unexpected staging boundary: {error:?}"),
        }
        assert!(temp.empty());
    }
    for external_bytes in [accepted, refused] {
        let mut external = base;
        external
            .add(
                Usage {
                    bytes: external_bytes,
                    ..Usage::default()
                },
                c.limits(),
            )
            .unwrap();
        let baseline = LIVE.load(Ordering::SeqCst);
        PEAK.store(baseline, Ordering::SeqCst);
        let result = wire.stage_retained_request(&c, &temp.0, &Cancellation::default(), external);
        let peak = PEAK.load(Ordering::SeqCst).saturating_sub(baseline) as u64;
        assert!(
            held.bytes + external_bytes + peak <= limit,
            "held={}, external={external_bytes}, new_peak={peak}, limit={limit}, admitted={}",
            held.bytes,
            result.is_ok()
        );
        assert_eq!(result.is_ok(), external_bytes == accepted);
        if let Ok(stage) = &result {
            assert!(stage.owned_usage().bytes >= 128 * 1024);
            stage.revalidate(&c, &Cancellation::default()).unwrap();
        }
        drop(result);
        assert!(temp.empty());
    }
    wire.revalidate(&c, &Cancellation::default()).unwrap();
}

#[test]
fn child_cancel_union_retains_all_three_owners_and_refuses_fifth_flag_before_growth() {
    let _serial = TEST.lock().unwrap_or_else(|e| e.into_inner());
    for canceled_owner in 0..3 {
        let temp = Temp::new();
        let d = Document::new("native cancellation union");
        let id = d.id;
        let c = coordinator(d);
        let capture = ExportCapture::all(&c, id).unwrap();
        let original = Cancellation::default();
        let creation = Cancellation::default();
        let caller = Cancellation::default();
        let inputs = exchange()
            .prepare_merge_inputs(&c, &capture, None, &temp.0, &original, Usage::default())
            .unwrap();
        let wire = inputs
            .output_wire_plan(&c, &caller, Usage::default())
            .unwrap();
        let stage = wire
            .stage_retained_request(&c, &temp.0, &creation, Usage::default())
            .unwrap();
        let child = stage.native_cancellation(&caller).unwrap();
        assert!(!child.is_canceled());
        [&original, &creation, &caller][canceled_owner].cancel();
        assert!(child.is_canceled());
        assert!(matches!(
            stage.native_cancellation(&caller),
            Err(Error::Canceled)
        ));
        drop(stage);
        assert!(temp.empty());
    }

    let temp = Temp::new();
    let d = Document::new("four original owners");
    let id = d.id;
    let c = coordinator(d);
    let capture = ExportCapture::all(&c, id).unwrap();
    let flags: [Cancellation; 4] = std::array::from_fn(|_| Cancellation::default());
    let original = flags[0]
        .either(&flags[1])
        .unwrap()
        .either(&flags[2])
        .unwrap()
        .either(&flags[3])
        .unwrap();
    let fifth = Cancellation::default();
    let inputs = exchange()
        .prepare_merge_inputs(&c, &capture, None, &temp.0, &original, Usage::default())
        .unwrap();
    let wire = inputs
        .output_wire_plan(&c, &original, Usage::default())
        .unwrap();
    assert!(matches!(
        wire.stage_retained_request(&c, &temp.0, &fifth, Usage::default()),
        Err(Error::Budget("cancellation owner limit"))
    ));
    assert!(temp.empty());
    let stage = wire
        .stage_retained_request(&c, &temp.0, &original, Usage::default())
        .unwrap();
    assert!(matches!(
        stage.native_cancellation(&fifth),
        Err(Error::Budget("cancellation owner limit"))
    ));
    stage.revalidate(&c, &original).unwrap();
    let child = stage.native_cancellation(&original).unwrap();
    flags[2].cancel();
    assert!(child.is_canceled());
    drop(stage);
    assert!(temp.empty());
}
