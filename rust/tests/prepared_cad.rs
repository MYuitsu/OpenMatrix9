use openmatrix9_rust::prepared_cad::*;
#[test]
fn native_validity_is_required_before_commit_and_unknown_is_not_permission(){
 for admit in [om9_prepared_cad_admit,om9_native_cad_result_admit] {
  for kind in 0..8 {assert_eq!(admit(kind,1,1),3);assert_eq!(admit(kind,0,0),7);}
  for invalid in [(8,2,2),(2,3,2),(2,2,3)] {assert_eq!(admit(invalid.0,invalid.1,invalid.2),1);}
 }
}
#[test]
fn solid_admission_requires_all_observed_prerequisites_but_sheets_remain_cad(){
 for admit in [om9_prepared_cad_admit,om9_native_cad_result_admit] {
  assert_eq!(admit(2,2,2),0);
  for solid in [0,1] {assert_eq!(admit(2,2,solid),3);}
  for kind in [0,1,3,4,5,6,7] {assert_eq!(admit(kind,2,1),0);}
 }
}
#[test]
fn logical_preparation_budget_is_cumulative_checked_and_distinct_from_peak_memory(){
 assert_eq!(om9_prepared_cad_budget(4_000_000,1000,16384,64*1024*1024-1000),0);
 for values in [(4_000_001,0,1,0),(1,0,16385,0),(1,1,1,64*1024*1024),(usize::MAX,0,1,0),(0,usize::MAX,0,1)] {
  assert_eq!(om9_prepared_cad_budget(values.0,values.1,values.2,values.3),2);
 }
 assert_eq!(om9_prepared_metadata(4_000_000,64*1024*1024,64),0);
 for values in [(4_000_001,0,0),(0,64*1024*1024+1,0),(0,0,65)] {assert_eq!(om9_prepared_metadata(values.0,values.1,values.2),2);}
}
#[test]
fn metadata_numbers_cannot_invoke_native_binding_with_nonfinite_values(){
 for number in [0.,-1.,1e150] {assert_eq!(om9_prepared_number(number),0);}
 for number in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY] {assert_eq!(om9_prepared_number(number),4);}
}
#[test]
fn required_binding_fields_reject_missing_or_wrong_types_before_publication(){
 for field in 0..6 {assert_eq!(om9_prepared_binding_field(field,0,0.,0),1);}
 for field in [0,1,8] {assert_eq!(om9_prepared_binding_field(field,1,0.,0),0);assert_eq!(om9_prepared_binding_field(field,3,0.,0),1);}
 assert_eq!(om9_prepared_binding_field(2,2,1e-6,0),0);
 assert_eq!(om9_prepared_binding_field(2,6,1.,0),0);
 assert_eq!(om9_prepared_binding_field(2,1,0.,0),1);
 for value in [0.,-1.] {assert_eq!(om9_prepared_binding_field(2,2,value,0),2);}
 assert_eq!(om9_prepared_binding_field(2,2,f64::INFINITY,0),4);
 for field in [3,4] {assert_eq!(om9_prepared_binding_field(field,3,0.,0),0);assert_eq!(om9_prepared_binding_field(field,4,0.,0),1);}
 for count in [3,4] {assert_eq!(om9_prepared_binding_field(5,4,0.,count),0);}
 assert_eq!(om9_prepared_binding_field(5,4,0.,2),1);
 for value in [0.,255.,20.5] {assert_eq!(om9_prepared_binding_field(6,2,value,0),0);}
 assert_eq!(om9_prepared_binding_field(6,1,0.,0),1);
 for value in [-1.,256.] {assert_eq!(om9_prepared_binding_field(6,6,value,0),2);}
 for value in [-2147483648.,-1.,2147483647.] {assert_eq!(om9_prepared_binding_field(7,6,value,0),0);}
 assert_eq!(om9_prepared_binding_field(7,6,1099511627776.,0),2);
 assert_eq!(om9_prepared_binding_field(7,2,1.,0),1);
 assert_eq!(om9_prepared_binding_field(99,1,0.,0),1);
}
