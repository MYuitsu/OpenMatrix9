// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::surface_history::*;
use openmatrix9_rust::surface::Kind;
use openmatrix9_rust::native_surface_capability::Stage;

fn recipe() -> String {
    r#"{"version":1,"kind":3,"style":0,"sectionMode":0,"pointCount":16,"tolerance":0.01,"continuityA":0,"continuityB":0,"frenet":false,"closed":false,"maintainHeight":false,"preview":true,"history":true,"historyCommand":false,"matchStart":false,"matchEnd":false,"slashes":[],"inputs":[{"count":1,"chain":false,"closed":false,"reverse":true,"seam":0.25},{"count":1,"chain":false,"closed":false,"reverse":false,"seam":0}]}"#.into()
}
fn extra(raw:&str,value:&str)->String {format!("{},\"future\":{value}}}",&raw[..raw.len()-1])}

#[test]
fn saved_request_owns_options_and_grouping_independent_of_command_session() {
    let mut raw=recipe();let saved=decode_saved(&raw,2).unwrap();raw.clear();
    assert_eq!(saved.request.kind,Kind::Loft);assert_eq!(saved.request.stage,Stage::Saved);
    assert_eq!(saved.request.input_count,2);assert!(saved.request.history&&saved.request.preview);
    assert_eq!(saved.inputs[0],SavedInput{count:1,chain:false,closed:false,reverse:true,seam:0.25});
    let chain=recipe().replacen("\"count\":1,\"chain\":false","\"count\":3,\"chain\":true",1);
    assert_eq!(decode_saved(&chain,4).unwrap().request.input_count,2);
    assert!(decode_saved(&chain,2).is_err());
}
#[test]
fn saved_json_rejects_oversize_depth_nodes_duplicate_trailing_and_nonfinite() {
    for raw in ["{}".into(),recipe()+" null",recipe().replace("\"version\":1","\"version\":1,\"version\":1"),recipe().replace("0.01","1e999"),
        extra(&recipe(),&format!("\"{}\"","x".repeat(1_048_576))),
        extra(&recipe(),&format!("{}0{}","[".repeat(70),"]".repeat(70))),
        extra(&recipe(),&format!("[{}]",vec!["0";16384].join(",")))] {assert!(decode_saved(&raw,2).is_err());}
    assert!(decode_saved(&extra(&recipe(),r#"{"data":[1,true,null,"é"]}"#),2).is_ok());
}
#[test]
fn saved_byte_boundary_accepts_compatible_data_and_rejects_one_more_byte() {
    let overhead=extra(&recipe(),r#""""#).len();
    let raw=extra(&recipe(),&format!("\"{}\"","x".repeat(1_048_576-overhead)));
    assert_eq!(raw.len(),1_048_576);assert!(decode_saved(&raw,2).is_ok());
    assert!(decode_saved(&(raw+" "),2).is_err());
}
#[test]
fn persistent_numbers_groups_and_link_cardinality_cannot_reach_native_capture() {
    for (from,to) in [("\"seam\":0.25","\"seam\":1"),("\"count\":1","\"count\":2"),("\"pointCount\":16","\"pointCount\":16.5"),
        ("\"tolerance\":0.01","\"tolerance\":0.00000001"),("\"reverse\":true","\"reverse\":1"),("\"version\":1","\"version\":2")] {
        assert!(decode_saved(&recipe().replacen(from,to,1),2).is_err(),"{from} -> {to}");
    }
    for count in [0,1,3,usize::MAX] {assert!(decode_saved(&recipe(),count).is_err());}
}
#[test]
fn saved_policy_reuses_actual_loft_sweep_options_and_closed_rail_rules() {
    assert!(decode_saved(&recipe().replace("\"kind\":3","\"kind\":1"),2).is_ok());
    assert!(decode_saved(&recipe().replace("\"kind\":3","\"kind\":2"),2).is_err());
    assert!(decode_saved(&recipe().replace("\"kind\":3","\"kind\":1").replace("\"style\":0","\"style\":1"),2).is_err());
    assert!(decode_saved(&recipe().replacen("\"closed\":false","\"closed\":true",1),2).is_err());
    assert!(decode_saved(&recipe().replace("\"maintainHeight\":false","\"maintainHeight\":true"),2).is_err());
    assert!(decode_saved(&recipe().replace("\"matchStart\":false","\"matchStart\":true"),2).is_ok());
}
#[test]
fn sweep2_saved_slashes_and_closed_rails_use_group_metadata() {
    let row=r#"{"count":1,"chain":false,"closed":false,"reverse":false,"seam":0}"#;
    let prefix=recipe().replace("\"kind\":3","\"kind\":2");
    let raw=format!("{}\"inputs\":[{row},{row},{row}]}}",prefix.split("\"inputs\":").next().unwrap());
    let slashed=raw.replace("\"slashes\":[]","\"slashes\":[[0.2,0.3],[0.8,0.9]]");
    let saved=decode_saved(&slashed,3).unwrap();assert_eq!(saved.request.slashes,vec![(0.2,0.3),(0.8,0.9)]);
    assert!(decode_saved(&slashed.replace("0.8,0.9","0.1,0.9"),3).is_err());
    assert!(decode_saved(&slashed.replacen("\"closed\":false","\"closed\":true",1),3).is_err());
    // Closed Sweep2 needs two closed rails and two sections (four logical inputs).
    let row_closed=row.replace("\"closed\":false","\"closed\":true");
    let prefix=prefix.split("\"inputs\":").next().unwrap().replacen("\"closed\":false","\"closed\":true",1);
    let closed=format!("{prefix}\"inputs\":[{row_closed},{row_closed},{row},{row}]}}");
    assert_eq!(decode_saved(&closed,4).unwrap().request.rail_closed,[true,true]);
}
#[test]
fn saved_rail_hints_do_not_override_current_native_closedness() {
    let row=r#"{"count":1,"chain":false,"closed":false,"reverse":false,"seam":0}"#;
    let raw=recipe().replace("\"kind\":3","\"kind\":1").replacen("\"closed\":false","\"closed\":true",1);
    let closed=format!("{}\"inputs\":[{row},{row},{row}]}}",raw.split("\"inputs\":").next().unwrap());
    assert!(decode_saved(&closed,3).is_ok(),"live closed rail must be admitted despite a stale false hint");
    let raw=recipe().replace("\"kind\":3","\"kind\":2").replace("\"slashes\":[]","\"slashes\":[[0.2,0.3]]");
    let row_hint=row.replace("\"closed\":false","\"closed\":true");
    let slashed=format!("{}\"inputs\":[{row_hint},{row_hint},{row}]}}",raw.split("\"inputs\":").next().unwrap());
    assert!(decode_saved(&slashed,3).is_ok(),"live open rails must be admitted despite stale true hints");
}
#[test]
fn borrowed_abi_rejects_bad_envelopes_without_touching_output() {
    let raw=recipe();let mut out=NativeSavedSurface::default();out.kind=99;
    for (p,n,count) in [(std::ptr::null(),raw.len(),2),(1usize as *const u8,1_048_577,2),(raw.as_ptr(),raw.len(),usize::MAX)] {
        assert_eq!(unsafe{om9_surface_history_decode(p,n,count,&mut out)},102);assert_eq!(out.kind,99);
    }
    assert_eq!(unsafe{om9_surface_history_decode(raw.as_ptr(),raw.len(),2,std::ptr::null_mut())},102);
    let invalid=b"{}";assert_eq!(unsafe{om9_surface_history_decode(invalid.as_ptr(),invalid.len(),2,&mut out)},102);assert_eq!(out.kind,99);
    let invalid_utf8=[255u8];assert_eq!(unsafe{om9_surface_history_decode(invalid_utf8.as_ptr(),1,2,&mut out)},102);
    let embedded_nul=raw.clone()+"\0";
    assert_eq!(unsafe{om9_surface_history_decode(embedded_nul.as_ptr(),embedded_nul.len(),2,&mut out)},102);assert_eq!(out.kind,99);
    assert_eq!(unsafe{om9_surface_history_decode(raw.as_ptr(),raw.len(),2,&mut out)},0);
    assert_eq!(out.kind,3);assert_eq!(out.group_count,2);assert_eq!(out.inputs[0].flags,4);assert_eq!(out.inputs[0].seam,0.25);
    assert_eq!(out.flags,160);assert_eq!(std::mem::size_of::<NativeSavedSurface>(),5168);
}
#[test]
fn rejected_complete_request_reports_operation_without_writing_output() {
    let raw=recipe();let row=r#"{"count":1,"chain":false,"closed":false,"reverse":false,"seam":0}"#;
    let one=format!("{}\"inputs\":[{row}]}}",raw.split("\"inputs\":").next().unwrap());
    let mut out=NativeSavedSurface::default();out.kind=99;
    assert_eq!(unsafe{om9_surface_history_decode(one.as_ptr(),one.len(),1,&mut out)},203);
    assert_eq!(out.kind,99);assert!(decode_saved(&one,1).is_err());
    for (kind,want) in [(1,201),(2,202)]{
        let raw=one.replace("\"kind\":3",&format!("\"kind\":{kind}"));
        assert_eq!(unsafe{om9_surface_history_decode(raw.as_ptr(),raw.len(),1,&mut out)},want);assert_eq!(out.kind,99);
    }
}
