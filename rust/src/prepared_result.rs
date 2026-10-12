// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned prepared CAD GeometryResult, native master and inspected facts.
use crate::builder_history::{Json,parse_document};
use crate::prepared_cad::{om9_prepared_cad_admit,om9_native_cad_result_admit,om9_prepared_binding_field};
use crate::units::Context;
use std::collections::BTreeMap;
use std::fmt::{self,Write};
pub const MAX_RESULT_BYTES:usize=16*1024*1024;
pub const MAX_MASTER_BYTES:usize=8*1024*1024;
pub const MAX_RESULT_NODES:usize=1_000_000;
pub const MAX_RESULT_DEPTH:usize=64;
pub(crate) fn object(v:&Json)->Result<&BTreeMap<String,Json>,u32>{if let Json::Object(o)=v{Ok(o)}else{Err(1)}}
pub(crate) fn field<'a>(o:&'a BTreeMap<String,Json>,key:&str)->Result<&'a Json,u32>{o.get(key).ok_or(1)}
pub(crate) fn text(v:&Json)->Result<&str,u32>{if let Json::String(s)=v{Ok(s)}else{Err(1)}}
pub(crate) fn number(v:&Json)->Result<f64,u32>{if let Json::Number(n)=v{Ok(*n)}else{Err(1)}}
fn tri(v:&Json)->Result<u32,u32>{match v{Json::Other=>Ok(0),Json::Boolean(false)=>Ok(1),Json::Boolean(true)=>Ok(2),_=>Err(1)}}
pub(crate) fn string(s:&str)->Json{Json::String(s.to_owned())}
pub(crate) fn map(fields:impl IntoIterator<Item=(&'static str,Json)>)->Json{Json::Object(fields.into_iter().map(|(k,v)|(k.to_owned(),v)).collect())}
fn count(v:&Json)->Result<usize,u32>{let n=number(v)?;if n.fract()!=0.||!(0. ..=1024.).contains(&n){return Err(2);}Ok(n as usize)}
pub(crate) fn array<const N:usize>(v:&Json)->Result<[f64;N],u32>{
 let Json::Array(a)=v else{return Err(1);};if a.len()!=N{return Err(1);}
 let mut result=[0.;N];for(i,v)in a.iter().enumerate(){result[i]=number(v)?;}Ok(result)
}
fn native_kind(v:&Json)->Result<crate::geometry_contract::RepresentationKind,u32>{
 use crate::geometry_contract::RepresentationKind as Kind;
 Ok(match number(v)? {0.|1.=>Kind::Brep,2.=>Kind::Solid,3.=>Kind::Shell,4.=>Kind::TrimmedFace,
  5.=>Kind::Polycurve,6.=>Kind::Curve,7.=>Kind::Point,_=>return Err(1)})
}
pub(crate) fn stamp(o:&BTreeMap<String,Json>)->Result<(),u32>{
 for key in ["document","object","generation","revision","epoch"]{
  let s=text(field(o,key)?)?;let n=s.parse::<u64>().map_err(|_|1u32)?;
  if n==0||n.to_string()!=s{return Err(1);}
 }Ok(())
}
pub(crate) fn frame(v:&Json)->Result<(),u32>{
 let matrix=array(v)?;let i=crate::geometry_frame::IDENTITY;
 crate::geometry_frame::Frame::compose(matrix,i,i,i).map_err(|_|6u32)?;Ok(())
}
fn builder_context(o:&BTreeMap<String,Json>,snapshot:&BTreeMap<String,Json>)->Result<(),u32>{
 use crate::geometry_contract::*;
 if number(field(o,"schema_version")?)?!=1.||text(field(o,"operation")?)?!="NativeBuilder"{return Err(1);}
 let slot=count(field(o,"template_slot")?)?;let templates=count(field(o,"template_count")?)?;
 if slot>=templates{return Err(2);}
 let scale=match field(o,"scale_to_gem")?{Json::Boolean(b)=>*b,_=>return Err(1)};
 let initial=array(field(o,"initial_dimensions")?)?;let current=array(field(o,"current_dimensions")?)?;
 let id=text(field(o,"feature_id")?)?;let raw=text(field(o,"recipe")?)?;
 let request=crate::native_builder_capability::Request{stage:crate::native_builder_capability::Stage::Construct,
  feature_id:id.into(),recipe:raw.into(),scale_to_gem:scale,record_count:1,template_count:templates,target_count:1,
  dimensions:Some((initial,current))};
 for key in ["source_native_shape_type","template_native_shape_type"]{
  operation_capability(OperationId::NativeBuilder,native_kind(field(o,key)?)?,OperationOptions::NativeBuilder(request.clone())).map_err(|_|1u32)?;
 }
 let recipe=crate::builder_history::validate_recipe(raw).map_err(|_|2u32)?;
 let expected=crate::builder_history::affine_plan(&recipe,initial,current,scale).map_err(|_|2u32)?;
 if array::<6>(field(o,"affine")?)?!=expected{return Err(2);}
 let source=object(field(o,"source_ref")?)?;stamp(source)?;
 for key in ["document_uid","object_name"]{if text(field(source,key)?)?.is_empty(){return Err(1);}}
 if text(field(source,"unit_context_snapshot")?)?!=text(field(snapshot,"unit_context_snapshot")?)?{return Err(1);}
 frame(field(source,"world")?)?;frame(field(o,"output_parent_to_world")?)?;
 let Json::Array(ancestors)=field(source,"ancestors")? else{return Err(1);};
 if ancestors.len()>64{return Err(2);}for a in ancestors{stamp(object(a)?)?;}
 if text(field(o,"attribute_role")?)?!="immutable_template_source_evidence"{return Err(1);}
 Ok(())
}
pub(crate) fn native_attributes(v:&Json)->Result<(),u32>{
 let o=object(v)?;
 if o.len()==2&&matches!(o.get("schema_version"),Some(Json::Number(1.)))
  &&matches!(o.get("availability"),Some(Json::String(s)) if s=="unavailable_legacy_record"){return Ok(());}
 crate::source_attributes::SourceAttributes::decode(v).map_err(|_|2u32)?;Ok(())
}
/// Bound persisted template metadata before a native JSON decoder or shape copy.
pub fn validate_template_attributes(raw:&str,templates:usize)->Result<(),u32>{
 if !crate::native_builder_capability::om9_native_builder_counts(1,templates,1){return Err(2);}
 if raw.is_empty(){return Ok(());} // Existing FCStd recipes did not capture attributes.
 let Json::Array(rows)=parse_document(raw,MAX_RESULT_BYTES,MAX_RESULT_NODES).map_err(|_|2u32)? else{return Err(1);};
 if rows.len()!=templates{return Err(1);}for row in &rows{crate::source_attributes::SourceAttributes::decode(row).map_err(|_|2u32)?;}Ok(())
}
/// Caller retains one initialized immutable UTF8 allocation for this call.
/// Header checks cannot prove pointer provenance; Rust retains no pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_builder_attributes_validate(input:*const u8,length:usize,templates:usize)->u32{
 if length>MAX_RESULT_BYTES||length>isize::MAX as usize||(!input.is_null()&&(input as usize).checked_add(length).is_none())
  ||(length!=0&&input.is_null()){return 2;}
 std::panic::catch_unwind(||{
  let raw=if length==0{""}else{match std::str::from_utf8(unsafe{std::slice::from_raw_parts(input,length)}){Ok(s)=>s,Err(_)=>return 1}};
  validate_template_attributes(raw,templates).err().unwrap_or(0)
 }).unwrap_or(7)
}
fn binding(o:&BTreeMap<String,Json>)->Result<(),u32>{
 for (index,key) in ["name","layer","tolerance","locked","visible","color"].into_iter().enumerate(){
  let v=field(o,key)?;let(kind,value,count)=match v{
   Json::String(_)=>(1,0.,0),Json::Boolean(_)=>(3,0.,0),Json::Number(n)=>(2,*n,0),Json::Array(a)=>(4,0.,a.len()),_=>(5,0.,0)};
  let status=om9_prepared_binding_field(index as u32,kind,value,count);if status!=0{return Err(status);}
 }
 if let Json::Array(channels)=field(o,"color")? {for channel in channels{
  let status=om9_prepared_binding_field(6,2,number(channel)?,0);if status!=0{return Err(status);}
 }}
 for key in ["source_uuid","class_name"] {if let Some(value)=o.get(key){if !matches!(value,Json::Other){text(value)?;}}}
 if let Some(value)=o.get("wire_density"){
  let n=number(value)?;let status=om9_prepared_binding_field(7,6,n,0);if status!=0{return Err(status);}
 }
 Ok(())
}
struct Output(String);
impl Write for Output {
 fn write_str(&mut self,s:&str)->fmt::Result{
  if !self.0.len().checked_add(s.len()).is_some_and(|n|n<=MAX_RESULT_BYTES){return Err(fmt::Error);}
  self.0.push_str(s);Ok(())
 }
}
fn quoted(w:&mut Output,s:&str)->fmt::Result{
 w.write_char('"')?;for c in s.chars(){match c{
  '"'=>w.write_str("\\\"")?,'\\'=>w.write_str("\\\\")?,'\n'=>w.write_str("\\n")?,
  '\r'=>w.write_str("\\r")?,'\t'=>w.write_str("\\t")?,c if c<' '=>write!(w,"\\u{:04x}",c as u32)?,c=>w.write_char(c)?}}
 w.write_char('"')
}
fn write_json(w:&mut Output,value:&Json)->fmt::Result {match value{
 Json::Other=>w.write_str("null"),Json::String(s)=>quoted(w,s),Json::Number(n)=>write!(w,"{n}"),Json::Boolean(b)=>write!(w,"{b}"),
 Json::Array(rows)=>{w.write_char('[')?;for(i,row)in rows.iter().enumerate(){if i!=0{w.write_char(',')?;}write_json(w,row)?;}w.write_char(']')},
 Json::Object(o)=>{w.write_char('{')?;for(i,(k,v))in o.iter().enumerate(){if i!=0{w.write_char(',')?;}quoted(w,k)?;w.write_char(':')?;write_json(w,v)?;}w.write_char('}')}
}}
pub(crate) fn encode_owned(value:&Json)->Result<String,u32>{
 validate_output_tree(value,0,&mut 0)?;
 let mut output=Output(String::new());write_json(&mut output,value).map_err(|_|2u32)?;Ok(output.0)
}
// Result wrapping/metadata can exceed the request's bounds. Inspect the actual
// owned output before writing bytes; recursion stops before descending past64.
fn validate_output_tree(value:&Json,depth:usize,nodes:&mut usize)->Result<(),u32>{
 if depth>MAX_RESULT_DEPTH||*nodes>=MAX_RESULT_NODES{return Err(2);}
 *nodes+=1;
 match value{
  Json::Array(rows)=>for row in rows{validate_output_tree(row,depth+1,nodes)?;},
  Json::Object(fields)=>for row in fields.values(){validate_output_tree(row,depth+1,nodes)?;},
  _=>{},
 }
 Ok(())
}
pub(crate) fn validate_native_snapshot(snapshot:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>,builder:bool)->Result<&'static str,u32>{
 let data=object(field(snapshot,"geometry_data")?)?;
 if number(field(snapshot,"schema_version")?)?!=1. {return Err(1);}
 let root=number(field(snapshot,"native_shape_type")?)?;
 if root.fract()!=0. || !(0. ..=7.).contains(&root){return Err(1);}
 let representation=match root as u32{0|1=>"brep",2=>"solid",3=>"shell",4=>"trimmed_face",5=>"polycurve",6=>"curve",7=>"point",_=>return Err(1)};
 let admit=if builder{om9_native_cad_result_admit}else{om9_prepared_cad_admit};
 let status=admit(root as u32,tri(field(validation,"native_valid")?)?,tri(field(validation,"valid_solid")?)?);
 if status!=0{return Err(status);}
 if text(field(snapshot,"ownership")?)?!="private_native_copy"{return Err(1);}
 if text(field(data,"representation")?)?!=representation || text(field(data,"coordinates")?)?!="native_local_with_locations"{return Err(1);}
 let master=object(field(data,"native_master")?)?;
 if text(field(master,"encoding")?)?!="occt_brep_ascii_v3" || text(field(master,"sdk_version")?)?.is_empty(){return Err(1);}
 let payload=text(field(master,"payload")?)?;
 if payload.len()>MAX_MASTER_BYTES{return Err(2);}
 if !payload.trim_start().starts_with("CASCADE Topology V3"){return Err(1);}
 let topology=object(field(snapshot,"topology_data")?)?;
 for key in ["faces","edges_3d","vertices"]{if !matches!(field(topology,key)?,Json::Array(_)){return Err(1);}}
 Ok(representation)
}
pub fn assemble(raw:&str)->Result<String,u32>{
 if raw.len()>MAX_RESULT_BYTES{return Err(2);}
 let request=crate::builder_history::parse_document_at_depth(raw,MAX_RESULT_BYTES,MAX_RESULT_NODES,MAX_RESULT_DEPTH).map_err(|reason|if matches!(reason,"JSON budget"|"JSON byte budget"){2u32}else{1u32})?;
 let request=object(&request)?;
 let mut snapshot=object(field(request,"snapshot")?)?.clone();
 let context=request.get("operation_context").map(object).transpose()?;
 let edit=match context {
  Some(c) if matches!(text(field(c,"operation")?)?,"NativeEdit"|"Boolean")=>Some(crate::edit_result::validate(c,&snapshot)?),
  _=>None,
 };
 let solid=match context {
  Some(c) if text(field(c,"operation")?)?=="ConstructSolid"=>Some(crate::solid_result::validate(c,&snapshot)?),
  _=>None,
 };
 let curve=match context {
  Some(c) if text(field(c,"operation")?)?=="ConstructCurve"=>Some(crate::curve::result::validate(c,&snapshot)?),
  _=>None,
 };
 let circle_history=match context{
  Some(c)if text(field(c,"operation")?)?=="ReplayCircleHistory"=>Some(crate::circle_history::result::validate(c,&snapshot)?),_=>None,
 };
 let surface=match context {
  Some(c) if text(field(c,"operation")?)?=="EditSurface"=>Some(crate::surface_result::validate(c,&snapshot)?),
  Some(_) if edit.is_some()||solid.is_some()||curve.is_some()||circle_history.is_some()=>None,
  Some(c)=>{builder_context(c,&snapshot)?;None},None=>None,
 };
 if number(field(&snapshot,"schema_version")?)?!=1. {return Err(1);}
 let validation=object(field(request,"validation")?)?;
 if solid.is_some(){crate::solid_result::validate_output(&snapshot,validation)?;}
 if curve.is_some(){crate::curve::result::validate_output(context.unwrap(),&snapshot,validation)?;}
 if circle_history.is_some(){crate::circle_history::result::validate_output(context.unwrap(),&snapshot,validation)?;}
 if surface.is_some(){crate::surface_result::validate_output(&snapshot,validation)?;}
 if edit.is_some(){crate::edit_result::validate_output(context.unwrap(),&snapshot,validation)?;}
 let representation=if snapshot.contains_key("native_shape_type"){
 validate_native_snapshot(&snapshot,validation,context.is_some())?
 }else{
  if context.is_some(){return Err(1);}
  object(field(&snapshot,"topology_data")?)?;
  crate::prepared_discrete::validate(object(field(&snapshot,"geometry_data")?)?,text(field(&snapshot,"ownership")?)?,validation)?
 };
 if surface.is_some()||edit.is_some()||solid.is_some()||curve.is_some()||circle_history.is_some(){crate::surface_result::constructed_attributes(field(&snapshot,"source_attributes")?)?;}
 else if context.is_some(){native_attributes(field(&snapshot,"source_attributes")?)?;}
 else{binding(object(field(&snapshot,"source_attributes")?)?)?;}
 let unit_raw=text(field(&snapshot,"unit_context_snapshot")?)?;
 let revision=if unit_raw.is_empty(){Json::Other}else{string(&Context::decode(unit_raw).map_err(|_|2u32)?.revision.to_string())};
 snapshot.insert("unit_context_revision".to_owned(),revision.clone());
 let observed=context.filter(|_|surface.is_none()&&edit.is_none()&&solid.is_none()&&curve.is_none()&&circle_history.is_none()).and_then(|c|c.get("native_transform_observations"))
  .map(|o|crate::builder_transform::inspect(o,context.unwrap(),&snapshot,revision)).transpose()?;
 let identity=object(field(request,"input_identity")?)?;
 if text(field(identity,"document_name")?)?.is_empty(){return Err(1);}
 let conversion=|phase:&str,from:Json,warning:&str|map([
  ("phase",string(phase)),("from",from),("to",string(representation)),("exactness",string("uninspected")),
  ("preserved_fields",Json::Array(vec![])),("dropped_fields",Json::Array(vec![])),("deviation_report",Json::Other),
  ("locus",Json::Other),("parameterization",Json::Other),("topology",Json::Other),("parameter_map",Json::Other),
  ("warnings",Json::Array(vec![string(warning)]))]);
 let mut result_validation=validation.clone();
 if !snapshot.contains_key("native_shape_type"){result_validation.insert("discrete_values_valid".to_owned(),Json::Boolean(true));}
 if representation=="mesh"{result_validation.insert("archive_correspondence".to_owned(),string("oriented_triangle_multiset_at_float32_precision"));}
 let copy_phase=if snapshot.contains_key("native_shape_type"){"prepared_native_to_private_copy"}else{"prepared_discrete_to_private_copy"};
 let mut reports=if circle_history.is_some(){
  vec![conversion("native_circle_history_replay",string("owned_current_history_inputs"),"Existing independent Circle replay and native output basis observed; global locus/deviation and parameter/topology maps are uninspected")]
 }else if curve.is_some(){
  vec![conversion("native_curve_construction",string("accepted_curve_inputs"),
   "Existing native Curve construction and validity observed; global locus/deviation and parameter/topology maps are uninspected")]
 }else if solid.is_some(){
  vec![conversion("native_solid_construction",string("accepted_construction_inputs"),
   "Native solid construction and validity observed; global source-to-output deviation and parameter or topology maps are uninspected")]
 }else if edit.is_some(){
  vec![conversion("native_edit_construction",string("native_edit_inputs"),
   "Native edit and validity observed; source-to-output deviation and global parameter or topology maps are uninspected")]
 }else if surface.is_some(){
  vec![conversion("native_surface_construction",string("ordered_curve_inputs"),
   "Native construction and validity observed; source-to-output deviation and global parameter or topology maps are uninspected")]
 }else if let Some(context)=context{
  let kind=number(field(context,"template_native_shape_type")?)? as usize;
  let from=["brep","brep","solid","shell","trimmed_face","polycurve","curve","point"][kind];
  vec![conversion("native_affine_template_to_output",string(from),
   "Native transform and validity observed; deviation, parameter and topology correspondence are not yet measured")]}else{
  vec![conversion("upstream_archive_conversion",Json::Other,"Upstream archive conversion is not certified by prepared geometry validation"),
   conversion(copy_phase,string(representation),"Private owned geometry and inspected fields do not certify an upstream parameter map")]};
 if let Some(e)=&observed{let Json::Object(report)=&mut reports[0]else{return Err(7);};
  report.insert("deviation_report".into(),e.deviation.clone());report.insert("parameter_map".into(),e.parameters.clone());
  report.insert("exactness".into(),string("sampled_only"));
  let warning=if let Json::Array(diagnostics)=&e.diagnostics{if diagnostics.is_empty(){
   "Recorded native query samples and subshape history do not certify global locus or parameterization preservation"
  }else{"Some native parameter queries are unavailable; successful samples and geometry validity do not certify global locus or parameterization preservation"}}else{return Err(7);};
  report.insert("warnings".into(),Json::Array(vec![string(warning)]));
 }
 let mut public_context=context.cloned();if let Some(c)=&mut public_context{c.remove("native_transform_observations");}
 if curve.is_some(){if let Some(c)=&mut public_context{
  let recipe=object(field(c,"construction")?)?.clone();c.insert("feature_id".into(),field(&recipe,"feature_id")?.clone());
  c.insert("source_refs".into(),field(&recipe,"source_refs")?.clone());c.insert("source_snapshot_indexing".into(),string("input_snapshots_zero_based_including_numeric_input"));
 }}
 if surface.is_some()||edit.is_some()||solid.is_some()||circle_history.is_some(){if let Some(c)=&mut public_context{c.remove("source_snapshots");}}
 let operation=if circle_history.is_some(){"ReplayCircleHistory"}else if curve.is_some(){"ConstructCurve"}else if solid.is_some(){"ConstructSolid"}else if edit.is_some(){text(field(context.unwrap(),"operation")?)?}else if surface.is_some(){"EditSurface"}else if context.is_some(){"NativeBuilder"}else{"CommitImport3dm"};
 let result=map([
  ("schema_version",Json::Number(1.)),("status",string("accepted")),("operation",string(operation)),
  ("input_identity",Json::Object(identity.clone())),("validation",Json::Object(result_validation)),
  ("lifetime",string(if context.is_some(){"immutable_native_operation_evidence_not_current_validity"}else{"immutable_import_evidence_not_current_validity"})),
  ("operation_context",public_context.map(Json::Object).unwrap_or(Json::Other)),
  ("input_snapshots",Json::Array(circle_history.or(curve).or(solid).or(edit).or(surface).unwrap_or_else(||observed.as_ref().map(|e|vec![e.input.clone()]).unwrap_or_default()))),
  ("output_snapshots",Json::Array(vec![Json::Object(snapshot)])),
  ("conversion_reports",Json::Array(reports)),
  ("diagnostics",observed.as_ref().map(|e|e.diagnostics.clone()).unwrap_or(Json::Array(vec![]))),
  ("topology_map",observed.map(|e|e.topology).unwrap_or(Json::Other))]);
 encode_owned(&result)
}
#[repr(C)]
#[derive(Debug,Clone,Copy)]
pub struct Summary {pub status:u32,pub reserved:u32,pub needed:usize}
/// Synchronous initialized immutable UTF8 bytes and writable output; neither
/// pointer retained. Null output/capacity0 queries; alloc/free stay caller-side.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_prepared_result_json(input:*const u8,length:usize,output:*mut u8,capacity:usize)->Summary {
 let rejected=|status|Summary{status,reserved:0,needed:0};
 if length==0||length>MAX_RESULT_BYTES||length>isize::MAX as usize{return rejected(2);}
 if input.is_null()||capacity>MAX_RESULT_BYTES||(!output.is_null()&&capacity>isize::MAX as usize)|| (output.is_null()&&capacity!=0){return rejected(1);}
 std::panic::catch_unwind(||{
  let bytes=unsafe{std::slice::from_raw_parts(input,length)};
  let Ok(raw)=std::str::from_utf8(bytes)else{return rejected(1);};
  let json=match assemble(raw){Ok(s)=>s,Err(status)=>return rejected(status)};
  let needed=json.len();
  if output.is_null()&&capacity==0{return Summary{status:0,reserved:0,needed};}
  if capacity<needed{return Summary{status:2,reserved:0,needed};}
  unsafe{std::ptr::copy_nonoverlapping(json.as_ptr(),output,needed)};
  Summary{status:0,reserved:0,needed}
 }).unwrap_or_else(|_|rejected(7))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_prepared_master_limit()->usize{MAX_MASTER_BYTES}
#[unsafe(no_mangle)]
pub extern "C" fn om9_prepared_result_limit()->usize{MAX_RESULT_BYTES}

#[cfg(test)]
mod output_budget_tests {
 use super::*;
 fn request(extra:&str)->String {
  format!(r#"{{"snapshot":{{"schema_version":1,"native_shape_type":6,"ownership":"private_native_copy","unit_context_snapshot":"","geometry_data":{{"representation":"curve","coordinates":"native_local_with_locations","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 fixture"}}}},"topology_data":{{"faces":[],"edges_3d":[],"vertices":[],"retained_extension":{extra}}},"source_attributes":{{"name":"owned","layer":"fixture","locked":false,"visible":true,"tolerance":0.000001,"color":[20,40,60]}}}},"input_identity":{{"document_name":"fixture","name":"owned"}},"validation":{{"native_valid":true,"valid_solid":false}}}}"#)
 }
 fn nested(depth:usize)->String{format!("{}0{}","[".repeat(depth),"]".repeat(depth))}
 #[test]
 fn request_at_depth_limit_cannot_publish_an_overdepth_result(){
  // Request/snapshot/topology/extension starts at depth3. Output wraps the
  // snapshot in an array, making the same retained value one level deeper.
  let raw=request(&nested(61));
  assert!(parse_document(&raw,MAX_RESULT_BYTES,MAX_RESULT_NODES).is_ok());
  assert_eq!(assemble(&raw),Err(2));
 }
 #[test]
 fn generated_result_metadata_consumes_the_shared_node_budget(){
  // Independently counted fixed request:34 nodes including the extension array.
  // Its999966 scalar children make exactly1million request nodes.
  let raw=request(&format!("[{}]",vec!["0";999_966].join(",")));
  assert!(raw.len()<MAX_RESULT_BYTES);
  assert!(parse_document(&raw,MAX_RESULT_BYTES,MAX_RESULT_NODES).is_ok());
  assert!(parse_document(&raw,MAX_RESULT_BYTES,MAX_RESULT_NODES-1).is_err());
  assert!(matches!(assemble(&raw),Err(2)),"request fits, but generated metadata must not overflow the result node limit");
 }
 #[test]
 fn output_depth_boundary_retains_geometry_and_roundtrips(){
  let raw=request(&nested(60));let output=assemble(&raw).unwrap();
  let parsed=parse_document(&output,MAX_RESULT_BYTES,MAX_RESULT_NODES).unwrap();
  let snapshot=object(&parsed).unwrap().get("output_snapshots").unwrap();
  let Json::Array(snapshots)=snapshot else{panic!("output snapshots")};
  let topology=object(object(&snapshots[0]).unwrap().get("topology_data").unwrap()).unwrap();
  assert_eq!(topology.get("retained_extension"),Some(&parse_document(&nested(60),1024,100).unwrap()));
 }
 #[test]
 fn output_budget_rejection_does_not_copy_into_the_caller_buffer(){
  let raw=request(&nested(61));let mut output=vec![0xa5;4096];
  let result=unsafe{om9_prepared_result_json(raw.as_ptr(),raw.len(),output.as_mut_ptr(),output.len())};
  assert_eq!(result.status,2);assert_eq!(result.needed,0);assert_eq!(output,vec![0xa5;4096]);
 }
}
