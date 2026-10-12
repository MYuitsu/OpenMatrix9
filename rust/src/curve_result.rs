// SPDX-License-Identifier: LGPL-2.1-or-later
//! Reuse existing Curve solvers to verify owned construction/result inputs.
//! Kernel masters and validity remain native observations, not conversion proofs.
use super::{CurveSession,Effect,input_log};
use crate::builder_history::{Json,parse_document};
use crate::prepared_result::{object,field,text,number,array,frame,map,string,encode_owned,MAX_RESULT_BYTES,MAX_RESULT_NODES};
use std::collections::BTreeMap;
fn rows(v:&Json)->Result<&[Json],u32>{if let Json::Array(a)=v{Ok(a)}else{Err(1)}}
fn identity(v:&Json,zero:bool)->Result<u64,u32>{let raw=text(v)?;let n=raw.parse::<u64>().map_err(|_|5u32)?;if (!zero&&n==0)||n.to_string()!=raw{return Err(5);}Ok(n)}
fn index(v:&Json,max:usize)->Result<usize,u32>{let n=number(v)?;if !n.is_finite()||n.fract()!=0.||n<0.||n>max as f64{return Err(2);}Ok(n as usize)}
fn point(v:&Json)->Result<[f64;3],u32>{let p=array(v)?;if p.iter().any(|n|!n.is_finite()){return Err(4);}Ok(p)}
fn points(v:&Json)->Result<Vec<[f64;3]>,u32>{let a=rows(v)?;if a.len()>MAX_RESULT_NODES/4{return Err(2);}a.iter().map(point).collect()}
fn native_packet(s:&CurveSession,event:&BTreeMap<String,Json>,event_index:usize,construction:&BTreeMap<String,Json>,units:&str,generation:u64)->Result<String,u32>{
 let mut refs=vec![];let mut snapshots=vec![];let all=rows(field(construction,"source_snapshots")?)?;
 for reference in rows(field(construction,"source_refs")?)?{
  let mut r=object(reference)?.clone();if index(field(&r,"accepted_event_index")?,input_log::MAX_EVENTS)?!=event_index{continue;}
  let slot=index(field(&r,"input_snapshot_index")?,all.len())?.checked_sub(1).ok_or(1u32)?;
  let value=all.get(slot).ok_or(1u32)?;let local=snapshots.iter().position(|v|v==value).unwrap_or(snapshots.len());
  if local==snapshots.len(){snapshots.push(value.clone());}
  r.remove("accepted_event_index");r.insert("input_snapshot_index".into(),Json::Number((local+1)as f64));refs.push(Json::Object(r));
 }
 if index(field(event,"native_source_count")?,1024)?!=refs.len(){return Err(1);}
 let kind=text(field(event,"kind")?)?;let action=match kind{"native_reference"=>"reference","native_fit_batch"=>"fit_batch","point"=>"point","native_solution"=>"tangent_solution",_=>return Err(1)};
 let value=|key|event.get(key).cloned().unwrap_or(Json::Other);
 let mut packet=map([("schema_version",Json::Number(1.)),("operation",string("AcceptCurveNativeInput")),("command",string(s.name())),
  ("generation",string(&generation.to_string())),("admission_revision",string(&s.native_input_revision().to_string())),("context_before",string(&s.circle_input_context_key().to_string())),
  ("unit_context_snapshot",string(units)),("action",string(action)),("text",value("text")),
  ("reference_kind",event.get("reference_kind").cloned().unwrap_or(Json::Number(0.))),
  ("point_world",event.get("point_world").cloned().unwrap_or(input_log::vector(&[0.;3]))),
  ("normal",event.get("normal").cloned().unwrap_or(input_log::vector(&[0.;3]))),
  ("points_world",event.get("points_world").cloned().unwrap_or(Json::Array(vec![]))),
  ("source_refs",Json::Array(refs)),("source_snapshots",Json::Array(snapshots))]);
 if let Some(fraction)=event.get("source_curve_fraction"){let Json::Object(o)=&mut packet else{return Err(1);};o.insert("source_curve_fraction".into(),fraction.clone());}
 if kind=="native_solution"{let Json::Object(o)=&mut packet else{return Err(1);};o.insert("solution_radius".into(),field(event,"radius")?.clone());o.insert("point_world".into(),field(event,"center")?.clone());o.insert("resolved_contact_fractions".into(),field(event,"resolved_contact_fractions")?.clone());o.insert("resolved_contact_indexing".into(),field(event,"resolved_contact_indexing")?.clone());}
 encode_owned(&packet)
}
fn replay(construction:&Json,units:&str)->Result<Vec<[f64;3]>,u32>{
 let c=object(construction)?;let generation=identity(field(c,"generation")?,false)?;
 if text(field(c,"operation")?)?!="ConstructCurve"||text(field(c,"source_snapshot_indexing")?)?!="source_snapshots_one_based"{return Err(1);}
 let command=text(field(c,"command")?)?;let events=rows(field(c,"accepted_input_events")?)?;
 if events.is_empty()||events.len()>input_log::MAX_EVENTS{return Err(2);}
 let scale=if units.is_empty(){1.}else{crate::units::Context::decode(units).map_err(|_|5u32)?.model_mm};
 let mut s=CurveSession::default();s.start(command).map_err(|_|1u32)?;let mut accepted=vec![];let mut previous_revision=0;
 for(i,value)in events.iter().enumerate(){
  if !s.active(){return Err(2);}
  let e=object(value)?;let before=identity(field(e,"admission_revision_before")?,true)?;let after=identity(field(e,"admission_revision_after")?,false)?;
  if before<previous_revision||before.checked_add(1)!=Some(after){return Err(5);}previous_revision=after;
  if number(field(e,"input_mm_per_unit")?)?!=scale{return Err(5);}
  let axes=rows(field(e,"frame_axes")?)?;if axes.len()!=3{return Err(6);}
  s.frame=Some(crate::coordinates::Frame::new(point(field(e,"frame_origin")?)?,[point(&axes[0])?,point(&axes[1])?,point(&axes[2])?]).map_err(|_|6u32)?);
  s.input_mm_per_unit=scale;s.native_input_revision=before;
  let context_before=identity(field(e,"circle_context_before")?,true)?;let context_after=identity(field(e,"circle_context_after")?,true)?;
  if command=="Circle"{
   if context_after<context_before{return Err(5);}s.circle_native_context_key=context_before;
   s.circle_native_context_counter=if context_after>context_before{context_after-1}else{context_before};
  }else if context_before!=0||context_after!=0{return Err(5);}
  let kind=text(field(e,"kind")?)?;
  let observed=match kind{
   "text"=>match field(e,"resolved_point_world")?{Json::Other=>vec![],v=>vec![point(v)?]},
   "point"|"mouse_point"=>vec![point(field(e,"point_world")?)?],
   "native_reference"=>if index(field(e,"reference_kind")?,5)?==1{vec![]}else{vec![point(field(e,"point_world")?)?]},
   "native_fit_batch"=>points(field(e,"points_world")?)?,"native_solution"=>vec![],_=>return Err(1),
  };
  if accepted.len().checked_add(observed.len()).is_none_or(|n|n>MAX_RESULT_NODES/4){return Err(2);}accepted.extend(observed);
  let effect=if e.contains_key("native_source_count"){
   let packet=native_packet(&s,e,i,c,units,generation)?;s.accept_native_json(&packet,generation)
  }else{match kind{
   "text"=>s.input(text(field(e,"text")?)?),"point"=>s.point(point(field(e,"point_world")?)?),
   "mouse_point"=>{let Json::Boolean(ortho)=field(e,"ortho")?else{return Err(1);};s.mouse_point(point(field(e,"point_world")?)?,*ortho)},
   "native_reference"=>{let p=point(field(e,"point_world")?)?;let n=point(field(e,"normal")?)?;let kind=index(field(e,"reference_kind")?,5)? as u32;
    if command=="Circle"{s.circle_native_reference(p,n,kind)}else{s.ellipse_native_reference(p,n,kind)}},
   "native_fit_batch"=>s.circle_fit_batch(&points(field(e,"points_world")?)?),
   "native_solution"=>s.circle_accept_spatial_solution(point(field(e,"center")?)?,point(field(e,"normal")?)?,number(field(e,"radius")?)?),_=>return Err(1),
  }}.map_err(|_|2u32)?;
  if effect==Effect::Cancelled{return Err(2);}
 }
 let canonical=s.construction_evidence(generation).map_err(|_|2u32)?;
 // Includes solver plans/options/basis, pre-normalization events and exact
 // source/event/index pairing, rather than trusting an asserted completed flag.
 if parse_document(&canonical,MAX_RESULT_BYTES,MAX_RESULT_NODES).map_err(|_|2u32)?!=*construction{return Err(2);}
 Ok(accepted)
}
pub(crate) fn validate(c:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>)->Result<Vec<Json>,u32>{
 if number(field(c,"schema_version")?)?!=1.||text(field(c,"operation")?)?!="ConstructCurve"
  ||text(field(c,"attribute_role")?)?!="source_evidence_in_input_snapshots_output_attributes_unavailable"||c.contains_key("native_transform_observations"){return Err(1);}
 frame(field(c,"output_parent_to_world")?)?;
 let construction=field(c,"construction")?;let recipe=object(construction)?;let units=text(field(output,"unit_context_snapshot")?)?;
 let accepted=replay(construction,units)?;
 let revision=if units.is_empty(){Json::Other}else{string(&crate::units::Context::decode(units).map_err(|_|5u32)?.revision.to_string())};
 let mut owned=vec![map([("schema_version",Json::Number(1.)),("ownership",string("private_plain_copy")),("unit_context_snapshot",string(units)),("unit_context_revision",revision.clone()),
  ("geometry_data",map([("representation",string("point_cloud")),("coordinates",string("document_world_mm")),("points_xyz",input_log::points(&accepted)),
   ("role",string("original_entrypoint_points_including_explicit_undo_history")),("accepted_input_events",field(recipe,"accepted_input_events")?.clone())])),
  ("topology_data",map([])),("source_attributes",map([("schema_version",Json::Number(1.)),("availability",string("not_applicable_constructed_output"))]))])];
 for snapshot in rows(field(recipe,"source_snapshots")?)?{
  let mut snapshot=object(snapshot)?.clone();snapshot.insert("unit_context_revision".into(),revision.clone());owned.push(Json::Object(snapshot));
 }
 Ok(owned)
}
fn native_matrix(value:&Json)->Result<[f64;16],u32>{
 let rows=rows(value)?;if rows.len()!=4{return Err(6);}
 let mut result=[0.;16];for(i,row)in rows.iter().enumerate(){result[i*4..i*4+4].copy_from_slice(&array::<4>(row)?);}Ok(result)
}
fn focus_position(c:&BTreeMap<String,Json>)->Result<Option<[f64;3]>,u32>{
 let Some(role)=c.get("output_role")else{return Ok(None);};
 let sign=match text(role)?{"ellipse_focus1"=>-1.,"ellipse_focus2"=>1.,_=>return Err(1)};
 let recipe=object(field(c,"construction")?)?;let options=object(field(recipe,"options")?)?;
 if text(field(recipe,"command")?)?!="Ellipse"||text(field(options,"mode")?)?!="FromFoci"||field(options,"mark_foci")?!=&Json::Boolean(true){return Err(1);}
 let ellipse=object(field(recipe,"ellipse")?)?;let center=point(field(ellipse,"center")?)?;let axis=point(field(ellipse,"major_direction")?)?;
 let major=number(field(ellipse,"major")?)?;let minor=number(field(ellipse,"minor")?)?;
 let distance=((major-minor)*(major+minor)).sqrt();
 let value: [f64;3]=std::array::from_fn(|i|center[i]+sign*distance*axis[i]);
 if value.iter().any(|x|!x.is_finite()){return Err(4);}Ok(Some(value))
}
pub(crate) fn validate_output(c:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>)->Result<(),u32>{
 if field(validation,"native_valid")?!=&Json::Boolean(true){return Err(1);}
 let topology=object(field(output,"topology_data")?)?;
 if !rows(field(topology,"faces")?)?.is_empty()||field(object(field(topology,"validation")?)?,"native_valid")?!=field(validation,"native_valid")?{return Err(3);}
 if let Some(expected)=focus_position(c)?{
  if number(field(output,"native_shape_type")?)?!=7.||!rows(field(topology,"edges_3d")?)?.is_empty(){return Err(1);}
  let vertices=rows(field(topology,"vertices")?)?;if vertices.len()!=1{return Err(3);}let vertex=object(&vertices[0])?;
  if text(field(vertex,"vertex_id")?)?!="1"||text(field(vertex,"geometry_coordinates")?)?!="native_local"||!crate::edge_geometry::tolerance_valid(number(field(vertex,"tolerance")?)?){return Err(3);}
  let location=native_matrix(field(vertex,"native_location")?)?;
  let geometry=object(field(output,"geometry_data")?)?;
  if native_matrix(field(geometry,"native_root_location")?)?!=location{return Err(6);}
  // A standalone vertex's location includes the root embedding already. Never
  // multiply the separately recorded root location into this point a second time.
  let i=crate::geometry_frame::IDENTITY;let transform=crate::geometry_frame::Frame::compose(array(field(c,"output_parent_to_world")?)?,i,i,location).map_err(|_|6u32)?;
  let actual=transform.transform(point(field(vertex,"point_xyz")?)?).map_err(|_|4u32)?;
  if actual.iter().zip(expected).any(|(a,b)|(a-b).abs()>64.*f64::EPSILON*a.abs().max(b.abs()).max(1.)){return Err(5);}
 }else if !matches!(number(field(output,"native_shape_type")?)?,5.|6.){return Err(1);}
 validate_checked_tangent_output(c,topology)?;
 Ok(())
}
fn validate_checked_tangent_output(c:&BTreeMap<String,Json>,topology:&BTreeMap<String,Json>)->Result<(),u32>{
 let recipe=object(field(c,"construction")?)?;
 let checked=rows(field(recipe,"accepted_input_events")?)?.iter().any(|event|matches!(event,Json::Object(o)if o.contains_key("resolved_contact_fractions")&&o.get("kind")==Some(&string("native_solution"))));
 if !checked{return Ok(());}
 if text(field(recipe,"command")?)?!="Circle"{return Err(1);}
 let plan=object(field(recipe,"circle")?)?;
 let expected=crate::circle::Plan::new(point(field(plan,"center")?)?,point(field(plan,"normal")?)?,number(field(plan,"radius")?)?).map_err(|_|2u32)?;
 let edges=rows(field(topology,"edges_3d")?)?;if edges.len()!=1{return Err(3);}
 let edge=object(&edges[0])?;
 if field(edge,"curve_present")?!=&Json::Boolean(true)||field(edge,"degenerate")?!=&Json::Boolean(false){return Err(3);}
 let start=text(field(edge,"start_vertex_id")?)?;
 if start.is_empty()||start!=text(field(edge,"end_vertex_id")?)?||!rows(field(topology,"vertices")?)?.iter().any(|v|object(v).ok().and_then(|v|v.get("vertex_id")).and_then(|v|text(v).ok())==Some(start)){return Err(3);}
 let near=|a:f64,b:f64|(a-b).abs()<=128.*f64::EPSILON*a.abs().max(b.abs()).max(1.);
 let data=object(field(edge,"geometry_data")?)?;
 let transform=crate::geometry_frame::Frame::compose(array(field(c,"output_parent_to_world")?)?,crate::geometry_frame::IDENTITY,crate::geometry_frame::IDENTITY,native_matrix(field(edge,"native_location")?)?).map_err(|_|6u32)?;
 let domain=array::<2>(field(edge,"domain")?)?;
 let spline=field(recipe,"spline")?;
 if spline!=&Json::Other{
  let spline=object(spline)?;if text(field(data,"kind")?)?!="nurbs_curve"{return Err(1);}
  let axis=object(field(data,"axis")?)?;
  for key in ["degree","periodic","knots","multiplicities"]{if field(axis,key)?!=field(spline,key)?{return Err(2);}}
  let expected_domain=array::<2>(field(spline,"domain")?)?;let full=array::<2>(field(axis,"full_domain")?)?;
  if domain.into_iter().zip(expected_domain).any(|(a,b)|!near(a,b))||full.into_iter().zip(expected_domain).any(|(a,b)|!near(a,b)){return Err(3);}
  let poles=rows(field(data,"poles_xyz")?)?;let expected_poles=rows(field(spline,"poles")?)?;if poles.len()!=expected_poles.len(){return Err(2);}
  for(actual,expected)in poles.iter().zip(expected_poles){let actual=transform.transform(point(actual)?).map_err(|_|4u32)?;if actual.into_iter().zip(point(expected)?).any(|(a,b)|!near(a,b)){return Err(5);}}
  let weights=field(data,"weights")?;if weights!=&Json::Other{let weights=rows(weights)?;if weights.len()!=poles.len()||weights.iter().any(|v|number(v)!=Ok(1.)){return Err(2);}}
  return Ok(());
 }
 if !near(domain[0],0.)||!near(domain[1],std::f64::consts::TAU){return Err(3);}
 if text(field(data,"kind")?)?!="analytic_curve"||text(field(data,"analytic_type")?)?!="circle"{return Err(1);}
 let actual=transform.transform(point(field(data,"origin_xyz")?)?).map_err(|_|4u32)?;
 if actual.into_iter().zip(expected.center).any(|(a,b)|!near(a,b))||!near(number(field(object(field(data,"parameters")?)?,"radius")?)?,expected.radius){return Err(5);}
 let axes=rows(field(data,"axes_xyz")?)?;if axes.len()!=3{return Err(6);}
 let origin=transform.transform([0.;3]).map_err(|_|6u32)?;let end=transform.transform(point(&axes[2])?).map_err(|_|6u32)?;
 if end.into_iter().zip(origin).zip(expected.normal).any(|((a,b),n)|!near(a-b,n)){return Err(5);}
 Ok(())
}
