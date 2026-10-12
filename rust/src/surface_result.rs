// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned result admission for the existing native Loft/Sweep solver.
//! Native masters and closure/validity are observations, not Rust kernel proofs.
use crate::builder_history::Json;
use crate::prepared_result::{object,field,text,number,frame,stamp,validate_native_snapshot,string};
use crate::geometry_contract::{operation_capability,OperationId,OperationOptions,RepresentationKind};
use std::collections::BTreeMap;

pub(crate) fn constructed_attributes(v:&Json)->Result<(),u32>{
 let o=object(v)?;
 if o.len()!=2||number(field(o,"schema_version")?)?!=1.
  ||text(field(o,"availability")?)?!="not_applicable_constructed_output"{return Err(1);}
 Ok(())
}
pub(crate) fn validate_output(snapshot:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>)->Result<(),u32>{
 if !matches!(number(field(snapshot,"native_shape_type")?)?,0.|1.|3.|4.){return Err(1);}
 let topology=object(field(snapshot,"topology_data")?)?;
 if !matches!(field(topology,"faces")?,Json::Array(faces) if !faces.is_empty())
  ||!matches!(field(validation,"native_valid")?,Json::Boolean(true))
  ||!matches!(field(validation,"valid_solid")?,Json::Boolean(false)){return Err(1);}
 let observed=object(field(topology,"validation")?)?;
 for key in ["native_valid","valid_solid"]{if field(observed,key)?!=field(validation,key)?{return Err(1);}}
 Ok(())
}
pub(crate) fn validate(c:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>)->Result<Vec<Json>,u32>{
 if number(field(c,"schema_version")?)?!=1.||text(field(c,"operation")?)?!="EditSurface"
  ||text(field(c,"source_snapshot_frame")?)?!="document_world"
  ||text(field(c,"attribute_role")?)?!="source_evidence_in_input_snapshots_output_attributes_unavailable"
  ||c.contains_key("native_transform_observations"){return Err(1);}
 frame(field(c,"output_parent_to_world")?)?;
 let Json::Array(refs)=field(c,"source_refs")? else{return Err(1);};
 let Json::Array(snapshots)=field(c,"source_snapshots")? else{return Err(1);};
 if refs.len()!=snapshots.len(){return Err(1);}
 let mut saved=crate::surface_history::decode_saved(text(field(c,"recipe")?)?,refs.len()).map_err(|_|2u32)?;
 let r=&mut saved.request;r.stage=crate::native_surface_capability::Stage::Construct;
 let mask=number(field(c,"rail_closed")?)?;
 if mask.fract()!=0.||!(0. ..=3.).contains(&mask){return Err(1);}
 let rails=r.kind.rails();let mask=mask as u32;
 if (rails==0&&mask!=0)||(rails==1&&mask&2!=0){return Err(1);}
 r.rail_closed=[mask&1!=0,mask&2!=0];
 operation_capability(OperationId::EditSurface,RepresentationKind::Polycurve,OperationOptions::NativeSurface(r.clone())).map_err(|_|2u32)?;
 let id=match (r.kind,r.history_command){
  (crate::surface::Kind::Sweep1,true)=>"OM9-SURFACE-002",
  (crate::surface::Kind::Sweep2,true)=>"OM9-SURFACE-004",_=>r.kind.feature_id(),
 };
 if text(field(c,"feature_id")?)?!=id{return Err(1);}
 constructed_attributes(field(output,"source_attributes")?)?;
 let units=text(field(output,"unit_context_snapshot")?)?;
 let revision=if units.is_empty(){Json::Other}else{string(&crate::units::Context::decode(units).map_err(|_|2u32)?.revision.to_string())};
 let mut owned=Vec::with_capacity(snapshots.len());let mut document=None;
 for (source,snapshot) in refs.iter().zip(snapshots){
  let source=object(source)?;stamp(source)?;frame(field(source,"world")?)?;
  let uid=text(field(source,"document_uid")?)?;let name=text(field(source,"object_name")?)?;
  if uid.is_empty()||name.is_empty(){return Err(1);}text(field(source,"subelement")?)?;
  let identity=(text(field(source,"document")?)?,uid);
  if document.is_some_and(|previous|previous!=identity){return Err(1);}document=Some(identity);
  let Json::Array(ancestors)=field(source,"ancestors")? else{return Err(1);};
  if ancestors.len()>64{return Err(2);}for a in ancestors{let a=object(a)?;stamp(a)?;
   if field(a,"document")?!=field(source,"document")?{return Err(1);}}
  let mut snapshot=object(snapshot)?.clone();
  if text(field(source,"unit_context_snapshot")?)?!=units||text(field(&snapshot,"unit_context_snapshot")?)?!=units{return Err(1);}
  if !matches!(number(field(&snapshot,"native_shape_type")?)?,5.|6.){return Err(1);}
  let topology=object(field(&snapshot,"topology_data")?)?;
  let validation=object(field(topology,"validation")?)?;
  validate_native_snapshot(&snapshot,validation,true)?;
  if !matches!(field(topology,"faces")?,Json::Array(a) if a.is_empty())
   ||!matches!(field(validation,"native_valid")?,Json::Boolean(true))
   ||!matches!(field(validation,"valid_solid")?,Json::Boolean(false)){return Err(1);}
  let attributes=field(&snapshot,"source_attributes")?;
  crate::source_attributes::SourceAttributes::decode(attributes).map_err(|_|2u32)?;
  let a=object(attributes)?;
  if text(field(a,"document_uid")?)?!=uid||text(field(a,"object_name")?)?!=name{return Err(1);}
  snapshot.insert("unit_context_revision".into(),revision.clone());owned.push(Json::Object(snapshot));
 }
 Ok(owned)
}
