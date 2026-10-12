// SPDX-License-Identifier: LGPL-2.1-or-later
//! Real admission API around immutable native captures, not intercepted packets.
use super::*;
fn parse(s:&str)->Json{parse_document(s,16*1024*1024,1_000_000).unwrap()}
fn fits()->Json{parse(include_str!("../tests/fixtures/circle_history_native_source/fit_geometry98/cases.json"))}
fn center()->Json{
 let a=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/all_modes.json"));
 get(&rows(&a)[0],"request").clone()
}
fn packet(s:&CurveSession,context:&Json,action:&str,kind:u32,p:[f64;3])->Json{
 let c=get(context,"operation_context");let refs=rows(get(c,"source_refs"));let snaps=rows(get(c,"source_snapshots"));
 let begin=s.points().len();let selected=get(&snaps[0],"selected_points_world").clone();
 let mut reference=refs[0].clone();let Json::Object(r)=&mut reference else{panic!("reference")};
 r.insert("role".into(),string(if action=="fit_batch"{"fit_points"}else if kind==4{"direction"}else{"point"}));
 r.insert("role_index".into(),Json::Number(0.));r.insert("point_begin".into(),Json::Number(begin as f64));
 r.insert("point_end".into(),Json::Number((begin+if action=="fit_batch"{rows(&selected).len()}else{1})as f64));
 r.insert("input_snapshot_index".into(),Json::Number(1.));
 map([("schema_version",Json::Number(1.)),("operation",string("AcceptCurveNativeInput")),("command",string(s.name)),
  ("generation",string(&u64::MAX.to_string())),("admission_revision",string(&s.native_input_revision().to_string())),("context_before",string(&s.circle_input_context_key().to_string())),
  ("unit_context_snapshot",string(UNITS)),("action",string(action)),("reference_kind",Json::Number(kind as f64)),
  ("point_world",point(p)),("normal",point([0.,0.,1.])),("points_world",if action=="fit_batch"{selected}else{Json::Array(vec![])}),
  ("text",string(if action=="fit_batch"{"Selection"}else{"Point=CenterSource.Vertex1"})),("source_refs",Json::Array(vec![reference])),("source_snapshots",Json::Array(vec![snaps[0].clone()]))])
}
fn state(s:&CurveSession)->(String,Json,Vec<[f64;3]>,u64,u64){(s.prompt(),s.accepted_input_log.json(),s.points().to_vec(),s.native_input_revision(),s.circle_input_context_key())}
fn tamper(v:&mut Json,fit:bool){
 let Json::Object(o)=v else{panic!("request")};let Json::Array(a)=o.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};
 let Json::Object(s)=&mut a[0]else{panic!("snapshot")};let Json::Array(p)=s.get_mut("selected_points_world").unwrap()else{panic!("selected")};
 let Json::Array(first)=&mut p[0]else{panic!("xyz")};let Json::Number(x)=&mut first[0]else{panic!("x")};*x+=0.5;
 let points=p.clone();o.insert(if fit{"points_world"}else{"point_world"}.into(),if fit{Json::Array(points)}else{points[0].clone()});
}
#[test]
fn initial_input_owned_selection_accepts_ten_native_fit_families_and_retains_sources(){
 for case in rows(&fits()){
  let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
  let p=packet(&s,get(case,"request"),"fit_batch",0,[0.;3]);s.accept_native_json(&encode_owned(&p).unwrap(),u64::MAX).expect("native full selection");
  assert_eq!(super::super::input_log::points(s.points()),*get(&p,"points_world"));
  s.input("").unwrap();let c=parse(&s.construction_evidence(u64::MAX).unwrap());
  assert_eq!(rows(get(&c,"source_snapshots"))[0],rows(get(&p,"source_snapshots"))[0]);
 }
}
#[test]
fn initial_input_owned_selection_rejects_stale_fit_geometry_atomically(){
 let mut failures=vec![];
 for case in rows(&fits()){
  let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();let before=state(&s);
  let good=packet(&s,get(case,"request"),"fit_batch",0,[0.;3]);let mut bad=good.clone();tamper(&mut bad,true);
  if s.accept_native_json(&encode_owned(&bad).unwrap(),u64::MAX).is_ok(){failures.push(get(case,"name").clone());continue;}
  assert_eq!(state(&s),before,"rejection preserves options/log/points/revision/context");
  s.accept_native_json(&encode_owned(&good).unwrap(),u64::MAX).expect("same capture remains valid after rejection");
 }
 assert!(failures.is_empty(),"stale selected full geometry accepted: {failures:?}");
}
#[test]
fn initial_input_owned_selection_binds_native_vertices_for_all_six_constructors(){
 for name in ["Line","Polyline","Rectangle","InterpCrv","Circle","Ellipse"]{
  let mut s=CurveSession::default();s.start(name).unwrap();let good=packet(&s,&center(),"point",0,[1.,45.,0.]);
  let mut bad=good.clone();tamper(&mut bad,false);let before=state(&s);
  assert!(s.accept_native_json(&encode_owned(&bad).unwrap(),u64::MAX).is_err(),"{name}: selected and input points cannot jointly detach from owned Vertex1");
  assert_eq!(state(&s),before);s.accept_native_json(&encode_owned(&good).unwrap(),u64::MAX).expect(name);
  assert_eq!(s.points(),&[[1.,45.,0.]]);
 }
}
#[test]
fn initial_input_owned_selection_binds_direction_and_fixed_point_reference_actions(){
 for(kind,setup)in [(4,vec!["0,0,0","Orientation"]),(5,vec!["Tangent","Radius=4"]) ]{
  let mut s=CurveSession::default();s.start("Circle").unwrap();for input in setup{s.input(input).unwrap();}
  let good=packet(&s,&center(),"reference",kind,[1.,45.,0.]);let mut bad=good.clone();tamper(&mut bad,false);let before=state(&s);
  assert!(s.accept_native_json(&encode_owned(&bad).unwrap(),u64::MAX).is_err(),"reference kind{kind} must bind actual owned Vertex1");
  assert_eq!(state(&s),before);s.accept_native_json(&encode_owned(&good).unwrap(),u64::MAX).expect("valid vertex reference");
 }
}
#[test]
fn initial_input_owned_selection_preserves_large_local_basis_with_valid_world_points(){
 let native=parse(include_str!("../tests/fixtures/curve_native_selection/local_world105.json"));let v=get(&native,"poles");
 let context=map([("operation_context",map([("source_refs",get(get(v,"operation_context"),"source_refs").clone()),
  ("source_snapshots",Json::Array(rows(get(v,"input_snapshots"))[1..].to_vec()))]))]);
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 let p=packet(&s,&context,"fit_batch",0,[0.;3]);
 s.accept_native_json(&encode_owned(&p).unwrap(),u64::MAX).expect("basis policy accepts local1e12; transformed world points are only ±5");
 assert_eq!(s.points(),&[[5.,0.,0.],[0.,5.,0.],[-5.,0.,0.],[0.,-5.,0.]]);
}
