use openmatrix9_rust::native_input_tree::{Gate, MAX_BYTES, TreeError};
use openmatrix9_rust::native_input_tree_ffi::*;
#[test]
fn native_exclusive_gate_observes_policy_and_sticky_failure() {
    let mut g = om9_input_tree_init();
    unsafe {
        assert_eq!(om9_input_tree_enter(&mut g, 1), 0);
        assert_eq!(om9_input_tree_children(&mut g, 1), 0);
        assert_eq!(om9_input_tree_enter(&mut g, 2), 0);
        assert_eq!(om9_input_tree_account(&mut g, MAX_BYTES), 0);
        assert_eq!(om9_input_tree_account(&mut g, 1), TreeError::Bytes as u32);
        assert_eq!(om9_input_tree_leave(&mut g, 2), TreeError::Bytes as u32);
        assert_eq!(om9_input_tree_finish(&mut g), TreeError::Bytes as u32);
    }
    assert_eq!((g.depth, g.nodes, g.bytes), (2, 2, MAX_BYTES));
}
#[test]
fn balanced_ffi_and_cycle_ffi_use_original_owner_identity() {
    let mut g = om9_input_tree_init();
    unsafe {
        assert_eq!(om9_input_tree_enter(&mut g, 1), 0);
        assert_eq!(om9_input_tree_leave(&mut g, 1), 0);
        assert_eq!(om9_input_tree_finish(&mut g), 0);
    }
    let mut g = om9_input_tree_init();
    unsafe {
        assert_eq!(om9_input_tree_enter(&mut g, 1), 0);
        assert_eq!(om9_input_tree_enter(&mut g, 1), TreeError::Cycle as u32);
    }
}
#[test]
fn null_misaligned_and_address_overflow_reject_without_dereferencing() {
    for ptr in [
        std::ptr::null_mut(),
        1usize as *mut Gate,
        (usize::MAX - 7) as *mut Gate,
    ] {
        unsafe {
            assert_eq!(om9_input_tree_enter(ptr, 1), 1);
            assert_eq!(om9_input_tree_leave(ptr, 1), 1);
            assert_eq!(om9_input_tree_children(ptr, 1), 1);
            assert_eq!(om9_input_tree_account(ptr, 1), 1);
            assert_eq!(om9_input_tree_finish(ptr), 1);
        }
    }
}
#[test]
fn initialized_but_corrupt_value_rejects_before_any_stack_index() {
    let mut g = om9_input_tree_init();
    g.depth = usize::MAX;
    unsafe {
        assert_eq!(om9_input_tree_enter(&mut g, 2), 1);
        assert_eq!(om9_input_tree_finish(&mut g), 1);
    }
}
