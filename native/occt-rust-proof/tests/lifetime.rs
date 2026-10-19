//! M00-L2-02 / L3-02 — lifetime, error safety and thread policy, asserted from Rust.

use om9_occt_rust_proof::{
    KernelError, box_volume, create_destroy, drilled_volume, handle_refcount, working_set_bytes,
};

#[test]
fn a_kernel_handle_reference_count_returns_to_baseline() {
    let (before, copied, after) = handle_refcount().expect("refcount");
    assert!(before >= 1, "a live handle must have a positive count");
    assert_eq!(copied, before + 1, "copying a handle must add one reference");
    assert_eq!(after, before, "nullifying the copy must restore the baseline");
}

#[test]
fn every_invalid_input_is_rejected_with_a_message() {
    let cases: [( &str, Result<f64, KernelError>); 9] = [
        ("box_zero_width", box_volume(0.0, 1.0, 1.0)),
        ("box_negative_depth", box_volume(1.0, 1.0, -1.0)),
        ("box_nan", box_volume(f64::NAN, 1.0, 1.0)),
        ("box_infinite", box_volume(f64::INFINITY, 1.0, 1.0)),
        ("drill_zero_radius", drilled_volume(10.0, 10.0, 4.0, 0.0)),
        (
            "drill_negative_radius",
            drilled_volume(10.0, 10.0, 4.0, -2.0),
        ),
        (
            "drill_nan_radius",
            drilled_volume(10.0, 10.0, 4.0, f64::NAN),
        ),
        ("box_huge", box_volume(1e300, 1e300, 1e300)),
        ("box_tiny", box_volume(1e-300, 1e-300, 1e-300)),
    ];
    for (name, outcome) in cases {
        let error = outcome.expect_err(name);
        assert!(
            !error.to_string().trim_end().ends_with(':'),
            "{name}: the failure must carry a reason, got {error}"
        );
    }
}

#[test]
fn ten_thousand_create_destroy_cycles_do_not_grow_the_working_set() {
    let before = working_set_bytes().expect("working set");
    let elapsed = create_destroy(10_000).expect("create/destroy");
    let after = working_set_bytes().expect("working set");
    assert!(elapsed < 60.0, "10k cycles took {elapsed}s");
    let growth_mb = (after - before) / (1024.0 * 1024.0);
    // Allocator caches and the kernel's pools settle well below this; a real leak over 10k shapes
    // would be far larger.
    assert!(growth_mb < 8.0, "working set grew {growth_mb} MiB over 10k cycles");
}

#[test]
fn serialized_calls_from_several_threads_stay_exact() {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let gate = Mutex::new(());
    let failures = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for _ in 0..100 {
                    let _held = gate.lock().expect("serialization mutex");
                    match box_volume(10.0, 10.0, 4.0) {
                        Ok(volume) if (volume - 400.0).abs() < 1e-9 => {}
                        _ => {
                            failures.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
        }
    });
    assert_eq!(failures.load(Ordering::Relaxed), 0);
}
