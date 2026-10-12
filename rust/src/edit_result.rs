// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned CAD evidence for the existing Edit solvers. Kernel facts are supplied
//! by the native adapter; this module does not parse or solve an OCCT master.
use crate::builder_history::Json;
use crate::prepared_result::{object,field,text,number,frame,stamp,string,validate_native_snapshot};
use crate::geometry_contract::{operation_capability,OperationId,OperationOptions,RepresentationKind};
use std::collections::BTreeMap;
fn integer(v:&Json)->Result<u32,u32>{let n=number(v)?;
 if !n.is_finite()||n.fract()!=0.||!(0. ..=1_048_576.).contains(&n){return Err(2);}Ok(n as u32)
}
fn representation(snapshot:&BTreeMap<String,Json>)->Result<RepresentationKind,u32>{
 Ok(match number(field(snapshot,"native_shape_type")?)? {
  0.|1.=>RepresentationKind::Brep,2.=>RepresentationKind::Solid,3.=>RepresentationKind::Shell,
  4.=>RepresentationKind::TrimmedFace,5.=>RepresentationKind::Polycurve,
  6.=>RepresentationKind::Curve,7.=>RepresentationKind::Point,_=>return Err(1),
 })
}
fn native(snapshot:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>)->Result<(),u32>{
 validate_native_snapshot(snapshot,validation,true)?;
 if !matches!(field(validation,"native_valid")?,Json::Boolean(true)){return Err(3);}
 let topology=object(field(snapshot,"topology_data")?)?;let observed=object(field(topology,"validation")?)?;
 for key in ["native_valid","valid_solid"]{if field(observed,key)?!=field(validation,key)?{return Err(1);}}
 if representation(snapshot)?==RepresentationKind::Solid&&!matches!(field(validation,"valid_solid")?,Json::Boolean(true)){return Err(3);}
 Ok(())
}
pub(crate) fn validate_output(c:&BTreeMap<String,Json>,snapshot:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>)->Result<(),u32>{
 native(snapshot,validation)?;let r=representation(snapshot)?;use RepresentationKind::*;
 let allowed=match integer(field(c,"kind")?)? {
  1=>matches!(r,Curve|Polycurve|TrimmedFace|Shell),2=>true,
  3=>matches!(r,Curve|Polycurve|TrimmedFace|Shell),4..=7=>matches!(r,Solid|TrimmedFace|Shell|Brep),_=>false,
 };
 if !allowed{return Err(1);}Ok(())
}
pub(crate) fn validate(c:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>)->Result<Vec<Json>,u32>{
 if number(field(c,"schema_version")?)?!=1.
  ||text(field(c,"source_snapshot_frame")?)?!="document_world"
  ||text(field(c,"attribute_role")?)?!="source_evidence_in_input_snapshots_output_attributes_unavailable"
  ||c.contains_key("native_transform_observations"){return Err(1);}
 let kind=integer(field(c,"kind")?)?;let flags=integer(field(c,"flags")?)?;let mode=integer(field(c,"mode")?)?;
 let k=crate::native_edit_capability::kind(kind).ok_or(1u32)?;
 let operation=if kind>=4{OperationId::Boolean}else{OperationId::NativeEdit};
 if text(field(c,"operation")?)?!=if kind>=4{"Boolean"}else{"NativeEdit"}
  ||text(field(c,"feature_id")?)?!=k.feature(){return Err(1);}
 frame(field(c,"output_parent_to_world")?)?;
 crate::surface_result::constructed_attributes(field(output,"source_attributes")?)?;
 let Json::Array(refs)=field(c,"source_refs")? else{return Err(1);};
 let Json::Array(snapshots)=field(c,"source_snapshots")? else{return Err(1);};
 if refs.is_empty()||refs.len()!=snapshots.len(){return Err(1);}
 let first=integer(field(c,"first_count")?)? as usize;
 if first>refs.len()||matches!(kind,4|5)&&(first==0||first==refs.len())
  ||kind==7&&(refs.len()!=2||first!=1){return Err(2);}
 let count=integer(field(c,"output_count")?)?;let index=integer(field(c,"output_index")?)?;
 if count==0||index>=count{return Err(2);}
 let owner=field(c,"output_owner_index")?;
 if kind==2{if integer(owner)? as usize>=refs.len(){return Err(2);}}
 else if !matches!(owner,Json::Other){return Err(1);}
 let tolerance=number(field(c,"tolerance_mm")?)?;
 if !tolerance.is_finite()||tolerance<=0.||tolerance>1e150{return Err(2);}
 let report=field(c,"tolerance_report")?;
 if !matches!(report,Json::Other){if kind!=1||text(report)?.is_empty(){return Err(1);}}
 let projection=field(c,"projection")?;
 if !matches!(projection,Json::Other){
  if kind!=3||flags&2==0{return Err(1);}
  let p=object(projection)?;let matrix=crate::prepared_result::array::<12>(field(p,"frame")?)?;
  if matrix.iter().any(|v|!v.is_finite()||v.abs()>1e150)||!matches!(field(p,"perspective")?,Json::Boolean(_)){return Err(6);}
 }else if kind==3&&flags&2!=0{return Err(1);}
 let units=text(field(output,"unit_context_snapshot")?)?;
 let revision=if units.is_empty(){Json::Other}else{string(&crate::units::Context::decode(units).map_err(|_|2u32)?.revision.to_string())};
 let mut owned=Vec::with_capacity(snapshots.len());let mut document=None;
 for (source,snapshot) in refs.iter().zip(snapshots){
  let source=object(source)?;stamp(source)?;frame(field(source,"world")?)?;
  let uid=text(field(source,"document_uid")?)?;let name=text(field(source,"object_name")?)?;
  if uid.is_empty()||name.is_empty()||!text(field(source,"subelement")?)?.is_empty(){return Err(1);}
  let identity=(text(field(source,"document")?)?,uid);
  if document.is_some_and(|previous|previous!=identity){return Err(1);}document=Some(identity);
  let Json::Array(ancestors)=field(source,"ancestors")? else{return Err(1);};
  if ancestors.len()>64{return Err(2);}for a in ancestors{let a=object(a)?;stamp(a)?;
   if field(a,"document")?!=field(source,"document")?{return Err(1);}}
  let mut snapshot=object(snapshot)?.clone();
  if text(field(source,"unit_context_snapshot")?)?!=units||text(field(&snapshot,"unit_context_snapshot")?)?!=units{return Err(1);}
  let topology=object(field(&snapshot,"topology_data")?)?;let validation=object(field(topology,"validation")?)?;
  native(&snapshot,validation)?;
  operation_capability(operation,representation(&snapshot)?,OperationOptions::NativeEdit{kind,flags,mode}).map_err(|_|2u32)?;
  let attrs=field(&snapshot,"source_attributes")?;
  crate::source_attributes::SourceAttributes::decode(attrs).map_err(|_|2u32)?;let attrs=object(attrs)?;
  if text(field(attrs,"document_uid")?)?!=uid||text(field(attrs,"object_name")?)?!=name{return Err(1);}
  snapshot.insert("unit_context_revision".into(),revision.clone());owned.push(Json::Object(snapshot));
 }
 let Json::Array(deleted)=field(c,"deleted_source_names")? else{return Err(1);};
 let count=if flags&4!=0{refs.len()}else if kind==4{first}else{0};
 if deleted.len()!=count{return Err(1);}
 for (name,source) in deleted.iter().zip(refs){
  if text(name)?!=text(field(object(source)?,"object_name")?)?{return Err(1);}}
 Ok(owned)
}
