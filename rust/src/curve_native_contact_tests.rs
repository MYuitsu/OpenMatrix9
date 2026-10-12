// SPDX-License-Identifier: LGPL-2.1-or-later
//! Initial native solve source/contact binding, independent literal line oracle.
use super::*;
fn parse(s:&str)->Json{parse_document(s,16*1024*1024,1_000_000).unwrap()}
fn ready()->CurveSession{
 let v=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/tangent.json"));let c=get(&v,"operation_context");
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("Tangent").unwrap();
 for i in 0..2{
  let p=crate::prepared_result::array::<3>(&rows(get(c,"points_world"))[i]).unwrap();let n=if i==0{[0.,1.,0.]}else{[1.,0.,0.]};
  let mut r=rows(get(c,"source_refs"))[i].clone();let Json::Object(o)=&mut r else{panic!()};o.insert("role".into(),string("tangent"));o.insert("role_index".into(),Json::Number(i as f64));o.insert("input_snapshot_index".into(),Json::Number(1.));
  let mut snapshot=rows(get(c,"source_snapshots"))[i].clone();let Json::Object(o)=&mut snapshot else{panic!()};o.insert("selected_points_world".into(),Json::Array(vec![point(p)]));
  let packet=map([("schema_version",Json::Number(1.)),("operation",string("AcceptCurveNativeInput")),("command",string("Circle")),("generation",string(&u64::MAX.to_string())),
   ("context_before",string(&s.circle_input_context_key().to_string())),("admission_revision",string(&s.native_input_revision().to_string())),("unit_context_snapshot",string(UNITS)),
   ("action",string("reference")),("reference_kind",Json::Number(3.)),("point_world",point(p)),("normal",point(n)),("points_world",Json::Array(vec![])),
   ("text",string("Curve=native source probe")),("source_curve_fraction",Json::Number(0.6)),("source_refs",Json::Array(vec![r])),("source_snapshots",Json::Array(vec![snapshot]))]);
  s.accept_native_json(&encode_owned(&packet).unwrap(),u64::MAX).unwrap();if i==0{s.input("Radius=4").unwrap();}
 }
 s
}
fn solution(s:&CurveSession,center:[f64;3],contacts:&str)->Json{
 map([("schema_version",Json::Number(1.)),("operation",string("AcceptCurveNativeInput")),("command",string("Circle")),("generation",string(&u64::MAX.to_string())),
 ("context_before",string(&s.circle_input_context_key().to_string())),("admission_revision",string(&s.native_input_revision().to_string())),("unit_context_snapshot",string(UNITS)),
 ("action",string("tangent_solution")),("reference_kind",Json::Number(0.)),("point_world",point(center)),("normal",point([0.,0.,1.])),("points_world",Json::Array(vec![])),("text",Json::Other),
 ("source_refs",Json::Array(vec![])),("source_snapshots",Json::Array(vec![])),("solution_radius",Json::Number(4.)),
 ("resolved_contact_fractions",parse(contacts)),("resolved_contact_indexing",string("current_constraint_points_zero_based"))])
}
fn state(s:&CurveSession)->(String,Json,Vec<[f64;3]>,u64,bool){(s.prompt(),s.accepted_input_log.json(),s.points().to_vec(),s.native_input_revision(),s.active())}
#[test]
fn initial_tangent_contacts_reject_unchecked_plan_with_owned_probes_atomically(){
 let mut s=ready();let before=state(&s);
 // Both probes remain exactly on their owned lines x=1/y=0, but this valid
 // numeric circle is one millimetre away from being tangent to y=0.
 assert!(s.circle_accept_spatial_solution([5.,5.,0.],[0.,0.,1.],4.).is_err());assert_eq!(state(&s),before);
}
#[test]
fn initial_tangent_contacts_accept_independent_line_contacts_separate_from_probes(){
 let mut s=ready();let v=solution(&s,[5.,4.,0.],"[0.6,0.625]");
 assert_eq!(s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap(),super::super::Effect::Commit);
 let c=parse(&s.construction_evidence(u64::MAX).unwrap());let e=rows(get(&c,"accepted_input_events")).last().unwrap();
 assert_eq!(get(e,"resolved_contact_fractions"),&parse("[0.6,0.625]"));
 assert_eq!(get(&c,"normalized_points"),&parse("[[1,4,0],[4,0,0]]"));
}
#[test]
fn initial_tangent_contacts_reject_off_circle_contact_wrong_count_and_plan(){
 for(center,contacts)in [([5.,4.,0.],"[0.6,0.6]"),([5.,4.,0.],"[0.6]"),([5.,4.,0.],"[null,0.625]"),([5.,5.,0.],"[0.6,0.625]")]{
  let mut s=ready();let before=state(&s);let v=solution(&s,center,contacts);assert!(s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).is_err());assert_eq!(state(&s),before);
 }
}
#[test]
fn initial_tangent_contacts_keep_fixed_point_null_and_from_first_semantics(){
 let mut s=ready();s.input("Undo").unwrap();s.input("FromFirstPoint=Yes").unwrap();
 s.circle_native_reference([5.,8.,0.],[0.;3],5).unwrap();
 let before=state(&s);let bad=solution(&s,[5.,4.,0.],"[0.6,0.5]");assert!(s.accept_native_json(&encode_owned(&bad).unwrap(),u64::MAX).is_err());assert_eq!(state(&s),before);
 let good=solution(&s,[5.,4.,0.],"[0.6,null]");assert_eq!(s.accept_native_json(&encode_owned(&good).unwrap(),u64::MAX).unwrap(),super::super::Effect::Commit);
}
#[test]
fn initial_tangent_contacts_replay_publication_through_the_same_owned_source_gate(){
 let mut s=ready();let v=solution(&s,[5.,4.,0.],"[0.6,0.625]");s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap();
 let construction=parse(&s.construction_evidence(u64::MAX).unwrap());
 let history=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/tangent.json"));
 let context=map([("schema_version",Json::Number(1.)),("operation",string("ConstructCurve")),("construction",construction.clone()),
  ("output_parent_to_world",super::super::input_log::vector(&crate::geometry_frame::IDENTITY)),("attribute_role",string("source_evidence_in_input_snapshots_output_attributes_unavailable"))]);
 let output=get(&history,"snapshot");assert!(super::super::result::validate(crate::prepared_result::object(&context).unwrap(),crate::prepared_result::object(output).unwrap()).is_ok());
 // Mutate the final native solution/contact together; these original probes
 // and complete source snapshots remain unchanged. Independent replay must
 // reject the stale valid numeric circle at the publication boundary.
 let mut bad=context;let Json::Object(o)=&mut bad else{panic!()};let Json::Object(c)=o.get_mut("construction").unwrap()else{panic!()};
 let Json::Array(events)=c.get_mut("accepted_input_events").unwrap()else{panic!()};let Json::Object(e)=events.last_mut().unwrap()else{panic!()};e.insert("center".into(),point([5.,5.,0.]));
 assert!(super::super::result::validate(crate::prepared_result::object(&bad).unwrap(),crate::prepared_result::object(output).unwrap()).is_err());
}
#[test]
fn initial_tangent_contacts_bind_published_native_circle_to_checked_solution(){
 let mut s=ready();let v=solution(&s,[5.,4.,0.],"[0.6,0.625]");s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap();
 let context=map([("construction",parse(&s.construction_evidence(u64::MAX).unwrap())),("output_parent_to_world",super::super::input_log::vector(&crate::geometry_frame::IDENTITY))]);
 let history=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/tangent.json"));let output=get(&history,"snapshot");let validation=get(get(output,"topology_data"),"validation");
 let verify=|value:&Json|super::super::result::validate_output(crate::prepared_result::object(&context).unwrap(),crate::prepared_result::object(value).unwrap(),crate::prepared_result::object(validation).unwrap());
 assert!(verify(output).is_ok());
 for key in ["center","radius","normal","domain","open","extra_edge"]{
  let mut changed=output.clone();let Json::Object(o)=&mut changed else{panic!()};let Json::Object(t)=o.get_mut("topology_data").unwrap()else{panic!()};let Json::Array(edges)=t.get_mut("edges_3d").unwrap()else{panic!()};
  if key=="extra_edge"{edges.push(edges[0].clone());}else{
   let Json::Object(edge)=&mut edges[0]else{panic!()};
   match key{
    "domain"=>{edge.insert("domain".into(),parse("[0,3.141592653589793]"));},
    "open"=>{edge.insert("end_vertex_id".into(),string("different_vertex"));},
    _=>{let Json::Object(data)=edge.get_mut("geometry_data").unwrap()else{panic!()};match key{
     "center"=>{data.insert("origin_xyz".into(),point([5.,5.,0.]));},
     "normal"=>{data.insert("axes_xyz".into(),parse("[[1,0,0],[0,0,1],[0,1,0]]"));},
     _=>{let Json::Object(parameters)=data.get_mut("parameters").unwrap()else{panic!()};parameters.insert("radius".into(),Json::Number(5.));}
    }}
   }
  }
  assert!(verify(&changed).is_err(),"unchecked native output {key}");
 }
}
#[test]
fn initial_tangent_contacts_preserve_deformable_output_and_verify_its_basis(){
 let mut s=ready();s.input("Deformable=Yes").unwrap();let v=solution(&s,[5.,4.,0.],"[0.6,0.625]");s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap();
 let construction=parse(&s.construction_evidence(u64::MAX).unwrap());let spline=get(&construction,"spline");
 let context=map([("construction",construction.clone()),("output_parent_to_world",super::super::input_log::vector(&crate::geometry_frame::IDENTITY))]);
 let history=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/tangent.json"));let mut output=get(&history,"snapshot").clone();
 // Synthetic typed output envelope exercises the owned basis contract. The
 // immutable native fixture master remains an independent circle, so this is
 // a unit test, not a native spline capture or master-decoding certificate.
 let Json::Object(o)=&mut output else{panic!()};let Json::Object(t)=o.get_mut("topology_data").unwrap()else{panic!()};let Json::Array(edges)=t.get_mut("edges_3d").unwrap()else{panic!()};let Json::Object(edge)=&mut edges[0]else{panic!()};
 edge.insert("domain".into(),get(spline,"domain").clone());
 edge.insert("geometry_data".into(),map([("kind",string("nurbs_curve")),("poles_xyz",get(spline,"poles").clone()),("weights",Json::Other),
  ("axis",map([("degree",get(spline,"degree").clone()),("periodic",get(spline,"periodic").clone()),("full_domain",get(spline,"domain").clone()),("knots",get(spline,"knots").clone()),("multiplicities",get(spline,"multiplicities").clone())]))]));
 let validation=get(get(&output,"topology_data"),"validation").clone();
 let verify=|value:&Json|super::super::result::validate_output(crate::prepared_result::object(&context).unwrap(),crate::prepared_result::object(value).unwrap(),crate::prepared_result::object(&validation).unwrap());
 assert!(verify(&output).is_ok(),"deformable tangent must remain supported");
 let Json::Object(o)=&mut output else{panic!()};let Json::Object(t)=o.get_mut("topology_data").unwrap()else{panic!()};let Json::Array(edges)=t.get_mut("edges_3d").unwrap()else{panic!()};let Json::Object(edge)=&mut edges[0]else{panic!()};let Json::Object(data)=edge.get_mut("geometry_data").unwrap()else{panic!()};
 let Json::Array(poles)=data.get_mut("poles_xyz").unwrap()else{panic!()};let Json::Array(p)=&mut poles[0]else{panic!()};p[0]=Json::Number(999.);
 assert!(verify(&output).is_err());
}
