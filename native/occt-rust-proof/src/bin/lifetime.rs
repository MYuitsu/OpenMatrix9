//! M00-L2-02 / L3-02 gate probe — lifetime, threading and error safety.
//!
//! Covers the four L4s of `OM9-M00-L3-02-02`:
//!
//! * `L4-02-02-01` RAII and the free path: a kernel handle's reference count rises on copy and returns to
//!   its baseline when the copy is nullified;
//! * `L4-02-02-02` invalid inputs and rollback: a matrix of bad inputs must come back as a status code with
//!   a message, never as a crash and never as a partially valid result;
//! * `L4-02-02-03` create/destroy and leaks: 10 000 create/destroy cycles with the working set sampled
//!   before and after;
//! * `L4-02-02-04` thread serialization: concurrent callers go through one mutex (the documented policy)
//!   and every result stays exact; independent construction from several threads is measured separately
//!   and reported as an observation, not as a promise.
//!
//! It prints one JSON object so the log is the evidence. Feasibility only: no product code runs here.
use om9_occt_rust_proof::{
    KernelError, box_volume, create_destroy, drilled_volume, handle_refcount, version,
    working_set_bytes,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    // --- L4-02-02-01: RAII and the free path ---------------------------------
    let refcount = handle_refcount();
    let (before, copied, after) = refcount.as_ref().copied().unwrap_or((-1, -1, -1));

    // --- L4-02-02-02: invalid inputs and rollback ----------------------------
    let mut invalid = Vec::new();
    let cases: Vec<(&str, Box<dyn Fn() -> Result<f64, KernelError>>)> = vec![
        ("box_zero_width", Box::new(|| box_volume(0.0, 1.0, 1.0))),
        ("box_negative_depth", Box::new(|| box_volume(1.0, 1.0, -1.0))),
        ("box_nan", Box::new(|| box_volume(f64::NAN, 1.0, 1.0))),
        ("box_infinite", Box::new(|| box_volume(f64::INFINITY, 1.0, 1.0))),
        ("drill_zero_radius", Box::new(|| drilled_volume(10.0, 10.0, 4.0, 0.0))),
        ("drill_negative_radius", Box::new(|| drilled_volume(10.0, 10.0, 4.0, -2.0))),
        ("drill_nan_radius", Box::new(|| drilled_volume(10.0, 10.0, 4.0, f64::NAN))),
        ("box_huge", Box::new(|| box_volume(1e300, 1e300, 1e300))),
        ("box_tiny", Box::new(|| box_volume(1e-300, 1e-300, 1e-300))),
    ];
    for (name, call) in &cases {
        let outcome = call();
        let (rejected, message) = match outcome {
            Ok(value) => (false, format!("accepted with {value}")),
            Err(error) => (true, error.to_string()),
        };
        invalid.push(format!(
            "{{\"case\": \"{name}\", \"rejected\": {rejected}, \"message\": \"{}\"}}",
            json_escape(&message)
        ));
    }

    // --- L4-02-02-03: create/destroy and leaks ------------------------------
    let cycles = 10_000i64;
    let before_bytes = working_set_bytes().unwrap_or(0.0);
    let started = Instant::now();
    let creation = create_destroy(cycles);
    let elapsed = started.elapsed().as_secs_f64();
    let after_bytes = working_set_bytes().unwrap_or(0.0);
    let growth_mb = (after_bytes - before_bytes) / (1024.0 * 1024.0);

    // --- L4-02-02-04: thread serialization ---------------------------------
    // The policy: the kernel is called one operation at a time. The mutex is the gate the application
    // will honour; the probe verifies that results stay exact under concurrent callers.
    let guard = Mutex::new(());
    let failures = AtomicUsize::new(0);
    let calls = 250usize;
    let threads = 4usize;
    let serialized_started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                for _ in 0..calls {
                    let _held = guard.lock().expect("serialization mutex");
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
    let serialized = serialized_started.elapsed().as_secs_f64();

    // Observation only: independent construction without the mutex. A failure here tells the project to
    // keep the mutex, not that the kernel is safe.
    let independent_failures = AtomicUsize::new(0);
    let independent_started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                for _ in 0..25 {
                    match box_volume(10.0, 10.0, 4.0) {
                        Ok(volume) if (volume - 400.0).abs() < 1e-9 => {}
                        _ => {
                            independent_failures.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
        }
    });
    let independent = independent_started.elapsed().as_secs_f64();

    let refcount_ok = refcount.is_ok() && before >= 1 && copied == before + 1 && after == before;
    let rejected = invalid.iter().filter(|entry| entry.contains("\"rejected\": true")).count();
    let all_rejected = rejected == invalid.len();
    println!(
        concat!(
            "{{\"occt_version\": \"{version}\", ",
            "\"raii\": {{\"before\": {before}, \"while_copied\": {copied}, \"after_nullify\": {after}, \"ok\": {raii_ok}}}, ",
            "\"invalid_inputs\": {{\"cases\": [{cases}], \"rejected\": {rejected}, \"all_rejected\": {all_rejected}}}, ",
            "\"create_destroy\": {{\"cycles\": {cycles}, \"elapsed_seconds\": {elapsed:.6}, ",
            "\"working_set_before_bytes\": {before_bytes:.0}, \"working_set_after_bytes\": {after_bytes:.0}, ",
            "\"growth_mb\": {growth_mb:.3}, \"ok\": {creation_ok}}}, ",
            "\"threads\": {{\"policy\": \"one kernel call at a time through a mutex\", ",
            "\"serialized_calls\": {serialized_calls}, \"serialized_failures\": {serialized_failures}, ",
            "\"serialized_seconds\": {serialized:.6}, \"independent_calls\": {independent_calls}, ",
            "\"independent_failures\": {independent_failures}, \"independent_seconds\": {independent:.6}}}}}"
        ),
        version = version(),
        before = before,
        copied = copied,
        after = after,
        raii_ok = refcount_ok,
        cases = invalid.join(", "),
        rejected = rejected,
        all_rejected = all_rejected,
        cycles = cycles,
        elapsed = elapsed,
        before_bytes = before_bytes,
        after_bytes = after_bytes,
        growth_mb = growth_mb,
        creation_ok = creation.is_ok(),
        serialized_calls = threads * calls,
        serialized_failures = failures.load(Ordering::Relaxed),
        serialized = serialized,
        independent_calls = threads * 25,
        independent_failures = independent_failures.load(Ordering::Relaxed),
        independent = independent,
    );
}
