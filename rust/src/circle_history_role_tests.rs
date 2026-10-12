// SPDX-License-Identifier: LGPL-2.1-or-later
//! Source-to-replay admission. Native snapshots stay immutable; altered numeric
//! envelopes are synthetic and do not claim fresh native output publication.
use super::*;
fn mode(name:&str)->Json{
 let cases=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/all_modes.json"));
 get(rows(&cases).iter().find(|v|get(v,"name")==&Json::String(name.into())).unwrap(),"request").clone()
}
fn admission(v:&Json)->Result<Vec<Json>,u32>{
 crate::circle_history::result::validate(crate::prepared_result::object(get(v,"operation_context")).unwrap(),crate::prepared_result::object(get(v,"snapshot")).unwrap())
}
fn resolve(v:&mut Json){
 // Recompute only the numeric envelope, so a rejection cannot be explained by
 // an inconsistent asserted plan. The expected source points come exclusively
 // from unchanged native captures, never from this replay helper.
 let c=captured_history_context(v);
 let recipe=crate::circle_history::Recipe::decode(&crate::prepared_result::array::<24>(&c["recipe"]).unwrap()).unwrap();
 let p=rows(&c["points_world"]).iter().map(|v|crate::prepared_result::array::<3>(v).unwrap()).collect::<Vec<_>>();
 let native=match &c["native_plan"]{Json::Other=>None,v=>Some(crate::prepared_result::array::<7>(v).unwrap())};
 let replay=crate::circle_history::replay(&recipe,&p,native).expect("valid synthetic numeric replay");
 c.insert("resolved_plan".into(),crate::prepared_result::map([
  ("center",Json::Array(replay.plan.center.map(Json::Number).into())),("normal",Json::Array(replay.plan.normal.map(Json::Number).into())),
  ("radius",Json::Number(replay.plan.radius)),("fit_deviation",Json::Number(replay.fit_deviation)),("approx_deviation",Json::Number(replay.approx_deviation))]));
}
#[test]
fn history_replay_roles_reject_stale_points_with_consistent_numeric_plans(){
 for(name,points)in [("Circle","[[0,0,0]]"),("Circle004","[[1,0,0],[6,0,0]]"),
 ("Circle008","[[-4,0,0],[3,0,0],[0,10,0]]"),("Circle009","[[1,0,0],[9,0,0]]"),
 ("Circle011","[[5,0,0],[0,5,0],[-5,0,0],[0,-5,0]]"),("Circle013","[[1,30,0]]")]{
  let mut v=mode(name);assert!(admission(&v).is_ok(),"baseline {name}");
  captured_history_context(&mut v).insert("points_world".into(),parse(points));resolve(&mut v);
  assert!(admission(&v).is_err(),"{name}: a valid numeric plan cannot stand in for current source points");
 }
}
#[test]
fn history_replay_roles_bind_direction_to_current_center_and_endpoint(){
 let mut v=mode("Circle002");assert!(admission(&v).is_ok());
 let c=captured_history_context(&mut v);let old=c["recorded_recipe"].clone();
 c.insert("recipe".into(),old);resolve(&mut v);
 assert!(admission(&v).is_err(),"old direction is valid numerically but current endpoint-center is(10,-45,0)");
}
#[test]
fn history_replay_roles_bind_radius_endpoint_to_current_anchor(){
 let mut v=mode("Circle006");assert!(admission(&v).is_ok());
 let c=captured_history_context(&mut v);let Json::Array(recipe)=c.get_mut("recipe").unwrap()else{panic!("recipe")};recipe[4]=Json::Number(5.);resolve(&mut v);
 assert!(admission(&v).is_err(),"current native endpoints(3,0,0)/(3,6,0) require radius6, not the old valid radius5");
}
#[test]
fn history_replay_roles_preserve_unbound_numeric_recipe_and_points(){
 for(field,value)in [("points_world","[[4,3,4]]"),("recipe","[2,0,0,2,0,1,1,0,0,1,0,0,0,1,0,0,0,1,0,3,8,0,-1,0]")]{
  let mut v=mode("Circle003");assert!(admission(&v).is_ok());
  captured_history_context(&mut v).insert(field.into(),parse(value));resolve(&mut v);
  assert!(admission(&v).is_err(),"unbound recorded {field} is immutable during replay");
 }
}
#[test]
fn history_replay_roles_reject_missing_malformed_and_overlapping_spans(){
 for(field,value)in [("recorded_source_roles","[]"),("recorded_source_roles","[0,0,1,0,0,1]"),
 ("recorded_source_roles","[1,0,1,0,1,1]"),("recorded_source_roles","[0,0,2,0,1,1]"),
 ("recorded_source_roles","[0,0,1,0,2,1]"),("recorded_source_parameters","[0]")]{
  let mut v=mode("Circle001");assert!(admission(&v).is_ok());
  captured_history_context(&mut v).insert(field.into(),parse(value));
  assert!(admission(&v).is_err(),"bad {field}={value} must not bypass source-to-input binding");
 }
 let mut v=mode("Circle");captured_history_context(&mut v).remove("recorded_recipe");
 assert!(admission(&v).is_err(),"recipe_v2 needs its actual stored recipe");
}
#[test]
fn history_replay_roles_keep_whole_fit_growth_and_enforce_retained_prefix(){
 let mut v=mode("Circle011");
 assert_eq!(rows(get(get(&v,"operation_context"),"recorded_points_world")).len(),4);
 assert_eq!(rows(get(get(&v,"operation_context"),"points_world")).len(),5);
 assert!(captured_history_assemble(&v).is_ok(),"whole source grows from4 to5");
 // Synthetic prefix of the same actual capture. Native captureReplaySources
 // truncates selected points while preserving the complete parent snapshot.
 let c=captured_history_context(&mut v);c.insert("recorded_source_roles".into(),parse("[3,0,4]"));
 let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};
 let Json::Object(snapshot)=&mut snapshots[0]else{panic!("snapshot")};
 let Json::Array(selected)=snapshot.get_mut("selected_points_world").unwrap()else{panic!("selected")};selected.truncate(4);
 assert!(admission(&v).is_err(),"prefix4 cannot produce all5 current points");
 let c=captured_history_context(&mut v);let Json::Array(points)=c.get_mut("points_world").unwrap()else{panic!("points")};points.truncate(4);resolve(&mut v);
 assert!(admission(&v).is_ok(),"retained prefix4 stays valid despite parent growth to5");
}
#[test]
fn history_replay_roles_rebuild_unsorted_fit_spans_preserving_numeric_gaps(){
 let mut v=mode("Circle011");let c=captured_history_context(&mut v);
 // Same native owner selected twice: whole source and retained prefix. The
 // complete snapshots are retained, with distinct selected point evidence.
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};let mut prefix=refs[0].clone();
 let Json::Object(r)=&mut prefix else{panic!("ref")};r.insert("role_index".into(),Json::Number(1.));r.insert("input_snapshot_index".into(),Json::Number(2.));refs.push(prefix);
 let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};let mut prefix=snapshots[0].clone();
 let Json::Object(s)=&mut prefix else{panic!("snapshot")};let Json::Array(selected)=s.get_mut("selected_points_world").unwrap()else{panic!("selected")};selected.truncate(2);snapshots.push(prefix);
 c.insert("recorded_source_roles".into(),parse("[1,3,2,3,0,2]"));c.insert("recorded_source_parameters".into(),parse("[0,0]"));
 c.insert("recorded_points_world".into(),parse("[[5,0,0],[0,5,0],[1,2,10],[-5,0,0],[0,-5,0]]"));
 let correct="[[7,0,10],[0,7,10],[1,2,10],[7,0,10],[0,7,10],[-7,0,10],[0,-7,10],[4.949747468305833,4.949747468305833,10]]";
 c.insert("points_world".into(),parse(correct));resolve(&mut v);
 assert!(admission(&v).is_ok(),"span offsets refer to original storage, not the growing current vector");
 let c=captured_history_context(&mut v);let Json::Array(points)=c.get_mut("points_world").unwrap()else{panic!("points")};points.swap(2,7);resolve(&mut v);
 assert!(admission(&v).is_err(),"numeric gap and selected points cannot be reordered");
}
fn fit_cases()->Json{parse(include_str!("../tests/fixtures/circle_history_native_source/fit_geometry98/cases.json"))}
fn fit_case(name:&str)->Json{
 get(rows(&fit_cases()).iter().find(|v|get(v,"name")==&Json::String(name.into())).unwrap(),"request").clone()
}
fn selected_snapshot(v:&mut Json)->&mut std::collections::BTreeMap<String,Json>{
 let c=captured_history_context(v);let Json::Array(s)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};
 let Json::Object(s)=&mut s[0]else{panic!("snapshot")};s
}
#[test]
fn history_fit_geometry_admits_all_ten_native_control_vertex_and_discrete_cases(){
 let cases=fit_cases();let mut failures=vec![];
 for case in rows(&cases){let v=get(case,"request");if let Err(e)=captured_history_assemble(v){failures.push((get(case,"name").clone(),e));}}
 assert!(failures.is_empty(),"native selection families rejected: {failures:?}");
}
#[test]
fn history_fit_geometry_rejects_stale_selected_points_even_with_consistent_replay(){
 let mut accepted=vec![];
 for case in rows(&fit_cases()){
  let name=get(case,"name").clone();let mut v=get(case,"request").clone();
  // Change the selected evidence and numeric replay together, leaving actual
  // native full geometry/location/stamps untouched. Generic role replay alone
  // cannot detect this; independent source geometry binding is required.
  let s=selected_snapshot(&mut v);let Json::Array(selected)=s.get_mut("selected_points_world").unwrap()else{panic!("selected")};
  let Json::Array(p)=&mut selected[0]else{panic!("point")};let Json::Number(x)=&mut p[0]else{panic!("x")};*x+=0.5;
  let points=Json::Array(selected.clone());captured_history_context(&mut v).insert("points_world".into(),points);resolve(&mut v);
  if admission(&v).is_ok(){accepted.push(name);}
 }
 assert!(accepted.is_empty(),"stale selected geometry admitted: {accepted:?}");
}
#[test]
fn history_fit_geometry_preserves_native_vertex_indices_independent_of_row_order(){
 let mut v=fit_case("CompoundVertices");assert!(admission(&v).is_ok());
 let s=selected_snapshot(&mut v);let Json::Object(t)=s.get_mut("topology_data").unwrap()else{panic!("topology")};
 let Json::Array(vertices)=t.get_mut("vertices").unwrap()else{panic!("vertices")};vertices.reverse();
 assert!(admission(&v).is_ok(),"internal topology row order does not change native vertex enumeration");
 let s=selected_snapshot(&mut v);let Json::Array(selected)=s.get_mut("selected_points_world").unwrap()else{panic!("selected")};selected.swap(0,1);
 let points=Json::Array(selected.clone());captured_history_context(&mut v).insert("points_world".into(),points);resolve(&mut v);
 assert!(admission(&v).is_err(),"the selected order must still follow native_topology_index");
}
#[test]
fn history_fit_geometry_validates_control_basis_before_accepting_poles(){
 for(name,field,value)in [("RationalNurbs","weights","[1,0,3,4]"),("PeriodicNurbs","weights","[1,-1,3,4]"),
 ("RationalBezier","degree","2"),("NurbsSurface","u_count","3"),("BezierSurface","degrees","[1,1]"),("NurbsSurface","pole_layout","\"v_major\"")]{
  let mut v=fit_case(name);assert!(admission(&v).is_ok());let s=selected_snapshot(&mut v);
  let Json::Object(t)=s.get_mut("topology_data").unwrap()else{panic!("topology")};
  let Json::Array(rows)=t.get_mut(if name.ends_with("Surface"){"faces"}else{"edges_3d"}).unwrap()else{panic!("rows")};
  let Json::Object(row)=&mut rows[0]else{panic!("row")};let Json::Object(g)=row.get_mut("geometry_data").unwrap()else{panic!("geometry")};g.insert(field.into(),parse(value));
  assert!(admission(&v).is_err(),"invalid {name} {field} cannot retain valid selected poles");
 }
}
#[test]
fn history_fit_geometry_distinguishes_whole_surface_from_retained_control_prefix(){
 let mut v=fit_case("NurbsSurface");assert!(admission(&v).is_ok());
 let s=selected_snapshot(&mut v);let Json::Array(selected)=s.get_mut("selected_points_world").unwrap()else{panic!("selected")};selected.truncate(5);
 let points=Json::Array(selected.clone());captured_history_context(&mut v).insert("points_world".into(),points.clone());resolve(&mut v);
 assert!(admission(&v).is_err(),"whole six-pole surface cannot silently supply only five poles");
 let c=captured_history_context(&mut v);c.insert("recorded_source_roles".into(),parse("[3,0,5]"));c.insert("recorded_points_world".into(),points);
 assert!(admission(&v).is_ok(),"retained prefix5 preserves full6-pole parent geometry");
}
#[test]
fn history_fit_geometry_control_selection_does_not_evaluate_restricted_edge_parameters(){
 let mut v=fit_case("PeriodicNurbs");assert!(admission(&v).is_ok());
 let s=selected_snapshot(&mut v);let Json::Object(t)=s.get_mut("topology_data").unwrap()else{panic!("topology")};
 let Json::Array(edges)=t.get_mut("edges_3d").unwrap()else{panic!("edges")};let Json::Object(edge)=&mut edges[0]else{panic!("edge")};
 // Synthetic edge restriction around genuine periodic basis/poles. Native
 // FitPoints reads poles without querying a value/D1 or using this interval.
 edge.insert("domain".into(),parse("[-4,8]"));
 assert!(admission(&v).is_ok(),"control poles do not require an edge evaluator parameter domain");
}
