// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::gumball_component_ffi::om9_gumball_component_frame;

const POINTS: [[f64; 3]; 4] = [[4., 5., 6.], [6., 5., 6.], [4., 6., 6.], [4., 5., 10.]];
const TRIANGLES: [[u32; 3]; 2] = [[0, 1, 2], [0, 2, 3]];

fn close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (&a, &b) in actual.iter().zip(expected) {
        assert!((a - b).abs() < 1e-10, "{actual:?} != {expected:?}");
    }
}

#[test]
fn abi_edge_has_actual_midpoint_and_canonical_direction() {
    let mut frame = [99.; 12];
    let status = unsafe {
        om9_gumball_component_frame(
            POINTS.as_ptr(),
            4,
            TRIANGLES.as_ptr(),
            2,
            1,
            2,
            0,
            frame.as_mut_ptr(),
        )
    };
    assert_eq!(status, 0);
    let s = 5_f64.sqrt();
    close(
        &frame,
        &[
            4.,
            5.5,
            6.,
            0.,
            1.,
            0.,
            -1. / s,
            0.,
            2. / s,
            2. / s,
            0.,
            1. / s,
        ],
    );
}

#[test]
fn abi_vertex_packs_area_weighted_frame_in_native_axis_order() {
    let mut frame = [99.; 12];
    assert_eq!(
        unsafe {
            om9_gumball_component_frame(
                POINTS.as_ptr(),
                4,
                TRIANGLES.as_ptr(),
                2,
                0,
                0,
                0,
                frame.as_mut_ptr(),
            )
        },
        0
    );
    let s = 5_f64.sqrt();
    close(
        &frame,
        &[
            4.,
            5.,
            6.,
            1. / s,
            0.,
            -2. / s,
            0.,
            1.,
            0.,
            2. / s,
            0.,
            1. / s,
        ],
    );
}

#[test]
fn null_inputs_reject_and_zero_writable_output() {
    for (points, triangles) in [
        (std::ptr::null(), TRIANGLES.as_ptr()),
        (POINTS.as_ptr(), std::ptr::null()),
    ] {
        let mut frame = [99.; 12];
        assert_eq!(
            unsafe {
                om9_gumball_component_frame(points, 4, triangles, 2, 0, 0, 0, frame.as_mut_ptr())
            },
            1
        );
        assert_eq!(frame, [0.; 12]);
    }
    assert_eq!(
        unsafe {
            om9_gumball_component_frame(
                POINTS.as_ptr(),
                4,
                TRIANGLES.as_ptr(),
                2,
                0,
                0,
                0,
                std::ptr::null_mut(),
            )
        },
        1
    );
}

#[test]
fn oversized_lengths_reject_before_reading_inputs() {
    for (points, triangles) in [(131073, 2), (4, 131073), (usize::MAX, usize::MAX)] {
        let mut frame = [99.; 12];
        assert_eq!(
            unsafe {
                om9_gumball_component_frame(
                    std::ptr::null(),
                    points,
                    std::ptr::null(),
                    triangles,
                    0,
                    0,
                    0,
                    frame.as_mut_ptr(),
                )
            },
            1
        );
        assert_eq!(frame, [0.; 12]);
    }
}

#[test]
fn invalid_kind_and_empty_snapshots_leave_zero_output() {
    for (points, triangles, kind) in [(4, 2, 2), (0, 2, 0), (4, 0, 1)] {
        let mut frame = [99.; 12];
        assert_eq!(
            unsafe {
                om9_gumball_component_frame(
                    POINTS.as_ptr(),
                    points,
                    TRIANGLES.as_ptr(),
                    triangles,
                    kind,
                    0,
                    1,
                    frame.as_mut_ptr(),
                )
            },
            1
        );
        assert_eq!(frame, [0.; 12]);
    }
}

#[test]
fn semantic_failures_cannot_publish_a_previous_or_partial_frame() {
    let mut invalid = POINTS;
    invalid[3][0] = f64::NAN;
    for (points, triangles, first, second) in [
        (invalid.as_slice(), TRIANGLES.as_slice(), 0, 2),
        (POINTS.as_slice(), [[0, 1, 99]].as_slice(), 0, 1),
        (POINTS.as_slice(), TRIANGLES.as_slice(), 0, 0),
        (POINTS.as_slice(), TRIANGLES.as_slice(), 1, 3),
        (
            [[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]].as_slice(),
            [[0, 1, 2]].as_slice(),
            0,
            1,
        ),
    ] {
        let mut frame = [99.; 12];
        assert_eq!(
            unsafe {
                om9_gumball_component_frame(
                    points.as_ptr(),
                    points.len(),
                    triangles.as_ptr(),
                    triangles.len(),
                    1,
                    first,
                    second,
                    frame.as_mut_ptr(),
                )
            },
            2
        );
        assert_eq!(frame, [0.; 12]);
    }
}

#[test]
fn abi_writes_exactly_twelve_doubles_for_success_and_failure() {
    let mut guarded = [77.; 14];
    assert_eq!(
        unsafe {
            om9_gumball_component_frame(
                POINTS.as_ptr(),
                4,
                TRIANGLES.as_ptr(),
                2,
                0,
                0,
                0,
                guarded.as_mut_ptr().add(1),
            )
        },
        0
    );
    assert_eq!((guarded[0], guarded[13]), (77., 77.));
    assert_eq!(
        unsafe {
            om9_gumball_component_frame(
                POINTS.as_ptr(),
                4,
                TRIANGLES.as_ptr(),
                2,
                0,
                99,
                0,
                guarded.as_mut_ptr().add(1),
            )
        },
        2
    );
    assert_eq!(&guarded[1..13], &[0.; 12]);
    assert_eq!((guarded[0], guarded[13]), (77., 77.));
}
