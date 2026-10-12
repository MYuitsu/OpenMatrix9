// SPDX-License-Identifier: LGPL-2.1-or-later
//! Cancellation invalidates publication tokens even when no Curve replaces it.
use openmatrix9_rust::curve::CurveSession;
use std::ffi::{CString,c_char};
unsafe extern "C" {
    fn om9_curve_start(text:*const c_char)->bool;
    fn om9_curve_input(text:*const c_char)->u32;
    fn om9_curve_generation()->u64;
    fn om9_curve_cancel();
}
#[test]
fn explicit_and_input_cancellation_retire_the_original_publication_generation(){
    // One test owns this executable's process-global command state.
    let name=CString::new("Line").unwrap();let cancel=CString::new("Cancel").unwrap();
    let mut reference=CurveSession::default();reference.start("Line").unwrap();
    let (explicit,input)=unsafe{
        assert!(om9_curve_start(name.as_ptr()));let first=om9_curve_generation();
        om9_curve_cancel();let explicit=om9_curve_generation()!=first;
        assert!(om9_curve_start(name.as_ptr()));let second=om9_curve_generation();
        assert_eq!(om9_curve_input(cancel.as_ptr()),3);let input=om9_curve_generation()!=second;
        om9_curve_cancel();(explicit,input)
    };
    assert_eq!((explicit,input),(true,true),"both cancellation entrypoints must invalidate the old publication token");
}
