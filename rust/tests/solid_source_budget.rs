// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::solid_result::om9_solid_source_budget_json;
fn admit(raw:&str)->u32{unsafe{om9_solid_source_budget_json(raw.as_ptr(),raw.len())}}
#[test]
fn aggregate_source_envelope_rejects_nodes_before_selected_batch_acceptance(){
 let large=format!(r#"{{"source_refs":[{{}}],"source_snapshots":[[{}]]}}"#,vec!["0";1_000_000].join(","));
 assert!(large.len()<16*1024*1024);
 assert_eq!(admit(&large),2,"under the byte bound does not imply under the shared parser node bound");
 assert_eq!(admit(r#"{"source_refs":[{},{}],"source_snapshots":[{"owned_master":"same source"}]}"#),0,"explicit sharing keeps every range without multiplying source data");
}
#[test]
fn envelope_shape_depth_bytes_and_ffi_headers_are_bounded(){
 assert_eq!(admit(r#"{"source_refs":[],"source_snapshots":[]}"#),0);
 for raw in ["{}",r#"{"source_refs":[],"source_snapshots":[{}]}"#,r#"{"source_refs":null,"source_snapshots":[]}"#]{assert_ne!(admit(raw),0);}
 let deep=format!(r#"{{"source_refs":[{{}}],"source_snapshots":[{}]}}"#,format!("{}0{}","[".repeat(64),"]".repeat(64)));
 assert_eq!(admit(&deep),2,"full request adds a context ancestor to the envelope");
 let raw=r#"{"source_refs":[],"source_snapshots":[]}"#;
 unsafe{
  assert_ne!(om9_solid_source_budget_json(std::ptr::null(),raw.len()),0);
  assert_ne!(om9_solid_source_budget_json(raw.as_ptr(),16*1024*1024+1),0);
  assert_ne!(om9_solid_source_budget_json((usize::MAX-1) as *const u8,8),0);
 }
}
#[test]
fn completed_request_node_budget_reports_invalid_basis(){
 let raw=format!(r#"{{"snapshot":{{"topology_data":{{"vertices":[{}]}}}}}}"#,vec!["0";1_000_000].join(","));
 assert_eq!(openmatrix9_rust::prepared_result::assemble(&raw),Err(2),"resource exhaustion must not claim an unsupported representation");
}
