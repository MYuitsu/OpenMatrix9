// SPDX-License-Identifier: LGPL-2.1-or-later
//! Initial path/probe correspondence witnesses around immutable native sources.
//! Added fraction fields below are analytically derived test metadata, not a
//! claim that current C++ producers already emit them.
use super::*;
fn parse(s:&str)->Json{parse_document(s,16*1024*1024,1_000_000).unwrap()}
fn captured(name:&str)->Json{parse(match name{
 "around"=>include_str!("../tests/fixtures/circle_history_native_source/contact_replay/around.json"),
 "tangent"=>include_str!("../tests/fixtures/circle_history_native_source/contact_replay/tangent.json"),_=>panic!("case"),
})}
fn packet(s:&CurveSession,case:&Json,kind:u32,p:[f64;3],n:[f64;3],fraction:Option<f64>)->Json{
 let c=get(case,"operation_context");let mut r=rows(get(c,"source_refs"))[0].clone();let mut snapshot=rows(get(c,"source_snapshots"))[0].clone();
 if kind==3{let Json::Object(o)=&mut snapshot else{panic!("snapshot")};o.insert("selected_points_world".into(),Json::Array(vec![rows(get(c,"points_world"))[0].clone()]));}
 let Json::Object(refs)=&mut r else{panic!("reference")};refs.insert("role".into(),string(if kind==1{"path"}else{"tangent"}));
 refs.insert("role_index".into(),Json::Number(0.));refs.insert("point_begin".into(),Json::Number(0.));refs.insert("point_end".into(),Json::Number(0.));refs.insert("input_snapshot_index".into(),Json::Number(1.));
 let sources=if kind==2{(vec![],vec![])}else{(vec![r],vec![snapshot])};
 let mut v=map([("schema_version",Json::Number(1.)),("operation",string("AcceptCurveNativeInput")),("command",string(s.name)),
 ("generation",string(&u64::MAX.to_string())),("context_before",string(&s.circle_input_context_key().to_string())),("admission_revision",string(&s.native_input_revision().to_string())),
 ("unit_context_snapshot",string(UNITS)),("action",string("reference")),("reference_kind",Json::Number(kind as f64)),
 ("point_world",point(p)),("normal",point(n)),("points_world",Json::Array(vec![])),
 ("text",string(if kind==2{"OnCurve=0.25"}else if kind==1{"Curve=DeformablePath.Edge1"}else{"Curve=TangentA.Edge1@1,4,0"})),
 ("source_refs",Json::Array(sources.0)),("source_snapshots",Json::Array(sources.1))]);
 if let Some(f)=fraction{let Json::Object(o)=&mut v else{panic!("packet")};o.insert("source_curve_fraction".into(),Json::Number(f));}
 v
}
fn state(s:&CurveSession)->(String,Json,Vec<[f64;3]>,u64,u64){(s.prompt(),s.accepted_input_log.json(),s.points().to_vec(),s.native_input_revision(),s.circle_input_context_key())}
fn path_session(name:&str)->CurveSession{
 let mut s=CurveSession::default();s.start(name).unwrap();s.input("AroundCurve").unwrap();
 let v=packet(&s,&captured("around"),1,[0.;3],[0.;3],None);s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap();s
}
#[test]
fn initial_curve_parameters_accept_actual_owned_path_and_probe_geometry(){
 for name in ["Circle","Ellipse"]{
  let mut s=path_session(name);let v=packet(&s,&captured("around"),2,[5.,0.,0.],[1.,0.,0.],Some(0.25));
  s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).expect("actual line domain0..20 at fraction.25");
 }
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("Tangent").unwrap();
 let v=packet(&s,&captured("tangent"),3,[1.,4.,0.],[0.,1.,0.],Some(0.6));
 s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).expect("actual vertical line probe at fraction.6");
}
#[test]
fn initial_curve_parameters_bind_oncurve_point_normal_and_declared_fraction(){
 let mut accepted=vec![];
 for name in ["Circle","Ellipse"]{for(label,p,n,f)in [
  ("wrong position",[7.,0.,0.],[1.,0.,0.],0.25),
  ("wrong normal",[5.,0.,0.],[0.,1.,0.],0.25),
  ("wrong fraction",[5.,0.,0.],[1.,0.,0.],0.5),
 ]{
  let mut s=path_session(name);let before=state(&s);let v=packet(&s,&captured("around"),2,p,n,Some(f));
  if s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).is_ok(){accepted.push(format!("{name}: {label}"));}else{assert_eq!(state(&s),before);}
 }}
 assert!(accepted.is_empty(),"path evidence detached from native source: {accepted:?}");
}
#[test]
fn initial_curve_parameters_bind_tangent_probe_and_direction_to_owned_curve(){
 let mut accepted=vec![];
 for(label,p,n,f)in [
  ("off curve",[2.,4.,0.],[0.,1.,0.],0.6),
  ("wrong tangent",[1.,4.,0.],[1.,0.,0.],0.6),
  ("wrong fraction",[1.,4.,0.],[0.,1.,0.],0.5),
 ]{
  let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("Tangent").unwrap();let before=state(&s);
  let v=packet(&s,&captured("tangent"),3,p,n,Some(f));
  if s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).is_ok(){accepted.push(label);}else{assert_eq!(state(&s),before);}
 }
 assert!(accepted.is_empty(),"tangent probe evidence detached from native source: {accepted:?}");
}
#[test]
fn initial_curve_parameters_reject_missing_invalid_fraction_and_removed_path_atomically(){
 for fraction in [None,Some(-0.1),Some(1.1)]{
  let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("Tangent").unwrap();let before=state(&s);
  let v=packet(&s,&captured("tangent"),3,[1.,4.,0.],[0.,1.,0.],fraction);
  assert!(s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).is_err());assert_eq!(state(&s),before);
 }
 let mut s=path_session("Circle");s.input("Undo").unwrap();let before=state(&s);
 let v=packet(&s,&captured("around"),2,[5.,0.,0.],[1.,0.,0.],Some(0.25));
 assert!(s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).is_err());assert_eq!(state(&s),before);
}
#[test]
fn initial_curve_parameters_admit_nineteen_native_families_at_their_actual_values_and_d1(){
 let cases=parse(include_str!("../tests/fixtures/owned_curve_evaluation/native_jets_oracle.json"));let mut count=0;
 let number=|v:&Json|crate::prepared_result::number(v).unwrap();
 for case in rows(&cases){for query in rows(get(case,"queries")){
  let p=crate::prepared_result::array::<3>(get(query,"value")).unwrap();let d=crate::prepared_result::array::<3>(get(query,"derivative")).unwrap();
  let length=d[0].hypot(d[1]).hypot(d[2]);let n=d.map(|v|v/length);let domain=rows(get(case,"domain"));let fraction=(number(get(query,"parameter"))-number(&domain[0]))/(number(&domain[1])-number(&domain[0]));
  let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("Tangent").unwrap();
  let mut v=packet(&s,&captured("tangent"),3,p,n,Some(fraction));let Json::Object(o)=&mut v else{panic!("packet")};let Json::Array(snapshots)=o.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};
  let Json::Object(snapshot)=&mut snapshots[0]else{panic!("snapshot")};snapshot.insert("selected_points_world".into(),Json::Array(vec![point(p)]));
  let Json::Object(topology)=snapshot.get_mut("topology_data").unwrap()else{panic!("topology")};let Json::Array(edges)=topology.get_mut("edges_3d").unwrap()else{panic!("edges")};let Json::Object(edge)=&mut edges[0]else{panic!("edge")};
  edge.insert("geometry_data".into(),get(case,"geometry_data").clone());edge.insert("domain".into(),get(case,"domain").clone());
  edge.insert("native_location".into(),parse("[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]"));
  // Synthetic input envelopes around genuine native D0/D1 oracle rows, not
  // live solver roots or new native BREP masters. Expected p/n never use the
  // owned evaluator under test.
  s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap_or_else(|e|panic!("family {:?} parameter {:?}: {e}",get(case,"name"),get(query,"parameter")));count+=1;
 }}assert_eq!(count,183);
}
