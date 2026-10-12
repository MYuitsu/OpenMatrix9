// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::prepared_result::assemble;
fn attrs()->&'static str {r#"{"schema_version":1,"document_uid":"doc-uuid","object_name":"Source","native_type":"Part::Feature","label":"Owned source","label2":null,"view_provider_type":null,"visibility":true,"selectable":true,"shape_appearance":null,"coverage":{"other_app_properties":"uninspected","rhino_user_data":"uninspected"}}"#}
fn snapshot(kind:u32,attributes:&str)->String {
 let representation=if kind==5{"polycurve"}else{"trimmed_face"};let faces=if kind==5{"[]"}else{r#"[{"native_topology_index":"1"}]"#};
 format!(r#"{{"schema_version":1,"native_shape_type":{kind},"ownership":"private_native_copy","unit_context_snapshot":"","geometry_data":{{"representation":"{representation}","coordinates":"native_local_with_locations","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 native fixture"}}}},"topology_data":{{"faces":{faces},"edges_3d":[],"vertices":[],"validation":{{"native_valid":true,"valid_solid":false}}}},"source_attributes":{attributes}}}"#)
}
fn recipe()->&'static str {r#"{"version":1,"kind":3,"style":0,"sectionMode":0,"pointCount":16,"tolerance":0.01,"continuityA":0,"continuityB":0,"frenet":false,"closed":false,"maintainHeight":false,"preview":true,"history":true,"historyCommand":false,"matchStart":false,"matchEnd":false,"slashes":[],"inputs":[{"count":1,"chain":false,"closed":false,"reverse":false,"seam":0},{"count":1,"chain":false,"closed":false,"reverse":false,"seam":0}]}"#}
fn quote(s:&str)->String {format!("\"{}\"",s.replace('\\',"\\\\").replace('"',"\\\""))}
fn source(index:u32)->String {format!(r#"{{"document":"1","object":"{index}","generation":"3","revision":"18446744073709551615","epoch":"4","document_uid":"doc-uuid","object_name":"Source","subelement":"Edge1","unit_context_snapshot":"","world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"ancestors":[]}}"#)}
pub fn request()->String {
 let input=snapshot(5,attrs());let output=snapshot(4,r#"{"schema_version":1,"availability":"not_applicable_constructed_output"}"#);
 let recipe=quote(recipe());let a=source(2);let b=source(3);
 format!(r#"{{"snapshot":{output},"input_identity":{{"document_name":"fixture","name":"Surface"}},"validation":{{"native_valid":true,"valid_solid":false}},"operation_context":{{"schema_version":1,"operation":"EditSurface","feature_id":"OM9-SURFACE-009","recipe":{recipe},"rail_closed":0,"source_refs":[{a},{b}],"source_snapshots":[{input},{input}],"source_snapshot_frame":"document_world","output_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"attribute_role":"source_evidence_in_input_snapshots_output_attributes_unavailable"}}}}"#)
}
#[test]
fn surface_result_retains_real_operation_recipe_and_owned_sources() {
 let output=assemble(&request()).expect("existing native Surface needs its own producer");
 assert!(output.contains(r#""operation":"EditSurface""#));assert!(output.contains(r#""input_snapshots":[{"#));
 assert!(output.contains("Owned source"));assert!(output.contains("OM9-SURFACE-009"));assert!(output.contains("18446744073709551615"));
 assert!(!output.contains("upstream_archive_conversion"));assert!(!output.contains("NativeBuilder"));
 assert!(output.contains(r#""exactness":"uninspected""#));assert!(output.contains(r#""topology_map":null"#));
}
#[test]
fn malformed_source_metadata_or_result_cannot_publish_a_surface() {
 for (from,to) in [("OM9-SURFACE-009","OM9-SURFACE-001"),("\"rail_closed\":0","\"rail_closed\":4"),
  ("\"source_snapshot_frame\":\"document_world\"","\"source_snapshot_frame\":\"object_local\""),
  ("\"generation\":\"3\"","\"generation\":\"0\""),("\"native_valid\":true","\"native_valid\":false"),
  ("\"availability\":\"not_applicable_constructed_output\"","\"availability\":\"unavailable_legacy_record\""),
  ("\"object_name\":\"Source\"","\"object_name\":\"Other\""),
  ("\"source_refs\":[","\"source_refs\":[{},"),("\"source_snapshots\":[","\"source_snapshots\":[{},")]
 {assert!(assemble(&request().replacen(from,to,1)).is_err(),"{from}");}
}
