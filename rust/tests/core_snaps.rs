use openmatrix9_rust::core_snaps::{Candidate, State};
use openmatrix9_rust::core_snaps::{
    om9_snap_end_pick, om9_snap_load, om9_snap_state, om9_snap_toggle,
};

#[test]
fn om9_snap_002_native_state_and_packed_candidates() {
    let previous = om9_snap_state();
    assert!(om9_snap_load(0));
    assert!(om9_snap_toggle(2));
    assert_eq!(om9_snap_state(), 2);
    assert!(om9_snap_toggle(1));
    assert_eq!(om9_snap_state(), 3);
    assert!(!om9_snap_toggle(3));
    assert!(!om9_snap_load(16));
    assert_eq!(om9_snap_state(), 3);
    let candidates = [1., 2., 30., 9., 0., 4., 5., 60., 3., 4.];
    assert_eq!(
        unsafe { om9_snap_end_pick(candidates.as_ptr(), 2, 0., 0., 5.) },
        1
    );
    assert_eq!(
        unsafe { om9_snap_end_pick(std::ptr::null(), 2, 0., 0., 5.) },
        usize::MAX
    );
    assert_eq!(
        unsafe { om9_snap_end_pick(candidates.as_ptr(), usize::MAX, 0., 0., 5.) },
        usize::MAX
    );
    assert!(om9_snap_toggle(1));
    assert_eq!(om9_snap_state(), 2);
    assert_eq!(
        unsafe { om9_snap_end_pick(candidates.as_ptr(), 2, 0., 0., 5.) },
        usize::MAX
    );
    assert!(om9_snap_load(5));
    assert_eq!(
        unsafe {
            openmatrix9_rust::core_snaps::om9_snap_mode_pick(candidates.as_ptr(), 2, 0., 0., 5., 4)
        },
        1
    );
    assert_eq!(
        unsafe {
            openmatrix9_rust::core_snaps::om9_snap_mode_pick(candidates.as_ptr(), 2, 0., 0., 5., 2)
        },
        usize::MAX
    );
    assert_eq!(
        unsafe {
            openmatrix9_rust::core_snaps::om9_snap_mode_pick(candidates.as_ptr(), 2, 0., 0., 5., 3)
        },
        usize::MAX
    );
    assert!(om9_snap_toggle(1));
    assert_eq!(
        unsafe {
            openmatrix9_rust::core_snaps::om9_snap_mode_pick(candidates.as_ptr(), 2, 0., 0., 5., 4)
        },
        usize::MAX
    );
    assert!(om9_snap_load(9));
    assert_eq!(
        unsafe {
            openmatrix9_rust::core_snaps::om9_snap_mode_pick(candidates.as_ptr(), 2, 0., 0., 5., 8)
        },
        1
    );
    assert!(om9_snap_toggle(8));
    assert_eq!(
        unsafe {
            openmatrix9_rust::core_snaps::om9_snap_mode_pick(candidates.as_ptr(), 2, 0., 0., 5., 8)
        },
        usize::MAX
    );
    assert!(om9_snap_load(previous));
}
fn candidate(x: f64, y: f64) -> Candidate {
    Candidate {
        world: [0., 0., 10.],
        screen: [x, y],
    }
}
#[test]
fn om9_snap_002_master_preserves_mode_and_state_roundtrips() {
    let mut state = State::default();
    state.toggle_end();
    assert!(state.end && !state.enabled);
    state.toggle_master();
    assert!(state.enabled && state.end);
    state.toggle_master();
    assert!(!state.enabled && state.end);
    for value in 0..16 {
        assert_eq!(State::decode(value).unwrap().encoded(), value);
    }
    assert!(State::decode(16).is_none());
}
#[test]
fn om9_snap_002_selects_nearest_projected_candidate_within_aperture() {
    let state = State {
        enabled: true,
        end: true,
        mid: false,
        point: false,
    };
    let candidates = [candidate(9., 0.), candidate(3., 4.), candidate(0., 5.)];
    assert_eq!(state.pick(&candidates, [0., 0.], 5.), Some(1));
    assert_eq!(state.pick(&candidates, [0., 0.], 4.99), None);
    assert_eq!(
        State {
            enabled: false,
            end: true,
            mid: false,
            point: false
        }
        .pick(&candidates, [0., 0.], 10.),
        None
    );
    assert_eq!(
        State {
            enabled: true,
            end: false,
            mid: false,
            point: false
        }
        .pick(&candidates, [0., 0.], 10.),
        None
    );
}
#[test]
fn om9_snap_002_rejects_invalid_projection_world_and_aperture() {
    let state = State {
        enabled: true,
        end: true,
        mid: false,
        point: false,
    };
    let bad = [
        Candidate {
            world: [f64::NAN, 0., 0.],
            screen: [0., 0.],
        },
        candidate(f64::INFINITY, 0.),
        Candidate {
            world: [1e10, 0., 0.],
            screen: [0., 0.],
        },
    ];
    assert_eq!(state.pick(&bad, [0., 0.], 10.), None);
    for radius in [0., -1., f64::NAN, f64::INFINITY, 257.] {
        assert_eq!(state.pick(&[candidate(0., 0.)], [0., 0.], radius), None);
    }
    assert_eq!(state.pick(&[candidate(0., 0.)], [f64::NAN, 0.], 10.), None);
}

#[test]
fn om9_snap_007_point_mode_preserves_other_choices() {
    let mut state = State::decode(7).unwrap();
    state.toggle_point();
    assert_eq!(state.encoded(), 15);
    state.toggle_master();
    assert_eq!(state.encoded(), 14);
    assert!(state.point && state.end && state.mid && !state.enabled);
    state.toggle_point();
    assert_eq!(state.encoded(), 6);
    assert_eq!(
        openmatrix9_rust::core_snaps::command("ToolsObjectSnapPoint"),
        Some("Osnap P")
    );
}

#[test]
fn om9_snap_003_mid_mode_is_independent_and_preserved_by_master() {
    let mut state = State::default();
    state.toggle_mid();
    assert!(state.mid && !state.end && !state.enabled);
    assert_eq!(state.encoded(), 4);
    state.toggle_end();
    state.toggle_master();
    assert_eq!(state.encoded(), 7);
    state.toggle_master();
    assert_eq!(state.encoded(), 6);
    state.toggle_mid();
    assert_eq!(state.encoded(), 2);
    assert_eq!(
        openmatrix9_rust::core_snaps::command("ToolsObjectSnapMidpoint"),
        Some("Osnap M")
    );
}
