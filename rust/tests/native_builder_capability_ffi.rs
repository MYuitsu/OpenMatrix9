// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::native_builder_capability::om9_native_builder_capability as query;
const ID:&[u8]=b"OM9-SETTING-001";
const RAW:&[u8]=br#"{"schema":1,"evaluator":"om9.affine-template","version":1,"settings":{}}"#;
unsafe fn call(representation:u32,stage:u32,id:*const u8,id_len:usize,raw:*const u8,raw_len:usize,dimensions:*const f64,count:usize)->u32{
    unsafe{query(representation,stage,id,id_len,raw,raw_len,1,1,1,1,dimensions,count)}
}
#[test]
fn explicit_bytes_and_actual_dimensions_admit_existing_affine_backend(){
    let dimensions=[2.,4.,8.,4.,4.,4.];
    for representation in 1..=7{unsafe{
        assert_eq!(call(representation,2,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),std::ptr::null(),0),4);
        assert_eq!(call(representation,3,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),dimensions.as_ptr(),6),4);
    }}
    for representation in [0,8,9,10,11,12,13,14,u32::MAX]{unsafe{
        assert_eq!(call(representation,2,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),std::ptr::null(),0),101);
    }}
}
#[test]
fn bad_foreign_headers_are_rejected_before_reading_or_copying(){
    let storage=[0.;7];let shifted=unsafe{storage.as_ptr().cast::<u8>().add(1)}.cast::<f64>();
    for (id,id_len,raw,raw_len,dimensions,count) in [
        (std::ptr::null(),ID.len(),RAW.as_ptr(),RAW.len(),std::ptr::null(),0),
        (ID.as_ptr(),usize::MAX,RAW.as_ptr(),RAW.len(),std::ptr::null(),0),
        (ID.as_ptr(),0,RAW.as_ptr(),RAW.len(),std::ptr::null(),0),
        (ID.as_ptr(),ID.len(),std::ptr::null(),RAW.len(),std::ptr::null(),0),
        (ID.as_ptr(),ID.len(),RAW.as_ptr(),1_048_577,std::ptr::null(),0),
        (ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),std::ptr::null(),6),
        (ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),shifted,6),
        (ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),storage.as_ptr(),usize::MAX),
        ((usize::MAX-2) as *const u8,ID.len(),RAW.as_ptr(),RAW.len(),std::ptr::null(),0),
    ]{unsafe{assert_eq!(call(6,2,id,id_len,raw,raw_len,dimensions,count),102);}}
}
#[test]
fn utf8_flags_stage_and_solve_options_share_existing_limits(){unsafe{
    assert_eq!(call(6,2,ID.as_ptr(),ID.len(),b"\xff".as_ptr(),1,std::ptr::null(),0),102);
    assert_eq!(call(6,99,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),std::ptr::null(),0),102);
    assert_eq!(call(6,3,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),std::ptr::null(),0),102);
    assert_eq!(query(6,2,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),1,1,1,2,std::ptr::null(),0),102);
    assert_eq!(query(6,2,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),1,1025,1,0,std::ptr::null(),0),102);
    let dimensions=[2.,4.,8.,f64::NAN,4.,4.];
    assert_eq!(call(6,3,ID.as_ptr(),ID.len(),RAW.as_ptr(),RAW.len(),dimensions.as_ptr(),6),102);
}}
