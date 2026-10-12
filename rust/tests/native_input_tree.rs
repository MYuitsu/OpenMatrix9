use openmatrix9_rust::native_input_tree::*;
#[test]
fn balanced_tree_allows_shared_child_in_separate_branches() {
    let mut g = Gate::new();
    g.enter(1).unwrap();
    for _ in 0..2 {
        g.children(1).unwrap();
        g.enter(2).unwrap();
        g.account(88).unwrap();
        g.leave(2).unwrap();
    }
    g.leave(1).unwrap();
    g.finished().unwrap();
    assert_eq!((g.depth, g.nodes, g.bytes), (0, 3, 176));
}
#[test]
fn current_path_cycle_is_rejected_before_child_access_and_stays_failed() {
    let mut g = Gate::new();
    g.enter(1).unwrap();
    g.enter(2).unwrap();
    assert_eq!(g.enter(1), Err(TreeError::Cycle));
    assert_eq!((g.depth, g.nodes), (2, 2));
    assert_eq!(g.enter(3), Err(TreeError::Cycle));
    assert_eq!(g.finished(), Err(TreeError::Cycle));
}
#[test]
fn zero_identity_and_unbalanced_exit_reject() {
    let mut g = Gate::new();
    assert_eq!(g.enter(0), Err(TreeError::Identity));
    let mut g = Gate::new();
    assert_eq!(g.leave(1), Err(TreeError::Stack));
    let mut g = Gate::new();
    g.enter(1).unwrap();
    assert_eq!(g.leave(2), Err(TreeError::Stack));
    let mut g = Gate::new();
    g.enter(1).unwrap();
    assert_eq!(g.finished(), Err(TreeError::Stack));
}
#[test]
fn depth_bound_is_checked_before_path_storage() {
    let mut g = Gate::new();
    for i in 1..=MAX_DEPTH {
        g.enter(i as u64).unwrap();
    }
    assert_eq!(g.enter(999), Err(TreeError::Depth));
    assert_eq!(g.depth, MAX_DEPTH);
}
#[test]
fn breadth_bound_precedes_native_child_loop_and_checked_add_never_wraps() {
    let mut g = Gate::new();
    g.enter(1).unwrap();
    assert_eq!(g.children(MAX_NODES), Err(TreeError::Nodes));
    assert_eq!(g.nodes, 1);
    let mut g = Gate::new();
    g.enter(1).unwrap();
    assert_eq!(g.children(usize::MAX), Err(TreeError::Nodes));
    let mut g = Gate::new();
    for _ in 0..MAX_NODES {
        g.enter(1).unwrap();
        g.leave(1).unwrap();
    }
    assert_eq!(g.enter(2), Err(TreeError::Nodes));
    assert_eq!(g.nodes, MAX_NODES);
}
#[test]
fn aggregate_logical_bytes_reject_before_the_next_basis_extraction() {
    let mut g = Gate::new();
    g.enter(1).unwrap();
    g.account(MAX_BYTES / 2).unwrap();
    g.account(MAX_BYTES / 2).unwrap();
    assert_eq!(g.account(1), Err(TreeError::Bytes));
    assert_eq!(g.bytes, MAX_BYTES);
    let mut g = Gate::new();
    g.enter(1).unwrap();
    g.account(1).unwrap();
    assert_eq!(g.account(usize::MAX), Err(TreeError::Bytes));
    assert_eq!(g.bytes, 1);
}
#[test]
fn corrupted_state_never_indexes_untrusted_depth_or_grants_permission() {
    for mode in 0..5 {
        let mut g = Gate::new();
        match mode {
            0 => g.magic = 0,
            1 => g.depth = usize::MAX,
            2 => g.nodes = usize::MAX,
            3 => g.bytes = usize::MAX,
            _ => g.failure = 99,
        };
        assert_eq!(g.enter(1), Err(TreeError::State));
        assert_eq!(g.finished(), Err(TreeError::State));
    }
}
#[test]
fn value_state_layout_is_explicit_for_native_exclusive_borrow() {
    assert_eq!(std::mem::size_of::<Gate>(), 544);
    assert_eq!(std::mem::align_of::<Gate>(), 8);
    assert_eq!(std::mem::offset_of!(Gate, path), 32);
    assert_eq!(std::mem::offset_of!(Gate, bytes), 24);
}
#[test]
fn forged_stack_is_rejected_without_authorizing_native_access() {
    for mode in 0..4 {
        let mut g = Gate::new();
        g.enter(1).unwrap();
        g.enter(2).unwrap();
        match mode {
            0 => g.nodes = 1,
            1 => g.path[0] = 0,
            2 => g.path[1] = 1,
            _ => g.path[3] = 99,
        };
        assert_eq!(g.enter(3), Err(TreeError::State));
    }
}
