// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{native_cage_capability::*,cage_commands::*};
fn envelope(kind:Kind)->Envelope{Request{phase:Phase::Ready,input_count:1,has_control:kind==Kind::Capture,
    hi:[10.;3],..Request::new(kind,Stage::Construct)}.into()}
#[test]
fn scalar_layout_query_and_rejections_preserve_caller_storage(){
    assert_eq!(std::mem::size_of::<Envelope>(),136);assert_eq!(std::mem::align_of::<Envelope>(),8);
    assert_eq!(std::mem::offset_of!(Envelope,input_count),24);assert_eq!(std::mem::offset_of!(Envelope,counts),32);
    assert_eq!(std::mem::offset_of!(Envelope,lo),80);assert_eq!(std::mem::offset_of!(Envelope,falloff),128);
    for kind in [Kind::Create,Kind::Capture,Kind::Release]{assert!(om9_native_cage_declared(kind as u32));
        let r=envelope(kind);let before=r;
        for input in 1..=8{assert_eq!(om9_native_cage_capability(input,r),7);}
        for input in 9..=13{assert_eq!(om9_native_cage_capability(input,r),101);}
        assert_eq!(r,before);
    }
    assert!(!om9_native_cage_declared(0));assert!(!om9_native_cage_declared(4));
    let r=envelope(Kind::Capture);
    for bad in [Envelope{flags:16,..r},Envelope{reserved:1,..r},Envelope{stage:0,..r},Envelope{phase:5,..r},
        Envelope{role:4,..r},Envelope{flags:0,..r},Envelope{falloff:-1.,..r}]{assert_eq!(om9_native_cage_capability(8,bad),102);}
    assert_eq!(om9_native_cage_capability(0,r),101);assert_eq!(om9_native_cage_capability(8,Envelope{kind:0,..r}),100);
}
#[test]
fn session_query_uses_real_phase_options_and_does_not_mutate_or_retain_handle(){unsafe{
    let h=om9_cage_session_new(2);assert!(!h.is_null());
    assert_eq!(om9_cage_session_capability(h,8,1,0),7);assert_eq!(om9_cage_session_capability(h,8,3,0),102);
    let lo=[0.;3];let hi=[10.;3];
    assert_eq!(om9_cage_session_add(h,c"Mesh".as_ptr(),c"basis".as_ptr(),lo.as_ptr(),hi.as_ptr(),0),0);
    assert_eq!(om9_cage_session_advance(h),0);
    assert_eq!(om9_cage_session_capability(h,7,2,1),7);assert_eq!(om9_cage_session_capability(h,8,2,1),101);
    assert_eq!(om9_cage_session_add(h,c"Control".as_ptr(),c"basis".as_ptr(),lo.as_ptr(),hi.as_ptr(),1),0);
    assert_eq!(om9_cage_session_option(h,c"Region".as_ptr(),c"Local".as_ptr()),0);
    assert_eq!(om9_cage_session_option(h,c"Falloff".as_ptr(),c"2.5".as_ptr()),0);
    let before=(*h).capability_request(Stage::Construct,Role::Geometry).unwrap();
    assert_eq!(om9_cage_session_capability(h,8,3,0),7);assert_eq!(om9_cage_session_prepare(h),0);
    assert_eq!((*h).capability_request(Stage::Construct,Role::Geometry).unwrap(),before);
    assert_eq!(om9_cage_session_capability(h,8,3,4),102);om9_cage_session_free(h);
    assert_eq!(om9_cage_session_capability(std::ptr::null(),8,3,0),103);
}}
#[test]
fn unknown_add_role_is_rejected_without_turning_it_into_a_native_control(){unsafe{
    let h=om9_cage_session_new(2);let lo=[0.;3];let hi=[10.;3];
    assert_eq!(om9_cage_session_add(h,c"Mesh".as_ptr(),c"basis".as_ptr(),lo.as_ptr(),hi.as_ptr(),0),0);
    assert_eq!(om9_cage_session_advance(h),0);
    assert_ne!(om9_cage_session_add(h,c"Bogus".as_ptr(),c"basis".as_ptr(),lo.as_ptr(),hi.as_ptr(),3),0);
    assert_eq!(om9_cage_session_phase(h),Phase::Control as u32);assert!(om9_cage_session_control(h).is_null());
    om9_cage_session_free(h);
}}
