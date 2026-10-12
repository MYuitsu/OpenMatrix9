use openmatrix9_rust::core_snaps::{
    om9_snap_accept_point, om9_snap_effective_state, om9_snap_load, om9_snap_state,
    om9_snap_submit, om9_snap_transient_clear,
};
use std::ffi::CString;

fn submit(input: &str) -> u32 {
    let input = CString::new(input).unwrap();
    unsafe { om9_snap_submit(input.as_ptr()) }
}

#[test]
fn one_shot_survives_rejected_pick_and_restores_persistent_modes_after_acceptance() {
    assert!(om9_snap_load(7)); // Persistent End + Mid.
    assert_eq!(submit("Osnap Once Point"), 2);
    assert_eq!(om9_snap_state(), 7);
    assert_eq!(om9_snap_effective_state(), 9);
    let candidate = [0., 0., 10., 0., 0.];
    assert_eq!(unsafe { openmatrix9_rust::core_snaps::om9_snap_mode_pick(
        candidate.as_ptr(), 1, 0., 0., 8., 8) }, 0);
    assert_eq!(unsafe { openmatrix9_rust::core_snaps::om9_snap_mode_pick(
        candidate.as_ptr(), 1, 0., 0., 8., 2) }, usize::MAX);
    assert_eq!(unsafe { openmatrix9_rust::core_snaps::om9_snap_mode_pick(
        std::ptr::null(), 0, 0., 0., 8., 8) }, usize::MAX);
    assert_eq!(om9_snap_effective_state(), 9); // Preview/no-candidate never consumes.
    om9_snap_accept_point(false);
    assert_eq!(om9_snap_effective_state(), 9);
    om9_snap_accept_point(true);
    assert_eq!(om9_snap_effective_state(), 7);

    assert_eq!(submit("osnap once e"), 2);
    assert_eq!(submit("Osnap Suspend"), 2);
    assert_eq!(om9_snap_effective_state(), 0);
    assert_eq!(om9_snap_state(), 7);
    om9_snap_accept_point(true); // A free point while suspended does not consume an unused override.
    assert_eq!(submit("Osnap Resume"), 2);
    assert_eq!(om9_snap_effective_state(), 3);
    om9_snap_transient_clear();
    assert_eq!(om9_snap_effective_state(), 7);

    assert_eq!(submit("Osnap Only Mid"), 1);
    assert_eq!(om9_snap_state(), 5);
    assert_eq!(submit("Osnap Clear"), 1);
    assert_eq!(om9_snap_state(), 1);
    assert_eq!(om9_snap_effective_state(), 1);
    assert_eq!(submit("Osnap Once Near"), 3);
    assert_eq!(om9_snap_state(), 1);
    assert_eq!(submit("Line"), 0);

    // An explicit one-shot can pick with persistent snapping switched off.
    assert!(om9_snap_load(6));
    assert_eq!(submit("Osnap Once Point"), 2);
    assert_eq!(om9_snap_effective_state(), 9);
    assert!(om9_snap_load(3)); // Reload/deactivate cannot restore a stale one-shot.
    assert_eq!(om9_snap_effective_state(), 3);
}
