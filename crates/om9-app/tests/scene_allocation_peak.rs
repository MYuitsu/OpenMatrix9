// SPDX-License-Identifier: LGPL-2.1-or-later
//! This isolated test executable uses a legal always-moving allocator so Vec
//! growth cannot hide old/new coexistence in an in-place System realloc.
use om9_app::{layer_dialog::LayerDialogKind, session::Controller};
use om9_core::{
    Error,
    budget::Limits,
    geometry::{Cloud, Geometry},
    model::{Definition, Document, Entity},
};
use om9_scene::scene::Scene;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

struct MovingAllocator;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
fn allocated(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::SeqCst) + bytes;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
// SAFETY: all storage comes from System with the requested layout. Realloc
// preserves min(old,new) bytes and frees the old block only after success.
unsafe impl GlobalAlloc for MovingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let Ok(next_layout) = Layout::from_size_align(size, layout.align()) else {
            return std::ptr::null_mut();
        };
        let next = unsafe { System.alloc(next_layout) };
        if !next.is_null() {
            allocated(size);
            unsafe {
                std::ptr::copy_nonoverlapping(pointer, next, layout.size().min(size));
                System.dealloc(pointer, layout);
            }
            LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: MovingAllocator = MovingAllocator;

#[test]
fn moving_reallocation_cannot_exceed_admitted_nested_scene_or_cloud_peak() {
    let mut document = Document::new("nested scene peak");
    let base = Definition::new(
        "eight points",
        (0..8)
            .map(|i| Entity::new("point", Geometry::Point([i as f64, 0., 0.]), None))
            .collect(),
    );
    let mut id = base.id;
    document.definitions.insert(id, base);
    for _ in 0..3 {
        let definition = Definition::new(
            "eight instances",
            (0..8)
                .map(|_| Entity::new("instance", Geometry::Instance(id), None))
                .collect(),
        );
        id = definition.id;
        document.definitions.insert(id, definition);
    }
    let root = Entity::new("root", Geometry::Instance(id), None);
    document.entities.insert(root.id, root);
    let limits = Limits {
        max_bytes: 2_500_000,
        ..Limits::default()
    };
    PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
    let mut controller = Controller::new(document, limits).unwrap();
    let nested_peak = PEAK.load(Ordering::SeqCst);
    assert_eq!(controller.scene().unwrap().objects.len(), 4096);
    assert!(nested_peak <= limits.max_bytes as usize, "{nested_peak}");
    let token = controller
        .begin_layer_dialog(
            LayerDialogKind::Rename,
            Some(om9_core::layers::PresetSlot::new(17).unwrap()),
        )
        .unwrap();
    controller
        .set_layer_dialog_name(token, "budgeted custom name")
        .unwrap();
    let epoch = controller.coordinator().epoch();
    PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
    let result = controller.confirm_layer_dialog(token);
    let publication_peak = PEAK.load(Ordering::SeqCst);
    assert!(matches!(result, Err(Error::Budget(_))));
    assert!(
        publication_peak <= limits.max_bytes as usize,
        "{publication_peak}"
    );
    assert_eq!(controller.coordinator().epoch(), epoch);
    assert!(controller.document().layers.is_empty());
    assert_eq!(controller.scene().unwrap().objects.len(), 4096);
    assert_eq!(
        controller.layer_dialog().unwrap().name(),
        Some("budgeted custom name")
    );
    controller.cancel_layer_dialog(token).unwrap();
    drop(controller);

    let mut document = Document::new("non power of two Cloud peak");
    let root = Entity::new(
        "Cloud",
        Geometry::Cloud(Cloud {
            points: (0..65_537).map(|i| [i as f64, 0., 0.]).collect(),
            normals: vec![],
            colors: vec![],
            hidden: vec![],
        }),
        None,
    );
    document.entities.insert(root.id, root);
    let document = Arc::new(document);
    let limits = Limits {
        max_bytes: 5_000_000,
        ..Limits::default()
    };
    PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
    let scene = Scene::capture(document, 1, &limits).unwrap();
    let cloud_peak = PEAK.load(Ordering::SeqCst);
    assert_eq!(scene.objects.len(), 1);
    assert!(cloud_peak <= limits.max_bytes as usize, "{cloud_peak}");
    drop(scene);

    let pattern = "x".repeat(8192);
    let mut document = Document::new("large hidden label snapshot");
    for _ in 0..1000 {
        let mut e = Entity::new(pattern.clone(), Geometry::Point([0.; 3]), None);
        e.attributes.hidden = true;
        document.entities.insert(e.id, e);
    }
    let limits = Limits {
        max_bytes: 14_000_000,
        ..Limits::default()
    };
    let mut controller = Controller::new(document, limits).unwrap();
    let epoch = controller.coordinator().epoch();
    PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
    let result = controller.begin_layer_dialog(LayerDialogKind::Show, None);
    let snapshot_peak = PEAK.load(Ordering::SeqCst);
    assert!(matches!(result, Err(Error::Budget(_))));
    assert!(controller.layer_dialog().is_none());
    assert_eq!(controller.coordinator().epoch(), epoch);
    assert!(
        controller
            .document()
            .entities
            .values()
            .all(|e| e.attributes.hidden && e.label == pattern)
    );
    assert!(
        snapshot_peak <= limits.max_bytes as usize,
        "{snapshot_peak}"
    );
    println!(
        "OM9_SCENE_PEAK nested_leaves=4096 nested_peak={nested_peak} nested_limit=2500000 cloud_points=65537 cloud_peak={cloud_peak} cloud_limit=5000000"
    );
    println!(
        "OM9_MODAL_PEAK publication_peak={publication_peak} publication_limit=2500000 hidden_roots=1000 label_bytes=8192 snapshot_peak={snapshot_peak} snapshot_limit=14000000 refused=true epoch_unchanged=true"
    );
}
