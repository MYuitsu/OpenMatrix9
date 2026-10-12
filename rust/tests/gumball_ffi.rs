// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{geometry_frame::IDENTITY, gumball_ffi::*};
fn frame() -> [f64; 12] {
    [0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.]
}
fn input() -> GumballBegin {
    GumballBegin {
        document: 1,
        view_generation: 1,
        unit_generation: 1,
        mm_per_unit: 1.,
        frame: frame(),
        cplane: frame(),
        ray_origin: [1., 1., 10.],
        ray_direction: [0., 0., -1.],
        handle: 0,
        modifiers: 0,
    }
}
fn source() -> GumballSource {
    GumballSource {
        document: 1,
        object: 2,
        generation: 1,
        revision: 1,
        epoch: 1,
        parent: 0,
        world: IDENTITY,
    }
}
unsafe fn ready() -> u64 {
    let h = om9_gumball_create();
    assert_ne!(h, 0);
    assert_eq!(
        unsafe {
            om9_gumball_event(
                h,
                &GumballEvent {
                    kind: 0,
                    pressed: 1,
                    ..Default::default()
                },
            )
        },
        0
    );
    h
}
#[test]
fn invalid_calls_do_not_touch_snapshot_or_session() {
    unsafe {
        let h = ready();
        let mut before = GumballSnapshot::default();
        assert_eq!(om9_gumball_snapshot(h, &mut before), 0);
        assert_eq!(
            om9_gumball_begin(h, std::ptr::null(), std::ptr::null(), 0),
            1
        );
        let mut b = input();
        b.handle = 99;
        assert_eq!(om9_gumball_begin(h, &b, &source(), 1), 1);
        assert_eq!(
            om9_gumball_begin(h, &input(), std::ptr::null(), usize::MAX),
            1
        );
        assert_eq!(
            om9_gumball_event(
                h,
                &GumballEvent {
                    kind: 99,
                    ..Default::default()
                }
            ),
            1
        );
        assert_eq!(om9_gumball_event(h, std::ptr::null()), 1);
        let mut after = GumballSnapshot::default();
        assert_eq!(om9_gumball_snapshot(h, &mut after), 0);
        assert_eq!(before, after);
        assert_eq!(om9_gumball_destroy(h), 0);
        assert_eq!(om9_gumball_snapshot(h, &mut after), 1);
        assert_eq!(after, before);
    }
}
#[test]
fn numeric_abi_preserves_double_and_independent_sessions() {
    unsafe {
        let a = ready();
        let b = ready();
        assert_eq!(om9_gumball_begin(a, &input(), &source(), 1), 0);
        let raw = b"5.123456789123 mm";
        assert_eq!(om9_gumball_input(a, raw.as_ptr(), raw.len()), 0);
        let mut out = GumballSnapshot::default();
        assert_eq!(om9_gumball_snapshot(a, &mut out), 0);
        assert_eq!(out.matrix[3], 5.123456789123);
        let saved = out;
        assert_eq!(om9_gumball_input(a, [255u8].as_ptr(), 1), 1);
        assert_eq!(om9_gumball_snapshot(a, &mut out), 0);
        assert_eq!(out, saved);
        assert_eq!(om9_gumball_snapshot(b, &mut out), 0);
        assert_eq!(out.phase, 1);
        assert_eq!(out.source_count, 0);
        assert_eq!(om9_gumball_destroy(a), 0);
        assert_eq!(om9_gumball_destroy(b), 0);
    }
}
#[test]
fn abi_layout_has_fixed_width_and_source_query_is_bounded() {
    unsafe {
        assert_eq!(std::mem::size_of::<GumballSource>(), 176);
        assert_eq!(std::mem::size_of::<GumballBegin>(), 280);
        let h = ready();
        assert_eq!(om9_gumball_begin(h, &input(), &source(), 1), 0);
        let mut written = 0;
        assert_eq!(
            om9_gumball_sources(h, std::ptr::null_mut(), 0, &mut written),
            0
        );
        assert_eq!(written, 1);
        let mut out = GumballSource::default();
        assert_eq!(om9_gumball_sources(h, &mut out, 1, &mut written), 0);
        assert_eq!(out.object, 2);
        assert_eq!(om9_gumball_destroy(h), 0);
    }
}
#[test]
fn frame_observation_abi_rejects_invalid_basis_atomically() {
    unsafe {
        let h = ready();
        let f = frame();
        assert_eq!(om9_gumball_frame(h, f.as_ptr(), f.as_ptr()), 0);
        let mut out = [42.; 12];
        let o = [1., 2., 3.];
        let x = [2., 0., 0.];
        let z = [1., 0., 4.];
        assert_eq!(
            om9_gumball_frame_from_directions(o.as_ptr(), x.as_ptr(), z.as_ptr(), out.as_mut_ptr()),
            0
        );
        assert_eq!(out, [1., 2., 3., 1., 0., 0., 0., 1., 0., 0., 0., 1.]);
        let old = out;
        assert_eq!(
            om9_gumball_frame_from_directions(o.as_ptr(), x.as_ptr(), x.as_ptr(), out.as_mut_ptr()),
            2
        );
        assert_eq!(out, old);
        assert_eq!(om9_gumball_destroy(h), 0);
    }
}

#[test]
fn command_abi_owns_only_exact_commands_and_preserves_outputs_on_error() {
    unsafe {
        let (mut kind, mut value) = (77, 88);
        let text = b"_GumballAlignment World";
        assert_eq!(om9_gumball_command(text.as_ptr(), text.len(), &mut kind, &mut value), 0);
        assert_eq!((kind, value), (2, 2));
        for text in [b"Line".as_slice(), b"Gumball garbage", &[255u8]] {
            kind = 77; value = 88;
            assert_ne!(om9_gumball_command(text.as_ptr(), text.len(), &mut kind, &mut value), 0);
            assert_eq!((kind, value), (77, 88));
        }
        assert_eq!(om9_gumball_command(std::ptr::null(), 1, &mut kind, &mut value), 2);
        assert_eq!(om9_gumball_command(text.as_ptr(), text.len(), std::ptr::null_mut(), &mut value), 2);
    }
}
#[test]
fn provenance_abi_is_bounded_and_never_writes_a_short_buffer() {
    unsafe {
        let h=ready();assert_eq!(om9_gumball_begin(h,&input(),&source(),1),0);
        let raw=b"5.123456789123 mm";assert_eq!(om9_gumball_input(h,raw.as_ptr(),raw.len()),0);
        let mut needed=0;assert_eq!(om9_gumball_report(h,std::ptr::null_mut(),0,&mut needed),0);assert!(needed>100&&needed<4096);
        let mut short=[42u8;10];let mut untouched=99;
        assert_ne!(om9_gumball_report(h,short.as_mut_ptr(),short.len(),&mut untouched),0);assert_eq!(short,[42;10]);assert_eq!(untouched,99);
        let mut out=vec![0u8;needed];assert_eq!(om9_gumball_report(h,out.as_mut_ptr(),out.len(),&mut needed),0);
        assert!(std::str::from_utf8(&out).unwrap().contains("5.123456789123 mm"));assert_eq!(om9_gumball_destroy(h),0);
        assert_ne!(om9_gumball_report(h,std::ptr::null_mut(),0,&mut untouched),0);assert_eq!(untouched,99);
    }
}
