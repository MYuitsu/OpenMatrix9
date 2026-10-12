// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native lineage/point observations are values, never retained OCCT owners.
use crate::builder_history::Json;
use crate::prepared_result::{object,field,text,number,array,string,map,validate_native_snapshot,native_attributes};
use crate::geometry_frame::{Frame,IDENTITY};
use std::collections::{BTreeMap,BTreeSet};

pub(crate) struct Evidence {pub input:Json,pub topology:Json,pub deviation:Json,pub parameters:Json,pub diagnostics:Json}
fn rows<'a>(o:&'a BTreeMap<String,Json>,key:&str)->Result<&'a [Json],u32>{
 if let Json::Array(a)=field(o,key)?{Ok(a)}else{Err(1)}
}
fn index(v:&Json,maximum:usize)->Result<usize,u32>{
 let n=number(v)?;if n.fract()!=0.||n<1.||n>maximum as f64{return Err(8);}Ok(n as usize)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_builder_trace_counts(vertices:usize,edges:usize,faces:usize,samples:usize)->bool{
 crate::edge_geometry::counts_allowed(edges,vertices)&&crate::topology_snapshot::counts_allowed(faces,0,0)
  &&faces.checked_add(edges).and_then(|n|n.checked_add(vertices)).is_some_and(|n|n<=16384)
  &&samples<=262144
}
fn counts(snapshot:&BTreeMap<String,Json>)->Result<[usize;3],u32>{
 let topology=object(field(snapshot,"topology_data")?)?;
 let counts=[rows(topology,"vertices")?.len(),rows(topology,"edges_3d")?.len(),rows(topology,"faces")?.len()];
 if !om9_builder_trace_counts(counts[0],counts[1],counts[2],0){return Err(2);}
 // Native enumeration may differ from the snapshot row order. Its namespace
 // must nevertheless be a complete unique permutation of 1..=row_count.
 for (kind,key) in ["vertices","edges_3d","faces"].iter().enumerate(){
  let mut ids=BTreeSet::new();
  for row in rows(topology,key)?{
   let raw=text(field(object(row)?,"native_topology_index")?)?;
   let id=raw.parse::<usize>().map_err(|_|8u32)?;
   if id==0||id>counts[kind]||raw!=id.to_string()||!ids.insert(id){return Err(8);}
  }
 }
 Ok(counts)
}
fn domain(v:&Json)->Result<[f64;2],u32>{
 let a=array(v)?;if !crate::topology_snapshot::domain_valid(a[0],a[1]){return Err(2);}Ok(a)
}
fn uv_domain(v:&Json)->Result<[f64;4],u32>{
 let a=array(v)?;if !crate::topology_snapshot::domain_valid(a[0],a[1])||!crate::topology_snapshot::domain_valid(a[2],a[3]){return Err(2);}Ok(a)
}
pub(crate) fn inspect(value:&Json,context:&BTreeMap<String,Json>,output:&BTreeMap<String,Json>,revision:Json)->Result<Evidence,u32>{
 let observed=object(value)?;
 if number(field(observed,"schema_version")?)?!=1.||text(field(observed,"method")?)?!="composed_occt_modified_shape"{return Err(1);}
 let mut input=object(field(observed,"input_snapshot")?)?.clone();
 let input_topology=object(field(&input,"topology_data")?)?;
 validate_native_snapshot(&input,object(field(input_topology,"validation")?)?,true)?;
 if field(&input,"native_shape_type")?!=field(context,"template_native_shape_type")?
  ||field(&input,"unit_context_snapshot")?!=field(output,"unit_context_snapshot")?
  ||field(&input,"source_attributes")?!=field(output,"source_attributes")?{return Err(1);}
 native_attributes(field(&input,"source_attributes")?)?;
 let source_counts=counts(&input)?;let output_counts=counts(output)?;
 if source_counts!=output_counts{return Err(8);}
 let pairs=rows(observed,"pairs")?;
 if pairs.len()!=source_counts.iter().sum::<usize>(){return Err(8);}
 let source=object(field(context,"source_ref")?)?;
 let world=array(field(source,"world")?)?;let parent=array(field(context,"output_parent_to_world")?)?;
 let values=array::<6>(field(context,"affine")?)?;let mut affine=IDENTITY;
 for i in 0..3{affine[4*i+i]=values[i];affine[4*i+3]=values[i+3];}
 let source_frame=Frame::compose(world,affine,IDENTITY,IDENTITY).map_err(|_|6u32)?;
 let output_frame=Frame::compose(parent,IDENTITY,IDENTITY,IDENTITY).map_err(|_|6u32)?;
 let unit=text(field(output,"unit_context_snapshot")?)?;
 let tolerance=if unit.is_empty(){1e-6}else{crate::units::Context::decode(unit).map_err(|_|2u32)?.absolute_mm};
 let mut source_ids=BTreeSet::new();let mut target_ids=BTreeSet::new();
 let mut mappings=Vec::new();let mut queries=Vec::new();let mut face_queries=Vec::new();let mut unavailable=Vec::new();let mut total=0usize;let mut maximum=0f64;
 for pair in pairs{
  let p=object(pair)?;let kind=text(field(p,"kind")?)?;
  let ordinal=match kind{"vertex"=>0,"edge"=>1,"face"=>2,_=>return Err(1)};
  let si=index(field(p,"source_index")?,source_counts[ordinal])?;let oi=index(field(p,"output_index")?,output_counts[ordinal])?;
  if !source_ids.insert((ordinal,si))||!target_ids.insert((ordinal,oi)){return Err(8);}
  let samples=rows(p,"samples")?;
  let failures=if p.contains_key("query_failures"){rows(p,"query_failures")?}else{&[]};
  if !failures.is_empty()&&ordinal==0{return Err(1);}
  total=total.checked_add(samples.len()).ok_or(2u32)?;
  let attempts=total.checked_add(unavailable.len()).and_then(|n|n.checked_add(failures.len())).ok_or(2u32)?;
  if !om9_builder_trace_counts(source_counts[0],source_counts[1],source_counts[2],attempts){return Err(2);}
  if ordinal==0&&samples.len()!=1{return Err(1);}
  let domains=if ordinal==1&&(!samples.is_empty()||!failures.is_empty()){Some((domain(field(p,"source_domain")?)?,domain(field(p,"output_domain")?)?))}else{None};
  let uv_domains=if ordinal==2&&(!samples.is_empty()||!failures.is_empty()){Some((uv_domain(field(p,"source_uv_domain")?)?,uv_domain(field(p,"output_uv_domain")?)?))}else{None};
  let mut mapping=p.clone();mapping.remove("samples");
  for failure in failures{
   let failure=object(failure)?;
   if ordinal==1{let parameter=number(field(failure,"source_parameter")?)?;let (sd,_)=domains.ok_or(1u32)?;
    if parameter<sd[0]||parameter>sd[1]{return Err(2);}
   }else{let uv=array::<2>(field(failure,"source_uv")?)?;let (sd,_)=uv_domains.ok_or(1u32)?;
    for i in 0..2{if uv[i]<sd[2*i]||uv[i]>sd[2*i+1]{return Err(2);}}
   }
   let reason=text(field(failure,"reason")?)?;
   if text(field(failure,"error")?)?!="KernelFailure"||!matches!(reason,"no_projection_solution"|"native_projection_failure"|"projected_point_outside_trim")
     ||(reason=="projected_point_outside_trim"&&ordinal!=2){return Err(1);}
   source_frame.transform(array(field(failure,"source_point")?)?).map_err(|_|4u32)?;
   let mut owned=failure.clone();owned.insert("kind".into(),string(kind));
   owned.insert("source_index".into(),Json::Number(si as f64));owned.insert("output_index".into(),Json::Number(oi as f64));
   owned.insert("operation".into(),string("NativeBuilder.ParameterQuery"));
   owned.insert("effect".into(),string("parameter_query_unavailable_geometry_validity_independent"));
   unavailable.push(Json::Object(owned));
  }
  for sample in samples{
   let sample=object(sample)?;
   let source_point=array(field(sample,"source_point")?)?;let output_point=array(field(sample,"output_point")?)?;
   let expected=source_frame.transform(source_point).map_err(|_|4u32)?;
   let measured=output_frame.transform(output_point).map_err(|_|4u32)?;
   let delta=std::array::from_fn::<_,3,_>(|i|measured[i]-expected[i]);
   let residual=delta[0].hypot(delta[1]).hypot(delta[2]);
   if !residual.is_finite(){return Err(4);}if residual>tolerance{return Err(8);}
   maximum=maximum.max(residual);
   if let Some((sd,od))=domains{
    let a=number(field(sample,"source_parameter")?)?;let b=number(field(sample,"output_parameter")?)?;
    if a<sd[0]||a>sd[1]||b<od[0]||b>od[1]{return Err(2);}
    let mut query=sample.clone();query.insert("source_index".into(),Json::Number(si as f64));query.insert("output_index".into(),Json::Number(oi as f64));
    query.insert("residual".into(),Json::Number(residual));queries.push(Json::Object(query));
   }
   if let Some((sd,od))=uv_domains{
    let a=array::<2>(field(sample,"source_uv")?)?;let b=array::<2>(field(sample,"output_uv")?)?;
    for i in 0..2{if a[i]<sd[2*i]||a[i]>sd[2*i+1]||b[i]<od[2*i]||b[i]>od[2*i+1]{return Err(2);}}
    let mut query=sample.clone();query.insert("source_index".into(),Json::Number(si as f64));query.insert("output_index".into(),Json::Number(oi as f64));
    query.insert("residual".into(),Json::Number(residual));face_queries.push(Json::Object(query));
   }
  }
  mapping.insert("sample_count".into(),Json::Number(samples.len() as f64));mappings.push(Json::Object(mapping));
 }
 if total==0{return Err(8);}
 input.insert("unit_context_revision".into(),revision);
 Ok(Evidence{input:Json::Object(input),topology:map([("method",string("composed_occt_modified_shape")),
  ("index_namespace",string("snapshot_native_topology_index")),("complete",Json::Boolean(true)),("rows",Json::Array(mappings))]),
  deviation:map([("measurement",string("native_point_samples")),("frame",string("document_world")),("length_unit",string("mm")),
   ("sample_count",Json::Number(total as f64)),("max_distance",Json::Number(maximum)),("acceptance_tolerance",Json::Number(tolerance)),("global_bound",Json::Other)]),
  diagnostics:Json::Array(unavailable.clone()),
  parameters:map([("method",string("native_parameter_query_samples")),("evaluation",string("recorded_parameters_only")),
   ("edge_queries",Json::Array(queries)),("face_queries",Json::Array(face_queries)),("global_mapping",Json::Other),
   ("recorded_queries_complete",Json::Boolean(unavailable.is_empty())),("unavailable_queries",Json::Array(unavailable))])})
}
