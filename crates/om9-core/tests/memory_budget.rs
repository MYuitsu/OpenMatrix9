use om9_core::budget::memory::{MemoryBudget, MemoryClass};

const MIB: u64 = 1024 * 1024;

#[test]
fn concurrent_reservations_obey_shared_ceiling_and_release_on_drop() {
    let budget = MemoryBudget::new(1024 * MIB).unwrap();
    let doc = budget
        .reserve(MemoryClass::Document, 300 * MIB, None)
        .unwrap();
    let undo = budget.reserve(MemoryClass::Undo, 200 * MIB, None).unwrap();
    let before = budget.totals();
    assert!(
        budget
            .reserve(MemoryClass::Kernel, 120 * MIB, None)
            .is_err()
    );
    assert_eq!(budget.totals(), before);
    drop(undo);
    let job = budget
        .reserve(MemoryClass::Kernel, 120 * MIB, None)
        .unwrap();
    assert_eq!(budget.totals().0, 420 * MIB);
    drop((doc, job));
    assert_eq!(budget.totals().0, 0);
}

#[test]
fn physical_headroom_failure_is_atomic() {
    let budget = MemoryBudget::new(8 * 1024 * MIB).unwrap();
    assert!(
        budget
            .reserve(MemoryClass::Export, 300 * MIB, Some(1000 * MIB))
            .is_err()
    );
    assert_eq!(budget.totals().0, 0);
    let job = budget
        .reserve(MemoryClass::Export, 100 * MIB, Some(1000 * MIB))
        .unwrap();
    assert_eq!(budget.totals().0, 100 * MIB);
    drop(job);
    assert_eq!(budget.totals().0, 0);
}

#[test]
fn simultaneous_pending_jobs_do_not_spend_the_same_free_ram() {
    let budget = MemoryBudget::new(8 * 1024 * MIB).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let budget = budget.clone();
        let barrier = barrier.clone();
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            budget.reserve(MemoryClass::Export, 120 * MIB, Some(1000 * MIB)).ok()
        }));
    }
    barrier.wait();
    let held: Vec<_> = workers.into_iter().map(|worker| worker.join().unwrap()).collect();
    assert_eq!(held.iter().filter(|item| item.is_some()).count(), 1);
    assert_eq!(budget.totals().0, 120 * MIB);
    drop(held);
    assert_eq!(budget.totals().0, 0);
    assert!(budget.reserve(MemoryClass::Export, 120 * MIB, Some(1000 * MIB)).is_ok());
}

#[test]
fn allocated_bytes_keep_managed_charge_without_spending_available_twice() {
    let budget = MemoryBudget::new(1024 * MIB).unwrap();
    // Unsampled requests are still pending; missing a sample is not evidence
    // that the allocation has already occurred.
    let mut owner = budget.reserve(MemoryClass::CpuCache, 100 * MIB, None).unwrap();
    assert!(budget.reserve(MemoryClass::Export, 20 * MIB, Some(300 * MIB)).is_err());
    owner.mark_allocated(80 * MIB).unwrap();
    assert_eq!(owner.pending_bytes(), 20 * MIB);
    assert_eq!(budget.totals().0, 100 * MIB);
    assert!(owner.mark_allocated(21 * MIB).is_err());
    assert_eq!(owner.pending_bytes(), 20 * MIB);
    let job = budget.reserve(MemoryClass::Export, 20 * MIB, Some(300 * MIB)).unwrap();
    assert_eq!(budget.totals().0, 120 * MIB);
    // Live memory still obeys the 60% managed ceiling even with ample Available.
    assert!(budget.reserve(MemoryClass::Kernel, 500 * MIB, Some(1024 * MIB)).is_err());
    drop((owner, job));
    assert_eq!(budget.totals().0, 0);
    assert!(budget.reserve(MemoryClass::Export, 40 * MIB, Some(300 * MIB)).is_ok());
}
