// SPDX-License-Identifier: LGPL-2.1-or-later
//! ABI ownership/header checks; opaque CAD bytes are schema fixtures, not kernel proof.
use openmatrix9_rust::curve::CurveSession;
use std::ffi::{c_char,CString};
#[repr(C)]struct Summary{status:u32,reserved:u32,needed:usize,generation:u64}
unsafe extern "C"{
 fn om9_curve_start(text:*const c_char)->bool;
 fn om9_curve_cancel();
 fn om9_curve_generation()->u64;
 fn om9_curve_input_revision()->u64;
 fn om9_curve_native_input_json(input:*const u8,length:usize)->u32;
 fn om9_curve_construction_json(generation:u64,output:*mut u8,capacity:usize)->Summary;
}
fn envelope(generation:u64,revision:u64,index:usize,x:f64)->String{
 let end=index+1;
 format!(r#"{{"schema_version":1,"operation":"AcceptCurveNativeInput","command":"Line","generation":"{generation}","context_before":"0","admission_revision":"{revision}","unit_context_snapshot":"1|1|1e0|1e0|1e-2|1e-3|1e-2","action":"point","text":"Point=Source.Vertex1","reference_kind":0,"point_world":[{x},0,0],"normal":[0,0,0],"points_world":[],"source_refs":[{{"document":"1","object":"2","generation":"3","revision":"18446744073709551615","epoch":"4","document_uid":"uid","object_name":"Source","subelement":"Vertex1","unit_context_snapshot":"1|1|1e0|1e0|1e-2|1e-3|1e-2","world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"ancestors":[],"role":"point","role_index":0,"point_begin":{index},"point_end":{end},"snapshot_frame":"document_world","input_snapshot_index":1}}],"source_snapshots":[{{"schema_version":1,"native_shape_type":6,"ownership":"private_native_copy","unit_context_snapshot":"1|1|1e0|1e0|1e-2|1e-3|1e-2","geometry_data":{{"representation":"curve","coordinates":"native_local_with_locations","native_root_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],"native_root_orientation":"forward","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 owned ABI schema fixture"}}}},"topology_data":{{"faces":[],"edges_3d":[],"vertices":[],"validation":{{"native_valid":true,"valid_solid":false,"closed":false}}}},"selected_points_world":[[{x},0,0]],"source_attributes":{{"schema_version":1,"document_uid":"uid","object_name":"Source","native_type":"Part::Feature","label":"Owned ABI source","label2":null,"view_provider_type":null,"visibility":true,"selectable":true,"shape_appearance":null,"coverage":{{"other_app_properties":"uninspected","rhino_user_data":"uninspected"}}}}}}]}}"#)
}
fn request(generation:u64,revision:u64,index:usize,x:f64)->String{
 // Literal native vertex evidence completes the opaque ABI schema fixture;
 // it does not turn the placeholder BREP bytes into kernel integration proof.
 let vertex=format!(r#""vertices":[{{"vertex_id":"1","native_topology_index":"1","point_xyz":[{x},0,0],"tolerance":1e-7,"geometry_coordinates":"native_local","native_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}}]"#);
 envelope(generation,revision,index,x).replace("\"native_shape_type\":6","\"native_shape_type\":7").replace("\"representation\":\"curve\"","\"representation\":\"point\"").replace("\"vertices\":[]",&vertex)
}
#[test]
fn native_json_ffi_rejects_bad_headers_and_stale_sessions_and_retains_owned_values(){unsafe{
 let name=CString::new("Line").unwrap();assert!(om9_curve_start(name.as_ptr()));let generation=om9_curve_generation();assert!(generation>0);
 assert_eq!(om9_curve_native_input_json(std::ptr::null(),1),0);
 assert_eq!(om9_curve_native_input_json(usize::MAX as *const u8,2),0);
 assert_eq!(om9_curve_native_input_json(std::ptr::null(),16*1024*1024+1),0);
 assert_eq!(om9_curve_native_input_json([0xff].as_ptr(),1),0);
 let bad=request(generation+1,om9_curve_input_revision(),0,0.);assert_eq!(om9_curve_native_input_json(bad.as_ptr(),bad.len()),0);
 let mut direct=CurveSession::default();direct.start("Line").unwrap();direct.point([0.,0.,0.]).unwrap();direct.point([10.,0.,0.]).unwrap();
 let stale=request(generation,om9_curve_input_revision(),1,10.);
 let first=request(generation,om9_curve_input_revision(),0,0.);assert_eq!(om9_curve_native_input_json(first.as_ptr(),first.len()),1);
 assert_eq!(om9_curve_native_input_json(stale.as_ptr(),stale.len()),0);
 let second=request(generation,om9_curve_input_revision(),1,10.);assert_eq!(om9_curve_native_input_json(second.as_ptr(),second.len()),2);
 let query=om9_curve_construction_json(0,std::ptr::null_mut(),0);assert_eq!(query.status,0);assert_eq!(query.generation,generation);assert_eq!(query.reserved,0);
 let mut bytes=vec![0;query.needed];let copied=om9_curve_construction_json(generation,bytes.as_mut_ptr(),bytes.len());assert_eq!(copied.status,0);
 let raw=String::from_utf8(bytes).unwrap();assert!(raw.contains("Owned ABI source")&&raw.contains("Point=Source.Vertex1")&&raw.contains("18446744073709551615"));
 assert!(raw.contains("\"normalized_points\":[[0,0,0],[10,0,0]]"));assert_eq!(direct.points(),&[[0.,0.,0.],[10.,0.,0.]]);
 om9_curve_cancel();let raw=request(generation,om9_curve_input_revision(),2,20.);assert_eq!(om9_curve_native_input_json(raw.as_ptr(),raw.len()),0);
 assert!(om9_curve_start(name.as_ptr()));assert!(om9_curve_generation()>generation);assert_eq!(om9_curve_native_input_json(raw.as_ptr(),raw.len()),0);om9_curve_cancel();
}}
