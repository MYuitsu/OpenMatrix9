// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned admission of the existing native Solid construction result.
//! Native master/validity are host observations, not global conversion proofs.
use crate::builder_history::Json;
use crate::prepared_result::{object,field,text,number,array,frame,stamp,map,string,validate_native_snapshot};
use crate::geometry_contract::{operation_capability,OperationId,OperationOptions,RepresentationKind};
use crate::{solid::{Kind,Mode},native_solid_capability::{Request,Stage}};
use std::collections::BTreeMap;
/// Necessary aggregate source-envelope gate before the session accepts a batch.
/// The completed request/output retain their own full budget and validity gates.
/// Native owners are never retained; the caller owns immutable initialized bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_solid_source_budget_json(input:*const u8,length:usize)->u32{
 use crate::prepared_result::{MAX_RESULT_BYTES,MAX_RESULT_NODES,MAX_RESULT_DEPTH};
 if length==0||length>MAX_RESULT_BYTES||length>isize::MAX as usize||input.is_null()
  ||(input as usize).checked_add(length).is_none(){return 2;}
 std::panic::catch_unwind(||{
  let raw=std::str::from_utf8(unsafe{std::slice::from_raw_parts(input,length)}).map_err(|_|1u32)?;
  // The actual request wraps this source envelope in operation_context.
  let value=crate::builder_history::parse_document_at_depth(raw,MAX_RESULT_BYTES,MAX_RESULT_NODES,MAX_RESULT_DEPTH-1).map_err(|_|2u32)?;
  let o=object(&value)?;let refs=rows(field(o,"source_refs")?)?;let snapshots=rows(field(o,"source_snapshots")?)?;
  if o.len()!=2||refs.len()>crate::fit_point_input::MAX_POINTS||snapshots.len()>refs.len()
   ||(refs.is_empty()!=snapshots.is_empty()){return Err(2);}
  Ok(())
 }).unwrap_or(Err(7)).err().unwrap_or(0)
}
fn rows(v:&Json)->Result<&[Json],u32>{if let Json::Array(a)=v{Ok(a)}else{Err(1)}}
fn boolean(v:&Json)->Result<bool,u32>{if let Json::Boolean(b)=v{Ok(*b)}else{Err(1)}}
fn index(v:&Json,maximum:usize)->Result<usize,u32>{let n=number(v)?;if !n.is_finite()||n.fract()!=0.||n<0.||n>maximum as f64{return Err(2);}Ok(n as usize)}
fn points(v:&Json)->Result<Vec<[f64;3]>,u32>{
 let values=rows(v)?;if values.len()>crate::fit_point_input::MAX_POINTS{return Err(2);}
 values.iter().map(|v|{let p=array(v)?;if !crate::solid::finite(p){return Err(4);}Ok(p)}).collect()
}
fn axes(v:&Json)->Result<[[f64;3];3],u32>{let a=rows(v)?;if a.len()!=3{return Err(1);}Ok([array(&a[0])?,array(&a[1])?,array(&a[2])?])}
fn decode(c:&BTreeMap<String,Json>)->Result<(Request,Vec<[f64;3]>),u32>{
 if number(field(c,"schema_version")?)?!=1.||text(field(c,"operation")?)?!="ConstructSolid"
  ||text(field(c,"coordinates")?)?!="document_world_mm"||text(field(c,"frame_role")?)?!="session_input_frame_at_capture"
  ||text(field(c,"lifetime")?)?!="immutable_construction_input_evidence_not_current_validity"
  ||text(field(c,"source_provenance")?)?!="numeric_inputs_only_native_owners_bound_separately"{return Err(1);}
 let generation=text(field(c,"generation")?)?;let n=generation.parse::<u64>().map_err(|_|1u32)?;
 if n==0||n.to_string()!=generation{return Err(5);}
 let kind=match text(field(c,"kind")?)?{"Box"=>Kind::Box,"Sphere"=>Kind::Sphere,_=>return Err(1)};
 let mode=match text(field(c,"mode")?)?{"Corners"=>Mode::Corners,"Diagonal"=>Mode::Diagonal,"3Point"=>Mode::ThreePoint,
  "Vertical"=>Mode::Vertical,"Center"=>Mode::Center,"Cube"=>Mode::Cube,"2Point"=>Mode::TwoPoint,
  "4Point"=>Mode::FourPoint,"FitPoints"=>Mode::FitPoints,"AroundCurve"=>Mode::AroundCurve,"Tangent"=>Mode::Tangent,_=>return Err(1)};
 let normalized=points(field(c,"points")?)?;let accepted=points(field(c,"accepted_points")?)?;
 if accepted.is_empty(){return Err(2);}
 match mode{
  Mode::FitPoints if normalized!=accepted||normalized.len()<3=>return Err(2),
  Mode::Tangent if normalized!=accepted||!matches!(normalized.len(),2|3)
   ||(normalized.len()==2&&number(field(c,"radius")?)?<=0.)=>return Err(2),
  Mode::AroundCurve if !normalized.is_empty()||!matches!(accepted.len(),2|3)=>return Err(2),
  Mode::ThreePoint if kind==Kind::Sphere&&(normalized.len()!=2||accepted.len()!=3)=>return Err(2),
  Mode::FourPoint if kind==Kind::Sphere&&(normalized.len()!=2||accepted.len()!=4)=>return Err(2),
  _ if mode!=Mode::AroundCurve&&normalized.is_empty()=>return Err(2),_=>{},
 }
 let last=array(field(c,"last")?)?;if !crate::solid::finite(last){return Err(4);}
 if let Some(v)=c.get("previous_accepted"){if !matches!(v,Json::Other)&&!crate::solid::finite(array(v)?){return Err(4);}}
 let circle=match field(c,"circle")?{Json::Other=>None,v=>{let o=object(v)?;Some((array(field(o,"center")?)?,number(field(o,"radius")?)?,array(field(o,"normal")?)?))}};
 let types=rows(field(c,"constraint_types")?)?;if types.len()>3{return Err(2);}
 let request=Request{kind,mode,stage:Stage::Construct,phase:index(field(c,"phase")?,u32::MAX as usize)? as u32,
  point_count:normalized.len(),length:number(field(c,"length")?)?,width:number(field(c,"width")?)?,radius:number(field(c,"radius")?)?,
  point_constraint:boolean(field(c,"point_constraint")?)?,constraint_types:types.iter().map(boolean).collect::<Result<Vec<_>,_>>()?,
  first:array(field(c,"first")?)?,axes:axes(field(c,"axes")?)?,frame_origin:array(field(c,"frame_origin")?)?,frame_axes:axes(field(c,"frame_axes")?)?,circle,output:Some(array(field(c,"output")?)?)};
 operation_capability(OperationId::ConstructSolid,RepresentationKind::Point,OperationOptions::NativeSolid(request.clone())).map_err(|_|2u32)?;
 if text(field(c,"feature_id")?)?!=if kind==Kind::Box{"OM9-SOLID-012"}else{"OM9-SOLID-014"}{return Err(1);}
 Ok((request,accepted))
}
pub(crate) fn validate_output(snapshot:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>)->Result<(),u32>{
 if number(field(snapshot,"native_shape_type")?)?!=2.{return Err(1);}
 let topology=object(field(snapshot,"topology_data")?)?;
 if !matches!(field(topology,"faces")?,Json::Array(a) if !a.is_empty()){return Err(1);}
 let observed=object(field(topology,"validation")?)?;
 for key in ["native_valid","valid_solid","closed"]{if !boolean(field(validation,key)?)?||field(observed,key)?!=field(validation,key)?{return Err(1);}}
 Ok(())
}
fn discrete(snapshot:&BTreeMap<String,Json>,source_world:&Json)->Result<RepresentationKind,u32>{
 let data=object(field(snapshot,"geometry_data")?)?;
 let (kind,representation,ownership)=match text(field(data,"kind")?)?{"mesh"=>(8,RepresentationKind::Mesh,"private_native_copy"),"point_cloud"=>(9,RepresentationKind::PointCloud,"private_plain_copy"),_=>return Err(1)};
 if text(field(snapshot,"ownership")?)?!=ownership||text(field(data,"other_native_channels")?)?!="uninspected"{return Err(1);}
 let xyz=points(field(data,"points_xyz")?)?;
 let triangles=if kind==9{if !matches!(field(data,"triangles")?,Json::Other){return Err(3);}vec![]}else{
  let faces=rows(field(data,"triangles")?)?;if faces.len()>crate::discrete_snapshot::MAX_FACES{return Err(2);}
  faces.iter().map(|v|{let a=rows(v)?;if a.len()!=3{return Err(3);}
   Ok([index(&a[0],xyz.len())? as u64,index(&a[1],xyz.len())? as u64,index(&a[2],xyz.len())? as u64])}).collect::<Result<Vec<_>,u32>>()?
 };
 crate::discrete_snapshot::Geometry::capture(kind,&xyz,&triangles).map_err(|e|match e{crate::discrete_snapshot::Error::Kind=>1u32,crate::discrete_snapshot::Error::Count=>2,crate::discrete_snapshot::Error::Coordinate=>4,crate::discrete_snapshot::Error::Face=>3})?;
 let location=rows(field(snapshot,"native_discrete_location")?)?;if location.len()!=4{return Err(6);}
 let mut flat=[0.;16];for (i,row) in location.iter().enumerate(){flat[i*4..i*4+4].copy_from_slice(&array::<4>(row)?);}
 let parent=array(field(snapshot,"native_parent_to_world")?)?;
 let transform=crate::geometry_frame::Frame::compose(parent,crate::geometry_frame::IDENTITY,crate::geometry_frame::IDENTITY,flat).map_err(|_|6u32)?;
 let near=|a:f64,b:f64|(a-b).abs()<=64.*f64::EPSILON*a.abs().max(b.abs()).max(1.);
 if !transform.matrix().iter().zip(array::<16>(source_world)?).all(|(a,b)|near(*a,b)){return Err(6);}
 let selected=points(field(snapshot,"selected_points_world")?)?;
 if selected.len()!=xyz.len(){return Err(2);}
 for (point,expected) in xyz.iter().zip(selected){
  let observed=transform.transform(*point).map_err(|_|6u32)?;
  if !observed.iter().zip(expected).all(|(a,b)|near(*a,b)){return Err(1);}
 }
 object(field(snapshot,"topology_data")?)?;
 Ok(representation)
}
pub(crate) fn validate(c:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>)->Result<Vec<Json>,u32>{
 if number(field(c,"schema_version")?)?!=1.||text(field(c,"operation")?)?!="ConstructSolid"
  ||text(field(c,"attribute_role")?)?!="source_evidence_in_input_snapshots_output_attributes_unavailable"
  ||c.contains_key("native_transform_observations"){return Err(1);}
 frame(field(c,"output_parent_to_world")?)?;
 let construction=field(c,"construction")?;let (request,accepted)=decode(object(construction)?)?;
 if field(c,"feature_id")?!=field(object(construction)?,"feature_id")?{return Err(1);}
 let refs=rows(field(c,"source_refs")?)?;let snapshots=rows(field(c,"source_snapshots")?)?;
 let indexed=match c.get("source_snapshot_indexing"){
  None=>false,Some(v) if text(v)?=="input_snapshots_zero_based_including_numeric_input"=>true,_=>return Err(1),
 };
 if refs.len()>crate::fit_point_input::MAX_POINTS||snapshots.len()>refs.len()||(!indexed&&refs.len()!=snapshots.len()){return Err(2);}
 let units=text(field(output,"unit_context_snapshot")?)?;
 let revision=if units.is_empty(){Json::Other}else{string(&crate::units::Context::decode(units).map_err(|_|2u32)?.revision.to_string())};
 let numeric=map([("schema_version",Json::Number(1.)),("ownership",string("private_plain_copy")),
  ("unit_context_snapshot",string(units)),("unit_context_revision",revision.clone()),
  ("geometry_data",map([("representation",string("point_cloud")),("coordinates",string("document_world_mm")),
   ("points_xyz",field(object(construction)?,"accepted_points")?.clone()),("role",string("accepted_construction_points"))])),
  ("topology_data",map([])),("source_attributes",map([("schema_version",Json::Number(1.)),("availability",string("not_applicable_constructed_output"))]))]);
 let mut owned=vec![numeric];let mut document=None;let mut path_count=0;let mut tangent=0u32;let mut fit_count=0usize;let mut fit_end=0usize;
 let mut seen=vec![false;snapshots.len()];
 for (slot,reference) in refs.iter().enumerate(){
  let source=object(reference)?;stamp(source)?;frame(field(source,"world")?)?;
  if !indexed&&source.contains_key("input_snapshot_index"){return Err(1);}
  let snapshot_index=if indexed{index(field(source,"input_snapshot_index")?,snapshots.len())?.checked_sub(1).ok_or(1u32)?}else{slot};
  let snapshot=snapshots.get(snapshot_index).ok_or(1u32)?;
  let uid=text(field(source,"document_uid")?)?;let name=text(field(source,"object_name")?)?;
  if uid.is_empty()||name.is_empty(){return Err(1);}text(field(source,"subelement")?)?;
  let identity=(text(field(source,"document")?)?,uid);if document.is_some_and(|old|old!=identity){return Err(1);}document=Some(identity);
  let ancestors=rows(field(source,"ancestors")?)?;if ancestors.len()>64{return Err(2);}
  for a in ancestors{let a=object(a)?;stamp(a)?;if field(a,"document")?!=field(source,"document")?{return Err(1);}}
  let snapshot=object(snapshot)?;
  if number(field(&snapshot,"schema_version")?)?!=1.||text(field(source,"unit_context_snapshot")?)?!=units||text(field(&snapshot,"unit_context_snapshot")?)?!=units{return Err(5);}
  let a=field(&snapshot,"source_attributes")?;crate::source_attributes::SourceAttributes::decode(a).map_err(|_|2u32)?;
  if text(field(object(a)?,"document_uid")?)?!=uid||text(field(object(a)?,"object_name")?)?!=name{return Err(1);}
  let representation=if snapshot.contains_key("native_shape_type"){
   let topology=object(field(&snapshot,"topology_data")?)?;let validation=object(field(topology,"validation")?)?;
   let name=validate_native_snapshot(&snapshot,validation,true)?;
   if text(field(source,"snapshot_frame")?)?!="document_world"{return Err(6);}
   match name{"solid"=>RepresentationKind::Solid,"shell"=>RepresentationKind::Shell,"trimmed_face"=>RepresentationKind::TrimmedFace,
    "polycurve"=>RepresentationKind::Polycurve,"curve"=>RepresentationKind::Curve,"point"=>RepresentationKind::Point,_=>RepresentationKind::Brep}
  }else{if text(field(source,"snapshot_frame")?)?!="native_local_with_locations"{return Err(6);}discrete(&snapshot,field(source,"world")?)?};
  operation_capability(OperationId::ConstructSolid,representation,OperationOptions::NativeSolid(request.clone())).map_err(|_|2u32)?;
  let role_index=index(field(source,"role_index")?,crate::fit_point_input::MAX_POINTS)?;
  let begin=index(field(source,"point_begin")?,accepted.len())?;let end=index(field(source,"point_end")?,accepted.len())?;
  match text(field(source,"role")?)?{
   "path"=>{if request.mode!=Mode::AroundCurve||role_index!=0||begin!=0||end!=0||!matches!(representation,RepresentationKind::Curve|RepresentationKind::Polycurve){return Err(1);}path_count+=1;},
   "tangent"=>{if request.mode!=Mode::Tangent||role_index>=request.constraint_types.len()||request.constraint_types[role_index]||begin!=0||end!=0||tangent&(1<<role_index)!=0||!matches!(representation,RepresentationKind::Curve|RepresentationKind::Polycurve){return Err(1);}tangent|=1<<role_index;},
   "fit_points"=>{if request.mode!=Mode::FitPoints||role_index!=fit_count||begin<fit_end||end<=begin{return Err(1);}
    if points(field(&snapshot,"selected_points_world")?)?!=accepted[begin..end]{return Err(1);}fit_count+=1;fit_end=end;},
   _=>return Err(1),
  }
  seen[snapshot_index]=true;
 }
 if seen.iter().any(|used|!*used){return Err(1);}
 // Stable snapshot indices are independent of first-reference ordering.
 for snapshot in snapshots{let mut snapshot=object(snapshot)?.clone();snapshot.insert("unit_context_revision".into(),revision.clone());owned.push(Json::Object(snapshot));}
 if (request.mode==Mode::AroundCurve&&path_count!=1)||(request.mode!=Mode::AroundCurve&&path_count!=0){return Err(1);}
 if request.mode==Mode::Tangent{let expected=request.constraint_types.iter().enumerate().fold(0,|mask,(i,point)|if *point{mask}else{mask|1<<i});if expected!=tangent{return Err(1);}}
 Ok(owned)
}
