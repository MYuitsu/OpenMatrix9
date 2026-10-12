// SPDX-License-Identifier: LGPL-2.1-or-later
//! Atomic admission of owned native input values; no document/kernel pointer.
use super::{CurveSession,Effect,input_log};
use crate::builder_history::Json;
use crate::prepared_result::{object,field,text,number,array,frame,stamp,string,validate_native_snapshot};
use std::collections::BTreeMap;
#[path="curve_native_contacts.rs"]
mod contacts;
fn rows(v:&Json)->Result<&[Json],u32>{if let Json::Array(a)=v{Ok(a)}else{Err(1)}}
fn index(v:&Json,max:usize)->Result<usize,u32>{let n=number(v)?;if !n.is_finite()||n.fract()!=0.||n<0.||n>max as f64{return Err(2);}Ok(n as usize)}
fn identity(v:&Json,zero:bool)->Result<u64,u32>{let s=text(v)?;let n=s.parse::<u64>().map_err(|_|1u32)?;if (!zero&&n==0)||n.to_string()!=s{return Err(5);}Ok(n)}
fn vector(v:&Json)->Result<[f64;3],u32>{let p=array(v)?;if p.iter().any(|n|!n.is_finite()){return Err(4);}Ok(p)}
fn text_field<'a>(o:&'a BTreeMap<String,Json>,key:&str)->Result<&'a str,u32>{text(field(o,key)?)}
fn matrix(v:&Json)->Result<[f64;16],u32>{
 let rows=rows(v)?;if rows.len()!=4{return Err(6);}let mut m=[0.;16];
 for(i,row)in rows.iter().enumerate(){m[i*4..i*4+4].copy_from_slice(&array::<4>(row)?);}
 frame(&crate::curve::input_log::vector(&m))?;Ok(m)
}
fn sources(o:&BTreeMap<String,Json>,action:&str,kind:u32,begin:usize,limit:usize,p:[f64;3],points:&[[f64;3]])->Result<(Vec<Json>,Vec<Json>),u32>{
 let refs=rows(field(o,"source_refs")?)?;let snapshots=rows(field(o,"source_snapshots")?)?;
 if refs.len()>1024||snapshots.len()>refs.len()||refs.is_empty()!=snapshots.is_empty(){return Err(2);}
 if refs.is_empty()&&(matches!(action,"point"|"fit_batch")||(action=="reference"&&kind!=2)){return Err(1);}
 let units=text(field(o,"unit_context_snapshot")?)?;if !units.is_empty(){crate::units::Context::decode(units).map_err(|_|5u32)?;}
 let mut used=vec![false;snapshots.len()];let mut document=None;let mut offset=begin;
 for value in refs{
  let r=object(value)?;stamp(r)?;frame(field(r,"world")?)?;
  let uid=text(field(r,"document_uid")?)?;let name=text(field(r,"object_name")?)?;let sub=text(field(r,"subelement")?)?;
  if uid.is_empty()||name.is_empty()||name.contains('\0')||sub.contains('\0'){return Err(1);}
  let doc=(text(field(r,"document")?)?,uid);if document.is_some_and(|old|old!=doc){return Err(1);}document=Some(doc);
  let ancestors=rows(field(r,"ancestors")?)?;if ancestors.len()>64{return Err(2);}
  for ancestor in ancestors{let a=object(ancestor)?;stamp(a)?;if field(a,"document")?!=field(r,"document")?{return Err(1);}}
  let slot=index(field(r,"input_snapshot_index")?,snapshots.len())?.checked_sub(1).ok_or(1u32)?;used[slot]=true;
  let snapshot=object(&snapshots[slot])?;
  if number(field(snapshot,"schema_version")?)?!=1.||text(field(r,"unit_context_snapshot")?)?!=units||text(field(snapshot,"unit_context_snapshot")?)?!=units{return Err(5);}
  let attrs=field(snapshot,"source_attributes")?;crate::source_attributes::SourceAttributes::decode(attrs).map_err(|_|2u32)?;
  let attrs=object(attrs)?;if text(field(attrs,"document_uid")?)?!=uid||text(field(attrs,"object_name")?)?!=name{return Err(1);}
  if snapshot.contains_key("native_shape_type"){
   let topology=object(field(snapshot,"topology_data")?)?;
   validate_native_snapshot(snapshot,object(field(topology,"validation")?)?,true)?;
   let data=object(field(snapshot,"geometry_data")?)?;matrix(field(data,"native_root_location")?)?;
   if !matches!(text(field(data,"native_root_orientation")?)?,"forward"|"reversed"|"internal"|"external"){return Err(3);}
   if text(field(r,"snapshot_frame")?)?!="document_world"{return Err(6);}
  }else{
   validate_discrete(snapshot,r)?;
   if text(field(r,"snapshot_frame")?)?!="native_local_with_locations"{return Err(6);}
  }
  let role=text(field(r,"role")?)?;index(field(r,"role_index")?,limit)?;
  let first=index(field(r,"point_begin")?,limit)?;let end=index(field(r,"point_end")?,limit)?;if first>end{return Err(2);}
  match action{
   "fit_batch"=>{
    if role!="fit_points"||first!=offset||end>begin+points.len(){return Err(1);}
    if snapshot.contains_key("native_shape_type"){crate::owned_point_selection::validate(r,snapshot,false)?;}
    let selected=rows(field(snapshot,"selected_points_world")?)?;
    let selected=selected.iter().map(vector).collect::<Result<Vec<_>,_>>()?;
    if selected.as_slice()!=&points[first-begin..end-begin]{return Err(1);}offset=end;
   },
   "point" if role=="point"||role=="radius_probe"||role=="direction"=>{
    if refs.len()!=1||first!=begin||end!=begin+1||rows(field(snapshot,"selected_points_world")?)?!=[input_log::vector(&p)]{return Err(1);}
    if snapshot.contains_key("native_shape_type")&&(sub.starts_with("Vertex")||number(field(snapshot,"native_shape_type")?)?==7.){crate::owned_point_selection::validate(r,snapshot,false)?;}
   },
   "reference" if (kind==4&&role=="direction")||(kind==5&&role=="point")=>{
    if refs.len()!=1||first!=begin||end!=begin+1||rows(field(snapshot,"selected_points_world")?)?!=[input_log::vector(&p)]{return Err(1);}
    if snapshot.contains_key("native_shape_type")&&(sub.starts_with("Vertex")||number(field(snapshot,"native_shape_type")?)?==7.){crate::owned_point_selection::validate(r,snapshot,false)?;}
   },
   "reference" if (kind==1&&role=="path")||(kind==3&&role=="tangent")=>{},
   _=>return Err(1),
  }
 }
 if used.iter().any(|b|!*b)||(action=="fit_batch"&&offset!=begin+points.len()){return Err(1);}
 Ok((refs.to_vec(),snapshots.to_vec()))
}
pub(crate) fn validate_discrete(snapshot:&BTreeMap<String,Json>,source:&BTreeMap<String,Json>)->Result<(),u32>{
 let data=object(field(snapshot,"geometry_data")?)?;
 let (kind,ownership)=match text(field(data,"kind")?)?{"point_cloud"=>(9,"private_plain_copy"),"mesh"=>(8,"private_native_copy"),_=>return Err(1)};
 if text(field(snapshot,"ownership")?)?!=ownership||text(field(data,"other_native_channels")?)?!="uninspected"{return Err(1);}
 let xyz=rows(field(data,"points_xyz")?)?.iter().map(vector).collect::<Result<Vec<_>,_>>()?;
 let faces=if kind==9{if !matches!(field(data,"triangles")?,Json::Other){return Err(3);}Vec::new()}else{
  rows(field(data,"triangles")?)?.iter().map(|v|{let row=rows(v)?;if row.len()!=3{return Err(3);}Ok([index(&row[0],xyz.len())? as u64,index(&row[1],xyz.len())? as u64,index(&row[2],xyz.len())? as u64])}).collect::<Result<Vec<_>,u32>>()?
 };
 crate::discrete_snapshot::Geometry::capture(kind,&xyz,&faces).map_err(|e|match e{crate::discrete_snapshot::Error::Coordinate=>4u32,crate::discrete_snapshot::Error::Face=>3,_=>2})?;
 let local=matrix(field(snapshot,"native_discrete_location")?)?;
 let parent=array(field(snapshot,"native_parent_to_world")?)?;
 let transform=crate::geometry_frame::Frame::compose(parent,crate::geometry_frame::IDENTITY,crate::geometry_frame::IDENTITY,local).map_err(|_|6u32)?;
 let near=|a:f64,b:f64|(a-b).abs()<=64.*f64::EPSILON*a.abs().max(b.abs()).max(1.);
 if !transform.matrix().iter().zip(array::<16>(field(source,"world")?)?).all(|(a,b)|near(*a,b)){return Err(6);}
 // The full source is retained independently of the selected subset. Native
 // subelement/index provenance remains in the source ref; no all-points alias.
 let selected=rows(field(snapshot,"selected_points_world")?)?;
 if selected.is_empty()||selected.len()>1024{return Err(2);}
 let indices=match source.get("selected_point_indices"){
  Some(v)=>{let values=rows(v)?;if values.len()!=selected.len(){return Err(1);}values.iter().map(|v|index(v,xyz.len().saturating_sub(1))).collect::<Result<Vec<_>,_>>()?},
  None if text(field(source,"subelement")?)?.is_empty()&&selected.len()==xyz.len()=>(0..xyz.len()).collect(),
  _=>return Err(1),
 };
 for(value,index)in selected.iter().zip(indices){
  let expected=transform.transform(*xyz.get(index).ok_or(1u32)?).map_err(|_|4u32)?;let value=vector(value)?;
  if !expected.into_iter().zip(value).all(|(a,b)|near(a,b)){return Err(1);}
 }
 object(field(snapshot,"topology_data")?)?;Ok(())
}
impl CurveSession{
 pub fn accept_native_json(&mut self,raw:&str,generation:u64)->Result<Effect,String>{
  self.accepted_input_log.admit_count()?;
  let mut source_hint=String::from("unavailable source");
  let mut parse=||->Result<_,u32>{
   use crate::prepared_result::{MAX_RESULT_BYTES,MAX_RESULT_NODES,MAX_RESULT_DEPTH};
   let v=crate::builder_history::parse_document_at_depth(raw,MAX_RESULT_BYTES,MAX_RESULT_NODES,MAX_RESULT_DEPTH-4).map_err(|_|2u32)?;
   let o=object(&v)?;
   if let Some(Json::Array(refs))=o.get("source_refs"){
    let names=refs.iter().take(16).filter_map(|v|{let r=object(v).ok()?;let name=text(r.get("object_name")?).ok()?;let sub=r.get("subelement").and_then(|v|text(v).ok()).unwrap_or("");
     Some(format!("{}{}{}",name.chars().take(256).collect::<String>(),if sub.is_empty(){""}else{"."},sub.chars().take(128).collect::<String>()))}).collect::<Vec<_>>();
    if !names.is_empty(){source_hint=names.join(", ");}
   }
   if !self.active||!matches!(self.name,"Line"|"Polyline"|"Rectangle"|"InterpCrv"|"Circle"|"Ellipse"){return Err(5);}
   if number(field(o,"schema_version")?)?!=1.||text(field(o,"operation")?)?!="AcceptCurveNativeInput"||text(field(o,"command")?)?!=self.name{return Err(1);}
   if generation==0||identity(field(o,"generation")?,false)?!=generation||identity(field(o,"context_before")?,true)?!=self.circle_input_context_key()
    ||identity(field(o,"admission_revision")?,true)?!=self.native_input_revision(){return Err(5);}
   let units=crate::prepared_result::text(field(o,"unit_context_snapshot")?)?;
   let model_mm=if units.is_empty(){1.}else{crate::units::Context::decode(units).map_err(|_|5u32)?.model_mm};
   if model_mm!=self.input_mm_per_unit{return Err(5);}
   let action=text(field(o,"action")?)?.to_owned();let kind=index(field(o,"reference_kind")?,5)? as u32;
   let p=vector(field(o,"point_world")?)?;let normal=vector(field(o,"normal")?)?;
   let points=rows(field(o,"points_world")?)?.iter().map(vector).collect::<Result<Vec<_>,_>>()?;
   if points.len()>1024||(action=="fit_batch"&&(points.is_empty()||self.name!="Circle")){return Err(2);}
   if !matches!(action.as_str(),"reference"|"point"|"fit_batch"|"tangent_solution"){return Err(1);}
   if action=="reference"&&!matches!(self.name,"Circle"|"Ellipse"){return Err(1);}
   let text=match field(o,"text")?{Json::Other=>None,v=>{let text=text(v)?;if text.len()>1024||text.contains('\0'){return Err(2);}Some(text.to_owned())}};
   let limit=match self.name{"Line"|"Polyline"=>4096,"InterpCrv"=>crate::spline::MAX_POLES,"Circle"=>1024,_=>3};
   let(refs,snapshots)=sources(o,&action,kind,self.points.len(),limit,p,&points)?;
   let fraction=if action=="reference"&&matches!(kind,2|3){
    let explicit=o.get("source_curve_fraction").filter(|v|**v!=Json::Other).map(number).transpose()?;
    let typed=if kind==2{if let Some(text)=&text{let t=text.trim();if t.get(..8).is_some_and(|v|v.eq_ignore_ascii_case("OnCurve=")){Some(t[8..].trim().parse::<f64>().map_err(|_|2u32)?)}else{None}}else{None}}else{None};
    if explicit.is_some()&&typed.is_some()&&explicit!=typed{return Err(2);}
    let fraction=explicit.or(typed).ok_or(1u32)?;
    let(reference,snapshot)=if kind==2{
     let(r,s)=self.accepted_input_log.latest_native_role("path").ok_or(1u32)?;(object(r)?,object(s)?)
    }else{
     if refs.len()!=1{return Err(1);}let r=object(&refs[0])?;let slot=index(field(r,"input_snapshot_index")?,snapshots.len())?.checked_sub(1).ok_or(1u32)?;
     (r,object(snapshots.get(slot).ok_or(1u32)?)?)
    };
    source_hint=format!("{}{}{}",text_field(reference,"object_name")?,if text_field(reference,"subelement")?.is_empty(){""}else{"."},text_field(reference,"subelement")?);
    let(point,tangent)=crate::owned_curve_input::selected_reference(reference,snapshot)?.at_fraction(fraction)?;
    if !p.into_iter().zip(point).all(|(a,b)|(a-b).abs()<=1e-6+128.*f64::EPSILON*a.abs().max(b.abs()).max(1.))
     ||!normal.into_iter().zip(tangent).all(|(a,b)|(a-b).abs()<=1e-8){return Err(5);}
    if kind==3{let selected=rows(field(snapshot,"selected_points_world")?)?;if selected.len()!=1||vector(&selected[0])?!=p{return Err(5);}}
    // Old typed OnCurve events already retain the original fraction token.
    // Validate it without changing their canonical historical event bytes.
    explicit
   }else{None};
   let solution=if action=="tangent_solution"{
    if self.name!="Circle"||kind!=0||!refs.is_empty()||!snapshots.is_empty()||!points.is_empty()||!self.circle_options.tangent_ready(){return Err(1);}
    let radius=number(field(o,"solution_radius")?)?;
    let plan=crate::circle::Plan::new(p,normal,radius).map_err(|_|2u32)?;
    let fractions=field(o,"resolved_contact_fractions")?;contacts::validate(self,&plan,fractions,field(o,"resolved_contact_indexing")?)?;
    Some((radius,fractions.clone()))
   }else{None};
   Ok((action,kind,p,normal,points,text,refs,snapshots,fraction,solution))
  };
  let(action,kind,p,normal,points,text,refs,snapshots,fraction,solution)=parse().map_err(|status|{
   let code=match status{4=>"NonFiniteGeometry",5=>"StaleGeometry",6=>"SingularTransform",3=>"InvalidTrimTopology",7=>"KernelFailure",_=>"InvalidBasis"};
   format!("{code}: {} input {source_hint}: native input/source envelope rejected (status {status})",self.name)
  })?;
  let mut event=self.input_event(match action.as_str(){"reference"=>"native_reference","fit_batch"=>"native_fit_batch","tangent_solution"=>"native_solution",_=>"point"},[
   ("point_world",input_log::vector(&p)),("normal",input_log::vector(&normal)),("reference_kind",Json::Number(kind as f64)),
   ("points_world",input_log::points(&points)),("text",text.map_or(Json::Other,|t|string(&t))),
   ("native_source_count",Json::Number(refs.len() as f64)),
  ]);
  if let Json::Object(o)=&mut event{
   o.insert("native_input_role".into(),string("owned_source_values_at_admission_not_current_source_validity"));
   if let Some(fraction)=fraction{o.insert("source_curve_fraction".into(),Json::Number(fraction));}
   if let Some((radius,contacts))=&solution{
    o.insert("center".into(),input_log::vector(&p));o.insert("radius".into(),Json::Number(*radius));
    o.insert("role".into(),string("kernel_output_not_original_user_input"));
    o.insert("resolved_contact_fractions".into(),contacts.clone());o.insert("resolved_contact_indexing".into(),string("current_constraint_points_zero_based"));
   }
  }
  let parent=self.accepted_input_log.clone();let prepared=parent.prepare_native(event,refs,snapshots).map_err(|e|format!("{e}: {} input {source_hint}",self.name))?;
  // Non-Circle solvers also may update options while preparing a point. Keep
  // all admission/solver failures atomic by publishing only the private clone.
  let mut candidate=self.clone();
  let next_revision=candidate.next_native_input_revision()?;
  let accept=|s:&mut CurveSession,key:Option<u64>|{
   let effect=match action.as_str(){
    "point"=>s.point_impl(p),"fit_batch"=>s.circle_fit_batch_impl(&points),"tangent_solution"=>s.circle_accept_spatial_solution_impl(p,normal,solution.as_ref().ok_or("Missing checked native solution")?.0),
    _ if s.name=="Circle"=>s.circle_native_reference_impl(p,normal,kind),_=>s.ellipse_native_reference_impl(p,normal,kind),
   }?;
   if effect!=Effect::Cancelled{s.native_input_revision=next_revision;s.accepted_input_log=parent.append(prepared,key.unwrap_or_else(||s.circle_input_context_key()),next_revision);}
   Ok(effect)
  };
  let effect=if action=="tangent_solution"{accept(&mut candidate,None)}else{candidate.with_circle_input_checkpoint(accept)}.map_err(|e|format!("InvalidBasis: {} input {source_hint}: {e}",self.name))?;
  *self=candidate;Ok(effect)
 }
}
