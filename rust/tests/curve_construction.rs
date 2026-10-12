// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::curve::{CurveSession,Effect};
#[test]
fn only_completed_constructor_state_is_exportable_with_a_nonzero_generation(){
 let mut s=CurveSession::default();assert!(s.construction_evidence(1).is_err());
 s.start("Line").unwrap();s.input("0,0,0").unwrap();assert!(s.construction_evidence(1).is_err());
 assert_eq!(s.input("10,0,0").unwrap(),Effect::Commit);
 assert_eq!(s.construction_evidence(0),Err(5));
 let raw=s.construction_evidence(u64::MAX).unwrap();
 assert!(raw.contains(r#""generation":"18446744073709551615""#));
 assert!(raw.contains(r#""normalized_points":[[0,0,0],[10,0,0]]"#));
 s.cancel();assert!(s.construction_evidence(1).is_err());
}
#[test]
fn six_existing_constructors_preserve_options_native_plans_and_fitted_basis(){
 let cases=[("Line",vec!["BothSides=Yes","0,0,0","10,0,0"],r#""both_sides":true"#),
 ("Polyline",vec!["0,0,0","3,0,0","3,4,0","Close"],r#""output_points":[[0,0,0],[3,0,0],[3,4,0],[0,0,0]]"#),
 ("Rectangle",vec!["0,0,0","4,3,0"],r#""command":"Rectangle""#),
 ("Circle",vec!["3Point","1,0,0","0,1,0","-1,0,0"],r#""radius":1"#),
 ("Ellipse",vec!["1,2,3","6,2,3","1,5,3"],r#""major":5"#),
 ("InterpCrv",vec!["Degree=1","0,0,0","3,0,0","3,4,0",""],r#""degree":1"#)];
 for(command,inputs,expected)in cases{
  let mut s=CurveSession::default();s.start(command).unwrap();for text in inputs{s.input(text).unwrap();}
  let raw=s.construction_evidence(1).unwrap();assert!(raw.contains(expected),"{command}: {expected}");
  assert!(raw.contains(r#""input_role":"normalized_session_state_not_complete_accepted_input_history""#));
  if command=="InterpCrv"{assert!(raw.contains(r#""spline":{"#));assert!(raw.contains(r#""knots":["#));}
 }
}
#[test]
fn one_shot_close_retains_effective_closure_separately_from_the_persistent_option(){
 for command in ["Polyline","InterpCrv"]{
  let mut s=CurveSession::default();s.start(command).unwrap();
  if command=="InterpCrv"{s.input("Degree=1").unwrap();}
  for text in ["0,0,0","3,0,0","3,4,0","Close"]{s.input(text).unwrap();}
  let raw=s.construction_evidence(1).unwrap();
  assert!(raw.contains(r#""close":true"#),"effective {command} Close must survive capture");
  assert!(raw.contains(r#""persistent_close":false"#),"one-shot Close must not rewrite the persistent option");
 }
 let mut s=CurveSession::default();s.start("Polyline").unwrap();
 for text in ["PersistentClose=Yes","0,0,0","3,0,0","3,4,0",""]{s.input(text).unwrap();}
 let raw=s.construction_evidence(1).unwrap();
 assert!(raw.contains(r#""close":true"#)&&raw.contains(r#""persistent_close":true"#));
}
#[test]
fn evidence_preserves_admitted_coordinate_boundary_and_finite_constructor_output(){
 let mut s=CurveSession::default();s.start("Line").unwrap();s.point([999_999_990.,0.,0.]).unwrap();
 assert_eq!(s.point([1e9,0.,0.]).unwrap(),Effect::Commit);
 assert!(s.construction_evidence(1).is_ok(),"the existing inclusive point boundary must remain publishable");
 let mut s=CurveSession::default();s.start("Line").unwrap();s.input("BothSides=Yes").unwrap();s.point([1e9,0.,0.]).unwrap();
 assert_eq!(s.point([999_999_990.,0.,0.]).unwrap(),Effect::Commit);
 assert!(s.output().iter().any(|p|p[0]>1e9));
 assert!(s.construction_evidence(1).is_ok(),"finite generated constructor points do not impose new input admission bounds");
}
#[test]
fn synchronous_construction_ffi_matches_generation_and_rejects_without_writing(){unsafe{
 #[repr(C)] struct Summary{status:u32,reserved:u32,needed:usize,generation:u64}
 unsafe extern "C" {
  fn om9_curve_start(text:*const std::ffi::c_char)->bool;
  fn om9_curve_input(text:*const std::ffi::c_char)->u32;
  fn om9_curve_cancel();
  fn om9_curve_construction_json(generation:u64,output:*mut u8,capacity:usize)->Summary;
 }
 use std::ffi::CString;
 let start=CString::new("Line").unwrap();assert!(om9_curve_start(start.as_ptr()));
 let waiting=om9_curve_construction_json(0,std::ptr::null_mut(),0);assert_ne!(waiting.status,0);
 for text in ["0,0,0","10,0,0"]{let text=CString::new(text).unwrap();assert_ne!(om9_curve_input(text.as_ptr()),0);}
 let query=om9_curve_construction_json(0,std::ptr::null_mut(),0);assert_eq!(query.status,0);assert!(query.generation>0&&query.needed>0);assert_eq!(query.reserved,0);
 let mut bytes=vec![0xa5;query.needed];let copy=om9_curve_construction_json(query.generation,bytes.as_mut_ptr(),bytes.len());assert_eq!(copy.status,0);
 let text=String::from_utf8(bytes).unwrap();assert!(text.contains(r#""command":"Line""#));
 let mut untouched=[0xa5;8];assert_eq!(om9_curve_construction_json(query.generation,untouched.as_mut_ptr(),untouched.len()).status,2);assert_eq!(untouched,[0xa5;8]);
 assert_ne!(om9_curve_construction_json(0,untouched.as_mut_ptr(),untouched.len()).status,0);
 assert_ne!(om9_curve_construction_json(query.generation,std::ptr::null_mut(),1).status,0);
 assert_ne!(om9_curve_construction_json(query.generation,(usize::MAX-1)as *mut u8,8).status,0);
 assert!(om9_curve_start(start.as_ptr()));
 assert_eq!(om9_curve_construction_json(query.generation,untouched.as_mut_ptr(),untouched.len()).status,5);assert_eq!(untouched,[0xa5;8]);
 om9_curve_cancel();assert_ne!(om9_curve_construction_json(0,std::ptr::null_mut(),0).status,0);
}}
