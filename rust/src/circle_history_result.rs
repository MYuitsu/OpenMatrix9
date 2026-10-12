// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned evidence for independent stored Circle replay; no live command state.
use crate::builder_history::{Json,parse_document};
use crate::prepared_result::{object,field,text,number,array,frame,stamp,string,map,validate_native_snapshot};
use crate::geometry_contract::RepresentationKind;
use super::{Recipe,Replay,replay};
use std::collections::BTreeMap;
#[path="circle_history_sources.rs"]
mod sources;
fn rows(v:&Json)->Result<&[Json],u32>{if let Json::Array(a)=v{Ok(a)}else{Err(1)}}
fn xyz(v:&Json)->Result<[f64;3],u32>{let p=array(v)?;crate::circle::check_point(p).map_err(|_|4u32)?;Ok(p)}
fn integer(v:&Json,max:usize)->Result<usize,u32>{let n=number(v)?;if !n.is_finite()||n.fract()!=0.||n<0.||n>max as f64{return Err(2);}Ok(n as usize)}
fn near(a:f64,b:f64)->bool{(a-b).abs()<=128.*f64::EPSILON*a.abs().max(b.abs()).max(1.)+1e-7}
fn source_point_matches(a:[f64;3],b:[f64;3])->bool{
 a.into_iter().zip(b).all(|(a,b)|(a-b).abs()<=1e-6+128.*f64::EPSILON*a.abs().max(b.abs()).max(1.))
}
fn selected_curve(reference:&BTreeMap<String,Json>,snapshot:&BTreeMap<String,Json>)->Result<crate::owned_curve_input::SelectedEdge,u32>{
 crate::owned_curve_input::selected_reference(reference,snapshot)
}
fn solved(c:&BTreeMap<String,Json>)->Result<(Recipe,Vec<[f64;3]>,Replay),u32>{
 let recipe=Recipe::decode(&array::<24>(field(c,"recipe")?)?).map_err(|_|2u32)?;
 let values=rows(field(c,"points_world")?)?;if values.is_empty()||values.len()>1024{return Err(2);}
 let points=values.iter().map(xyz).collect::<Result<Vec<_>,_>>()?;
 let native=match field(c,"native_plan")?{Json::Other=>None,v=>Some(array::<7>(v)?)};
 if native.is_some()!=matches!(recipe.options.mode,crate::circle::Mode::AroundCurve|crate::circle::Mode::Tangent){return Err(2);}
 let result=replay(&recipe,&points,native).map_err(|_|2u32)?;
 let plan=object(field(c,"resolved_plan")?)?;
 if xyz(field(plan,"center")?)?!=result.plan.center||array::<3>(field(plan,"normal")?)?!=result.plan.normal
  ||number(field(plan,"radius")?)?!=result.plan.radius||number(field(plan,"fit_deviation")?)?!=result.fit_deviation
  ||number(field(plan,"approx_deviation")?)?!=result.approx_deviation{return Err(5);}
 Ok((recipe,points,result))
}
pub(crate) fn validate(c:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>)->Result<Vec<Json>,u32>{
 if number(field(c,"schema_version")?)?!=1.||text(field(c,"operation")?)?!="ReplayCircleHistory"
  ||text(field(c,"feature_id")?)?!="OM9-CURVE-005"||text(field(c,"attribute_role")?)?!="source_evidence_in_input_snapshots_output_attributes_unavailable"
  ||text(field(c,"source_snapshot_indexing")?)?!="input_snapshots_zero_based_including_numeric_input"||c.contains_key("native_transform_observations"){return Err(1);}
 frame(field(c,"output_parent_to_world")?)?;let(recipe,points,result)=solved(c)?;
 let storage=text(field(c,"storage")?)?;
 if !matches!(storage,"recipe_v2"|"legacy_around_curve")||storage=="legacy_around_curve"&&recipe.options.mode!=crate::circle::Mode::AroundCurve{return Err(1);}
 let original=text(field(c,"original_construction")?)?;
 if !original.is_empty(){
  let parsed=parse_document(original,crate::prepared_result::MAX_RESULT_BYTES,crate::prepared_result::MAX_RESULT_NODES).map_err(|_|2u32)?;
  let saved=object(&parsed)?;if text(field(saved,"command")?)?!="Circle"||text(field(saved,"operation")?)?!="ConstructCurve"{return Err(1);}
 }
 let units=text(field(output,"unit_context_snapshot")?)?;
 let revision=if units.is_empty(){Json::Other}else{string(&crate::units::Context::decode(units).map_err(|_|5u32)?.revision.to_string())};
 let refs=rows(field(c,"source_refs")?)?;let snapshots=rows(field(c,"source_snapshots")?)?;
 if refs.len()>1024||snapshots.len()>refs.len()||refs.is_empty()!=snapshots.is_empty(){return Err(2);}
 if matches!(recipe.options.mode,crate::circle::Mode::AroundCurve|crate::circle::Mode::Tangent)&&refs.is_empty(){return Err(1);}
 let around=recipe.options.mode==crate::circle::Mode::AroundCurve;
 let point_roles=Json::Array((0..points.len()).map(|index|string(if around{
  if index==0{"recorded_path_probe"}else{"radius_pick_world"}
 }else{"current_replay_point"})).collect());
 let path_anchor=if around{Json::Array(result.plan.center.iter().map(|v|Json::Number(*v)).collect())}else{Json::Other};
 let mut owned=vec![map([("schema_version",Json::Number(1.)),("ownership",string("private_plain_copy")),("unit_context_snapshot",string(units)),("unit_context_revision",revision.clone()),
  ("geometry_data",map([("representation",string("point_cloud")),("coordinates",string("document_world_mm")),("points_xyz",Json::Array(points.iter().map(|p|Json::Array(p.iter().map(|v|Json::Number(*v)).collect())).collect())),
   ("role",string("circle_history_replay_inputs")),("point_roles",point_roles),("resolved_path_anchor_xyz",path_anchor),
   ("history_storage",map([("kind",string(storage)),("recipe",field(c,"recipe")?.clone()),("original_command_events",string(if original.is_empty(){"unavailable_legacy_record"}else{"retained_in_original_construction"}))]))])),
  ("topology_data",map([])),("source_attributes",map([("schema_version",Json::Number(1.)),("availability",string("not_applicable_constructed_output"))]))])];
 let mut used=vec![false;snapshots.len()];let mut document=None;
 // A source role addresses the recorded Sources list, not its shared snapshot
 // slot. Sharing geometry never permits duplicate or missing semantic roles.
 let mut role_indices=vec![false;refs.len()];let mut paths=0usize;
 for r in refs{
  let r=object(r)?;stamp(r)?;frame(field(r,"world")?)?;
  let uid=text(field(r,"document_uid")?)?;let name=text(field(r,"object_name")?)?;
  if uid.is_empty()||name.is_empty()||name.contains('\0')||text(field(r,"subelement")?)?.contains('\0'){return Err(1);}
  let id=(text(field(r,"document")?)?,uid);if document.is_some_and(|old|old!=id){return Err(1);}document=Some(id);
  let ancestors=rows(field(r,"ancestors")?)?;if ancestors.len()>64{return Err(2);}for a in ancestors{let a=object(a)?;stamp(a)?;if field(a,"document")?!=field(r,"document")?{return Err(1);}}
  let slot=integer(field(r,"input_snapshot_index")?,snapshots.len())?.checked_sub(1).ok_or(1u32)?;
  let mut snapshot=object(snapshots.get(slot).ok_or(1u32)?)?.clone();used[slot]=true;
  if number(field(&snapshot,"schema_version")?)?!=1.||text(field(r,"unit_context_snapshot")?)?!=units||text(field(&snapshot,"unit_context_snapshot")?)?!=units{return Err(5);}
  let attrs=field(&snapshot,"source_attributes")?;crate::source_attributes::SourceAttributes::decode(attrs).map_err(|_|2u32)?;let attrs=object(attrs)?;
  if text(field(attrs,"document_uid")?)?!=uid||text(field(attrs,"object_name")?)?!=name{return Err(1);}
  let parent_representation=if snapshot.contains_key("native_shape_type"){
   let topology=object(field(&snapshot,"topology_data")?)?;validate_native_snapshot(&snapshot,object(field(topology,"validation")?)?,true)?;
   if text(field(r,"snapshot_frame")?)?!="document_world"{return Err(6);}
   match number(field(&snapshot,"native_shape_type")?)?{0.|1.=>RepresentationKind::Brep,2.=>RepresentationKind::Solid,3.=>RepresentationKind::Shell,4.=>RepresentationKind::TrimmedFace,5.=>RepresentationKind::Polycurve,6.=>RepresentationKind::Curve,7.=>RepresentationKind::Point,_=>return Err(1)}
  }else{
   crate::curve::native_input::validate_discrete(&snapshot,r)?;
   if text(field(r,"snapshot_frame")?)?!="native_local_with_locations"{return Err(6);}
   match text(field(object(field(&snapshot,"geometry_data")?)?,"kind")?)?{"mesh"=>RepresentationKind::Mesh,"point_cloud"=>RepresentationKind::PointCloud,_=>return Err(1)}
  };
  let selected=match number(field(r,"selected_representation")?)?{
   1.=>RepresentationKind::Point,2.=>RepresentationKind::Curve,3.=>RepresentationKind::Polycurve,4.=>RepresentationKind::TrimmedFace,
   5.=>RepresentationKind::Shell,6.=>RepresentationKind::Solid,7.=>RepresentationKind::Brep,8.=>RepresentationKind::Mesh,9.=>RepresentationKind::PointCloud,_=>return Err(1)
  };
  let sub=text(field(r,"subelement")?)?;
  if sub.is_empty(){if selected!=parent_representation{return Err(1);}}
  else if let Some(index)=sub.strip_prefix("Vertex"){
   if selected!=RepresentationKind::Point||index.parse::<usize>().map_or(true,|i|i==0){return Err(1);}
   let topology=object(field(&snapshot,"topology_data")?)?;
   let vertex=rows(field(topology,"vertices")?)?.iter().find_map(|v|{let v=object(v).ok()?;(text(field(v,"native_topology_index").ok()?).ok()?==index).then_some(v)}).ok_or(3u32)?;
   let location=rows(field(vertex,"native_location")?)?;if location.len()!=4{return Err(6);}let mut m=[0.;16];
   for(i,row)in location.iter().enumerate(){m[i*4..i*4+4].copy_from_slice(&array::<4>(row)?);}
   let i=crate::geometry_frame::IDENTITY;let f=crate::geometry_frame::Frame::compose(i,i,i,m).map_err(|_|6u32)?;
   let values=rows(field(&snapshot,"selected_points_world")?)?;if values.len()!=1{return Err(1);}
   if f.transform(crate::owned_point_selection::local_point(field(vertex,"point_xyz")?)?).map_err(|_|6u32)?.iter().zip(xyz(&values[0])?).any(|(a,b)|!near(*a,b)){return Err(5);}
  }else if let Some(index)=sub.strip_prefix("Edge"){
   if selected!=RepresentationKind::Curve||index.parse::<usize>().map_or(true,|i|i==0){return Err(1);}
   let topology=object(field(&snapshot,"topology_data")?)?;
   if !rows(field(topology,"edges_3d")?)?.iter().any(|v|object(v).ok().and_then(|v|v.get("native_topology_index")).and_then(|v|text(v).ok())==Some(index)){return Err(3);}
  }else if let Some(index)=sub.strip_prefix("Face"){
   if selected!=RepresentationKind::TrimmedFace||index.parse::<usize>().map_or(true,|i|i==0){return Err(1);}
   let topology=object(field(&snapshot,"topology_data")?)?;
   if !rows(field(topology,"faces")?)?.iter().any(|v|object(v).ok().and_then(|v|v.get("native_topology_index")).and_then(|v|text(v).ok())==Some(index)){return Err(3);}
  }else{return Err(1);}
  let role=text(field(r,"role")?)?;
  if !matches!(role,"history_source"|"history_path"){return Err(1);}
  let index=integer(field(r,"role_index")?,1024)?;
  let bound=role_indices.get_mut(index).ok_or(1u32)?;if *bound{return Err(1);}*bound=true;
  if role=="history_path"&&(!matches!(recipe.options.mode,crate::circle::Mode::AroundCurve)
   ||!matches!(selected,RepresentationKind::Curve|RepresentationKind::Polycurve)){return Err(1);}
  if role=="history_path"{
   paths+=1;if index+1!=refs.len(){return Err(1);}
   let(point,tangent)=selected_curve(r,&snapshot)?.at_fraction(number(field(c,"path_fraction")?)?)?;
   if !source_point_matches(point,result.plan.center)||tangent.into_iter().zip(result.plan.normal).any(|(a,b)|(a-b).abs()>1e-8){return Err(5);}
  }
  recipe.construction_capability(selected,points.len()).map_err(|_|1u32)?;
  snapshot.insert("unit_context_revision".into(),revision.clone());
 }
 if paths!=usize::from(recipe.options.mode==crate::circle::Mode::AroundCurve){return Err(1);}
 if used.iter().any(|v|!*v){return Err(1);}
 sources::validate(c,&points)?;
 if recipe.options.mode==crate::circle::Mode::Tangent{
  validate_tangent_constraints(c,&points,&result)?;
 }
 for snapshot in snapshots{let mut snapshot=object(snapshot)?.clone();snapshot.insert("unit_context_revision".into(),revision.clone());owned.push(Json::Object(snapshot));}
 Ok(owned)
}
fn validate_tangent_constraints(c:&BTreeMap<String,Json>,points:&[[f64;3]],result:&Replay)->Result<(),u32>{
 let refs=rows(field(c,"source_refs")?)?;
 let roles=rows(field(c,"recorded_source_roles")?)?;
 if roles.len()!=refs.len().checked_mul(3).ok_or(2u32)?{return Err(1);}
 let parameters=rows(field(c,"recorded_source_parameters")?)?;
 if parameters.len()!=refs.len(){return Err(1);}
 let snapshots=rows(field(c,"source_snapshots")?)?;
 let contacts=rows(field(c,"resolved_contact_fractions")?)?;
 if contacts.len()!=points.len()||text(field(c,"resolved_contact_indexing")?)?!="current_constraint_points_zero_based"{return Err(1);}
 let mut contact_curves=vec![false;points.len()];
 for reference in refs{
  let reference=object(reference)?;
  let index=integer(field(reference,"role_index")?,refs.len())?;
  let role=roles.get(index.checked_mul(3).ok_or(2u32)?).ok_or(1u32)?;
  if number(role)?==2.&&!matches!(number(field(reference,"selected_representation")?)?,2.|3.){return Err(1);}
  let parameter=number(parameters.get(index).ok_or(1u32)?)?;if !parameter.is_finite(){return Err(2);}
  let offset=integer(roles.get(index*3+1).ok_or(1u32)?,points.len())?;
  let slot=integer(field(reference,"input_snapshot_index")?,snapshots.len())?.checked_sub(1).ok_or(1u32)?;
  let snapshot=object(snapshots.get(slot).ok_or(1u32)?)?;
  let point=*points.get(offset).ok_or(1u32)?;
  match number(role)?{
   2.=>{
    let curve=selected_curve(reference,snapshot)?;
    let(current,_)=curve.at_fraction(parameter)?;
    if !source_point_matches(current,point){return Err(5);}
    contact_curves[offset]=true;
    let(contact,tangent)=curve.at_fraction(number(&contacts[offset])?)?;
    if !super::tangent_point_matches(&result.plan,contact){return Err(5);}
    let dot=|a:[f64;3],b:[f64;3]|a.into_iter().zip(b).map(|(x,y)|x*y).sum::<f64>();
    let delta=std::array::from_fn(|j|contact[j]-result.plan.center[j]);
    if dot(tangent,result.plan.normal).abs()>1e-8||dot(delta,tangent).abs()/result.plan.radius>1e-8{return Err(5);}
    let recipe=Recipe::decode(&array::<24>(field(c,"recipe")?)?).map_err(|_|2u32)?;
    if recipe.options.from_first&&offset==0{
     let d:[f64;3]=std::array::from_fn(|j|contact[j]-points[0][j]);
     if d[0].hypot(d[1]).hypot(d[2])>1e-6{return Err(5);}
    }
   },
   0.=>{
    let selected=rows(field(snapshot,"selected_points_world")?)?;
    if selected.len()!=1||!source_point_matches(xyz(&selected[0])?,point){return Err(5);}
   },
   _=>{},
  }
 }
 for(contact,is_curve)in contacts.iter().zip(contact_curves){if !is_curve&&contact!=&Json::Other{return Err(1);}}
 let mut curve_constraints=vec![false;points.len()];let mut bound_points=vec![false;points.len()];
 for role in roles.chunks_exact(3){
  let kind=number(&role[0])?;
  let offset=integer(&role[1],points.len())?;
  let count=integer(&role[2],points.len())?;
  if offset>=points.len(){return Err(1);}
  match kind{
   0.|2.=>{
    if count!=1||bound_points[offset]{return Err(1);}bound_points[offset]=true;
    curve_constraints[offset]=kind==2.;
   },
   -1.|4.=>{if count!=0{return Err(1);}},
   _=>return Err(1),
  }
 }
 // OnCurve points are probes for native solver selection, and need not be the
 // solved contact. Every remaining constraint is a fixed circumference point.
 // This test is independent of curve family and preserves those probe semantics.
 for (point,is_curve)in points.iter().zip(curve_constraints){
  if is_curve{continue;}
  if !super::tangent_point_matches(&result.plan,*point){return Err(5);}
 }
 Ok(())
}
pub(crate) fn validate_output(c:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>)->Result<(),u32>{
 if field(validation,"native_valid")?!=&Json::Boolean(true)||!matches!(number(field(output,"native_shape_type")?)?,5.|6.){return Err(1);}
 let topology=object(field(output,"topology_data")?)?;if !rows(field(topology,"faces")?)?.is_empty(){return Err(3);}
 let edges=rows(field(topology,"edges_3d")?)?;if edges.len()!=1{return Err(3);}
 let edge=object(&edges[0])?;let data=object(field(edge,"geometry_data")?)?;
 if field(edge,"curve_present")?!=&Json::Boolean(true)||field(edge,"degenerate")?!=&Json::Boolean(false){return Err(3);}
 let start=text(field(edge,"start_vertex_id")?)?;
 if start.is_empty()||start!=text(field(edge,"end_vertex_id")?)?
  ||!rows(field(topology,"vertices")?)?.iter().any(|v|object(v).ok().and_then(|v|v.get("vertex_id")).and_then(|v|text(v).ok())==Some(start)){return Err(3);}
 let domain=array::<2>(field(edge,"domain")?)?;
 let location=rows(field(edge,"native_location")?)?;if location.len()!=4{return Err(6);}let mut matrix=[0.;16];
 for(i,row)in location.iter().enumerate(){matrix[i*4..i*4+4].copy_from_slice(&array::<4>(row)?);}
 let i=crate::geometry_frame::IDENTITY;let transform=crate::geometry_frame::Frame::compose(array(field(c,"output_parent_to_world")?)?,i,i,matrix).map_err(|_|6u32)?;
 let(_,_,result)=solved(c)?;
 let point=|p:[f64;3]|transform.transform(p).map_err(|_|6u32);
 if let Some(spline)=result.spline{
  if text(field(data,"kind")?)?!="nurbs_curve"{return Err(1);}let axis=object(field(data,"axis")?)?;
  if number(field(axis,"degree")?)?!=spline.degree as f64||field(axis,"periodic")?!=&Json::Boolean(spline.periodic){return Err(2);}
  let poles=rows(field(data,"poles_xyz")?)?;if poles.len()!=spline.poles.len(){return Err(2);}
  for(p,q)in poles.iter().zip(&spline.poles){if point(xyz(p)?)?.iter().zip(q).any(|(a,b)|!near(*a,*b)){return Err(5);}}
  let weights=field(data,"weights")?;if !matches!(weights,Json::Other){if rows(weights)?.len()!=poles.len()||rows(weights)?.iter().any(|v|number(v)!=Ok(1.)){return Err(2);}}
  let(knots,mults)=spline.host_knots();let actual=rows(field(axis,"knots")?)?;let counts=rows(field(axis,"multiplicities")?)?;
  let full=array::<2>(field(axis,"full_domain")?)?;let expected=[*knots.first().ok_or(2u32)?,*knots.last().ok_or(2u32)?];
  if domain.iter().zip(expected).any(|(a,b)|!near(*a,b))||full.iter().zip(expected).any(|(a,b)|!near(*a,b)){return Err(3);}
  if actual.len()!=knots.len()||counts.len()!=mults.len(){return Err(2);}
  for(a,b)in actual.iter().zip(knots){if !near(number(a)?,b){return Err(5);}}
  for(a,b)in counts.iter().zip(mults){if number(a)?!=b as f64{return Err(2);}}
 }else{
  if !near(domain[0],0.)||!near(domain[1],std::f64::consts::TAU){return Err(3);}
  if text(field(data,"kind")?)?!="analytic_curve"||text(field(data,"analytic_type")?)?!="circle"{return Err(1);}
  let center=point(xyz(field(data,"origin_xyz")?)?)?;
  if center.iter().zip(result.plan.center).any(|(a,b)|!near(*a,b))||!near(number(field(object(field(data,"parameters")?)?,"radius")?)?,result.plan.radius){return Err(5);}
  let axes=rows(field(data,"axes_xyz")?)?;if axes.len()!=3{return Err(6);}let normal=xyz(&axes[2])?;let origin=point([0.;3])?;let end=point(normal)?;
  let normal:[f64;3]=std::array::from_fn(|j|end[j]-origin[j]);
  if normal.iter().zip(result.plan.normal).any(|(a,b)|!near(*a,b)){return Err(5);}
 }
 Ok(())
}
