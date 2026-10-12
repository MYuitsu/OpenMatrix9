use openmatrix9_rust::prepared_result::assemble;
fn request(mesh:bool)->String {
 let data=if mesh {r#"{"representation":"mesh","kind":"mesh","coordinates":"native_local_with_locations","points_xyz":[[0,0,0],[2,0,0],[0,3,0]],"triangles":[[0,1,2]],"native_discrete_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],"native_storage_precision":"float32","source_archive_mesh":{"vertices":[[0,0,0],[2,0,0],[0,3,0]],"faces":[[0,1,2,2]]}}"#} else {r#"{"representation":"point_cloud","coordinates":"native_local","current_fields":{"schema_version":1,"points":[[10000000.00000001,2,3],[4,5,6]],"normals":[[0,0,1],[0,1,0]],"colors":[20,40,60,0,80,100,120,10],"values":[0.25,0.75],"plane":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"ordered":true,"has_plane":false},"native_storage_precision":"float64"}"#};
 let ownership=if mesh{"private_native_copy"}else{"private_plain_copy"};
 format!(r#"{{"snapshot":{{"schema_version":1,"ownership":"{ownership}","unit_context_snapshot":"","geometry_data":{data},"topology_data":{{"discrete_kind":"{}"}},"source_attributes":{{"name":"Điểm","layer":"fixture","locked":false,"visible":true,"tolerance":0.000001,"color":[20,40,60]}}}},"input_identity":{{"document_name":"fixture","name":"Điểm"}},"validation":{{"native_plane_valid":true,"native_valid":null,"valid_solid":false}}}}"#,if mesh{"mesh"}else{"point_cloud"})
}
#[test]
fn mesh_result_owns_native_triangles_archive_observations_and_representation(){
 let mut input=request(true);let result=assemble(&input).unwrap();input.clear();
 for expected in ["\"representation\":\"mesh\"","\"triangles\":[[0,1,2]]","\"source_archive_mesh\":","\"native_storage_precision\":\"float32\"","\"unit_context_revision\":null","\"exactness\":\"uninspected\""]{assert!(result.contains(expected),"{expected}");}
}
#[test]
fn cloud_result_owns_full_float64_current_fields_without_mesh_conversion(){
 let mut input=request(false);let result=assemble(&input).unwrap();input.clear();
 for expected in ["10000000.00000001","\"representation\":\"point_cloud\"","\"normals\":[[0,0,1],[0,1,0]]","\"values\":[0.25,0.75]","\"ordered\":true","\"native_storage_precision\":\"float64\"","\"ownership\":\"private_plain_copy\""]{assert!(result.contains(expected),"{expected}");}
}
#[test]
fn invalid_mesh_indices_and_archive_schema_reject_with_typed_error(){
 let input=request(true);
 for (before,after,code) in [("\"triangles\":[[0,1,2]]","\"triangles\":[[0,1,99]]",3),("\"faces\":[[0,1,2,2]]","\"faces\":[[0,1,99,99]]",3),("\"vertices\":","\"unknown_vertices\":",1),("\"private_native_copy\"","\"borrowed\"",1)]{assert_eq!(assemble(&input.replace(before,after)),Err(code),"{before}");}
}
#[test]
fn cloud_channels_flags_and_schema_require_complete_existing_contract(){
 let input=request(false);
 for(before,after,code)in[("\"normals\":[[0,0,1],[0,1,0]]","\"normals\":[[0,0,1]]",2),("\"colors\":[20,40,60,0,80,100,120,10]","\"colors\":[1,2,3]",2),("\"values\":[0.25,0.75]","\"values\":[0.25]",2),("\"ordered\":true","\"ordered\":1",1),("\"schema_version\":1,\"points\"","\"schema_version\":2,\"points\"",1),("\"has_plane\":false","\"has_plane\":true",0)]{
  if code==0{assert!(assemble(&input.replace(before,after)).is_ok());}else{assert_eq!(assemble(&input.replace(before,after)),Err(code),"{before}");}
 }
 let active=input.replace("\"has_plane\":false","\"has_plane\":true").replace("\"native_plane_valid\":true","\"native_plane_valid\":false");assert_eq!(assemble(&active),Err(2));
}
#[test]
fn discrete_count_and_coordinate_limits_apply_before_acceptance(){
 let input=request(false);
 let points=std::iter::repeat_n("[0,0,0]",100_001).collect::<Vec<_>>().join(",");
 let oversized=input.replace("\"points\":[[10000000.00000001,2,3],[4,5,6]]",&format!("\"points\":[{points}]"));assert_eq!(assemble(&oversized),Err(2));
 assert_eq!(assemble(&input.replace("10000000.00000001","1e151")),Err(4));
 assert_eq!(assemble(&input.replace("\"points\":[[10000000.00000001,2,3],[4,5,6]]","\"points\":[]")),Err(2));
}
#[test]
fn explicit_units_keep_full_revision_for_owned_discrete_results(){
 let raw=request(false).replace("\"unit_context_snapshot\":\"\"","\"unit_context_snapshot\":\"1|18446744073709551615|25.4|1|1e-6|1e-6|1e-3\"");
 assert!(assemble(&raw).unwrap().contains("\"unit_context_revision\":\"18446744073709551615\""));
}
#[test]
fn native_segment_indices_use_the_rust_scalar_policy(){
 use openmatrix9_rust::prepared_cad::om9_prepared_binding_field as field;
 assert_eq!(field(9,6,0.,1),0);assert_eq!(field(9,6,9.,10),0);
 for value in [-1.,0.5,10.] {assert_eq!(field(9,6,value,10),3);}
 assert_eq!(field(9,6,0.,0),3);assert_eq!(field(9,6,f64::NAN,10),4);
 assert_eq!(field(9,2,1.,10),1);assert_eq!(field(9,6,1.,100_001),2);
}
#[test]
fn mesh_archive_must_describe_the_same_oriented_triangles(){
 let input=request(true);
 for (before,after) in [
  ("\"vertices\":[[0,0,0],[2,0,0],[0,3,0]]","\"vertices\":[[10,0,0],[12,0,0],[10,3,0]]"),
  ("\"faces\":[[0,1,2,2]]","\"faces\":[[0,2,1,1]]"),
  ("\"faces\":[[0,1,2,2]]","\"faces\":[[0,1,2,2],[0,1,2,2]]"),
  ("[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]","[[1,0,0,10],[0,1,0,0],[0,0,1,0],[0,0,0,1]]")
 ] {assert_eq!(assemble(&input.replace(before,after)),Err(8),"{before}");}
}
#[test]
fn mesh_archive_preserves_quad_precision_and_unused_vertices_with_reindexed_native_data(){
 let input=request(true)
  .replace("\"points_xyz\":[[0,0,0],[2,0,0],[0,3,0]]","\"points_xyz\":[[2,0,0],[2,3,0],[0,3,0],[0,0,0]]")
  .replace("\"triangles\":[[0,1,2]]","\"triangles\":[[0,1,3],[1,2,3]]")
  .replace("\"vertices\":[[0,0,0],[2,0,0],[0,3,0]]","\"vertices\":[[0,0,0],[2.00000001,0,0],[2,3,0],[0,3,0],[99,99,99]]")
  .replace("\"faces\":[[0,1,2,2]]","\"faces\":[[0,1,2,3]]");
 assert!(assemble(&input).is_ok());
 assert!(assemble(&input.replace("\"triangles\":[[0,1,3],[1,2,3]]","\"triangles\":[[3,1,2],[1,3,0]]")).is_ok());
}
#[test]
fn mesh_archive_correspondence_applies_native_location_once(){
 let input=request(true)
  .replace("\"vertices\":[[0,0,0],[2,0,0],[0,3,0]]","\"vertices\":[[10,0,0],[12,0,0],[10,3,0]]")
  .replace("[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]","[[1,0,0,10],[0,1,0,0],[0,0,1,0],[0,0,0,1]]");
 assert!(assemble(&input).is_ok());
}
