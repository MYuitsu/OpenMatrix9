use openmatrix9_rust::{ffi,native_curve_capability::om9_native_curve_capability as gate};
use std::ffi::{CStr,c_char};
unsafe extern "C"{
    fn om9_curve_start(text:*const c_char)->bool;
    fn om9_curve_input(text:*const c_char)->u32;
    fn om9_curve_point(x:f64,y:f64,z:f64)->u32;
    fn om9_curve_cancel();
}
#[test]
fn declaration_reuses_existing_catalog_without_authorizing_other_commands(){
    let mut found=[false;6];
    for index in 0..ffi::om9_command_count(){
        // Existing catalog getters return immutable, process-lifetime strings.
        let id=unsafe{CStr::from_ptr(ffi::om9_command_id(index))}.to_str().unwrap();
        let known=["Line","Polyline","InterpCrv","Rectangle","Circle","Ellipse"].iter().position(|name|*name==id);
        assert_eq!(ffi::om9_native_curve_declared(index),known.is_some(),"{id}");
        if let Some(kind)=known{found[kind]=true;}
    }
    assert!(found.into_iter().all(|value|value));assert!(!ffi::om9_native_curve_declared(usize::MAX));
}
#[test]
fn current_scalar_query_uses_actual_session_and_completed_output(){
    // One session-state test in this file; other tests only query fresh declaration.
    unsafe{om9_curve_cancel();}
    assert_eq!(gate(1,2),100);
    for stage in [0,5,u32::MAX]{assert_eq!(gate(1,stage),102);}
    unsafe{assert!(om9_curve_start(c"Line".as_ptr()));}
    assert_eq!(gate(1,1),4);assert_eq!(gate(1,2),4);assert_eq!(gate(1,4),4);
    assert_eq!(gate(2,2),101);assert_eq!(gate(1,3),102);assert_eq!(gate(u32::MAX,2),101);
    unsafe{assert_eq!(om9_curve_point(0.,0.,0.),1);assert_eq!(om9_curve_point(2.,0.,0.),2);}
    assert_eq!(gate(1,3),4);assert_eq!(gate(1,4),100);
    unsafe{assert!(om9_curve_start(c"Circle".as_ptr()));assert_eq!(om9_curve_input(c"FitPoints".as_ptr()),1);}
    for input in [1,2,3,4,5,6,7,8,9]{assert_eq!(gate(input,2),4);}
    for input in [0,10,11,12,13]{assert_eq!(gate(input,2),101);}
    assert_eq!(gate(1,3),102);unsafe{om9_curve_cancel();}assert_eq!(gate(1,3),100);
}
