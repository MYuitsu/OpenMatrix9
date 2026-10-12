// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent native point/FitPoints selection on owned geometry. No handles.
use crate::builder_history::Json;
use std::collections::BTreeMap;
use crate::prepared_result::{object,field,text,number,array};
use crate::geometry_frame::{Frame,IDENTITY};
use crate::nurbs_basis::{Axis,SurfaceBasis};
fn rows(v:&Json)->Result<&[Json],u32>{if let Json::Array(a)=v{Ok(a)}else{Err(1)}}
pub(crate) fn local_point(v:&Json)->Result<[f64;3],u32>{
 let p=array(v)?;let limit=crate::support_snapshot::policy().max_coordinate;
 if p.iter().any(|v|!v.is_finite()||v.abs()>limit){return Err(4);}Ok(p)
}
fn xyz(v:&Json)->Result<[f64;3],u32>{local_point(v)}
fn world(frame:&Frame,p:[f64;3])->Result<[f64;3],u32>{
 let p=frame.transform(p).map_err(|_|6u32)?;crate::circle::check_point(p).map_err(|_|4u32)?;Ok(p)
}
fn integer(v:&Json,max:usize)->Result<usize,u32>{let n=number(v)?;if !n.is_finite()||n.fract()!=0.||n<0.||n>max as f64{return Err(2);}Ok(n as usize)}
fn source_point_matches(a:[f64;3],b:[f64;3])->bool{a.into_iter().zip(b).all(|(a,b)|(a-b).abs()<=1e-6+128.*f64::EPSILON*a.abs().max(b.abs()).max(1.))}

fn location(row:&BTreeMap<String,Json>)->Result<Frame,u32>{
 if text(field(row,"geometry_coordinates")?)?!="native_local"{return Err(6);}
 let a=rows(field(row,"native_location")?)?;if a.len()!=4{return Err(6);}
 let mut m=[0.;16];for(i,row)in a.iter().enumerate(){m[i*4..i*4+4].copy_from_slice(&array::<4>(row)?);}
 // CAD capture is document_world: each native location already embeds the
 // complete own/parent transform. SourceRef.world is never applied again.
 Frame::compose(IDENTITY,IDENTITY,IDENTITY,m).map_err(|_|6u32)
}
fn native_index(row:&BTreeMap<String,Json>)->Result<usize,u32>{
 let s=text(field(row,"native_topology_index")?)?;let i=s.parse::<usize>().map_err(|_|3u32)?;
 if i==0||i.to_string()!=s{return Err(3);}Ok(i)
}
fn element<'a>(a:&'a[Json],s:&str)->Result<&'a BTreeMap<String,Json>,u32>{
 let i=s.parse::<usize>().map_err(|_|3u32)?;if i==0||i.to_string()!=s{return Err(3);}
 let mut found=None;for row in a{let row=object(row)?;if native_index(row)?==i{if found.is_some(){return Err(3);}found=Some(row);}}
 found.ok_or(3)
}
struct AxisData{degree:usize,knots:Vec<f64>,mults:Vec<usize>,periodic:bool,domain:[f64;2]}
impl AxisData{
 fn borrowed(&self)->Axis<'_>{Axis{degree:self.degree,knots:&self.knots,multiplicities:&self.mults,periodic:self.periodic,domain:self.domain}}
 fn decode(v:&Json)->Result<Self,u32>{
  let a=object(v)?;let policy=crate::support_snapshot::policy();
  let degree=integer(field(a,"degree")?,policy.max_degree)?;if degree==0{return Err(2);}
  let k=rows(field(a,"knots")?)?;let m=rows(field(a,"multiplicities")?)?;
  if !(2..=policy.max_knots).contains(&k.len())||k.len()!=m.len(){return Err(2);}
  let periodic=match field(a,"periodic")?{Json::Boolean(b)=>*b,_=>return Err(1)};
  Ok(Self{degree,knots:k.iter().map(number).collect::<Result<Vec<_>,_>>()?,mults:m.iter().map(|v|integer(v,degree+1)).collect::<Result<Vec<_>,_>>()?,periodic,domain:array(field(a,"full_domain")?)?})
 }
 fn bezier(degree:&Json,domain:&Json)->Result<Self,u32>{
  let degree=integer(degree,crate::support_snapshot::policy().max_degree)?;if degree==0{return Err(2);}
  Ok(Self{degree,knots:vec![0.,1.],mults:vec![degree+1;2],periodic:false,domain:array(domain)?})
 }
}
fn surface_poles(row:&BTreeMap<String,Json>)->Result<Option<Vec<[f64;3]>>,u32>{
 // faceSupportSnapshot may unwrap a rectangular trim for inspection. Native
 // FitPoints downcasts the original surface only; do not treat its unwrapped
 // support poles as a direct selectable native surface.
 let kind=match text(field(row,"native_class")?)?{"Geom_BSplineSurface"=>"nurbs_surface","Geom_BezierSurface"=>"bezier_surface",_=>return Ok(None)};
 let data=object(field(row,"geometry_data")?)?;
 if number(field(data,"schema_version")?)?!=1.||text(field(data,"kind")?)?!=kind||text(field(data,"pole_layout")?)?!="u_major"{return Err(2);}
 let u=integer(field(data,"u_count")?,1024)?;let v=integer(field(data,"v_count")?,1024)?;
 let count=u.checked_mul(v).ok_or(2u32)?;if u<2||v<2||count>1024{return Err(2);}
 let p=rows(field(data,"poles_xyz")?)?;if p.len()!=count{return Err(2);}
 let w=match field(data,"weights")?{Json::Other=>None,w=>Some(rows(w)?)};if w.is_some_and(|w|w.len()!=count){return Err(2);}
 let axes=if kind=="nurbs_surface"{
  let a=rows(field(data,"axes")?)?;if a.len()!=2{return Err(2);}[AxisData::decode(&a[0])?,AxisData::decode(&a[1])?]
 }else{
  let degrees=rows(field(data,"degrees")?)?;let domains=rows(field(data,"full_domains")?)?;
  if degrees.len()!=2||domains.len()!=2{return Err(2);}[AxisData::bezier(&degrees[0],&domains[0])?,AxisData::bezier(&degrees[1],&domains[1])?]
 };
 let poles=p.iter().map(xyz).collect::<Result<Vec<_>,_>>()?;
 let weights=w.map(|w|w.iter().map(number).collect::<Result<Vec<_>,_>>()).transpose()?;
 let basis=SurfaceBasis{poles:&poles,weights:weights.as_deref(),u_count:u,v_count:v,axes:[axes[0].borrowed(),axes[1].borrowed()]};
 if kind=="bezier_surface"{crate::support_snapshot::BezierSurface::capture(basis).map_err(|_|2u32)?;}else{basis.validate(crate::support_snapshot::policy()).map_err(|_|2u32)?;}
 let frame=location(row)?;Ok(Some(poles.into_iter().map(|p|world(&frame,p)).collect::<Result<Vec<_>,_>>()?))
}
fn curve_poles(value:&Json)->Result<Option<Vec<[f64;3]>>,u32>{
 let row=object(value)?;
 let class=match field(row,"native_class")?{Json::Other if field(row,"curve_present")?==&Json::Boolean(false)=>return Ok(None),v=>text(v)?};
 let kind=match class{"Geom_BSplineCurve"=>"nurbs_curve","Geom_BezierCurve"=>"bezier_curve",_=>return Ok(None)};
 let data=object(field(row,"geometry_data")?)?;if text(field(data,"kind")?)?!=kind{return Err(2);}
 let poles=rows(field(data,"poles_xyz")?)?;if poles.is_empty()||poles.len()>1024{return Err(2);}
 crate::owned_curve_input::validate_control_curve(field(row,"geometry_data")?)?;
 let frame=location(row)?;
 Ok(Some(poles.iter().map(|p|world(&frame,xyz(p)?)).collect::<Result<Vec<_>,_>>()?))
}
fn vertex(row:&BTreeMap<String,Json>)->Result<[f64;3],u32>{world(&location(row)?,xyz(field(row,"point_xyz")?)?)}
pub(crate) fn validate(reference:&BTreeMap<String,Json>,snapshot:&BTreeMap<String,Json>,prefix:bool)->Result<(),u32>{
 let selected=rows(field(snapshot,"selected_points_world")?)?;
 if !snapshot.contains_key("native_shape_type"){
  // Parent already independently validates discrete XYZ/indices/own+parent
  // matrices. Circle whole/prefix must also retain native sequential selection.
  if !text(field(reference,"subelement")?)?.is_empty(){return Err(1);}
  let data=object(field(snapshot,"geometry_data")?)?;let count=rows(field(data,"points_xyz")?)?.len();
  if selected.len()>count||!prefix&&selected.len()!=count{return Err(2);}
  if let Some(indices)=reference.get("selected_point_indices"){
   let indices=rows(indices)?;if indices.len()!=selected.len(){return Err(2);}
   for(i,v)in indices.iter().enumerate(){if integer(v,count)?!=i{return Err(1);}}
  }
  return Ok(());
 }
 if text(field(reference,"snapshot_frame")?)?!="document_world"{return Err(6);}
 let t=object(field(snapshot,"topology_data")?)?;
 let faces=rows(field(t,"faces")?)?;let edges=rows(field(t,"edges_3d")?)?;let vertices=rows(field(t,"vertices")?)?;
 let sub=text(field(reference,"subelement")?)?;
 let expected=if let Some(s)=sub.strip_prefix("Vertex"){vec![vertex(element(vertices,s)?)?]}
 else if let Some(s)=sub.strip_prefix("Face"){surface_poles(element(faces,s)?)?.ok_or(1u32)?}
 else if !sub.is_empty(){return Err(1);}
 else{
  // Exact native reader precedence: one direct surface, one direct curve,
  // otherwise unique vertices in native TopExp enumeration order.
  let surface=if faces.len()==1{surface_poles(object(&faces[0])?)?}else{None};
  if let Some(p)=surface{p}else if let Some(p)=if edges.len()==1{curve_poles(&edges[0])?}else{None}{p}
  else{
   if vertices.is_empty()||vertices.len()>1024{return Err(2);}
   let mut ordered=vertices.iter().map(|v|{let row=object(v)?;Ok((native_index(row)?,row))}).collect::<Result<Vec<_>,u32>>()?;
   ordered.sort_by_key(|v|v.0);
   for(i,(index,_))in ordered.iter().enumerate(){if *index!=i+1{return Err(3);}}
   ordered.into_iter().map(|(_,row)|vertex(row)).collect::<Result<Vec<_>,_>>()?
  }
 };
 if selected.len()>expected.len()||!prefix&&selected.len()!=expected.len(){return Err(2);}
 for(p,q)in selected.iter().zip(expected){if !source_point_matches(xyz(p)?,q){return Err(5);}}
 Ok(())
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn history_fit_geometry_absent_native_curve_preserves_vertex_fallback(){
  // Literal selection-only contract: a degenerate edge has no Curve3d, and
  // native Reader::curve returns false before reading its unique vertex.
  // No native integration or complete BREP snapshot is claimed by this unit.
  let parse=|s:&str|crate::builder_history::parse_document(s,10000,1000).unwrap();
  let r=parse(r#"{"snapshot_frame":"document_world","subelement":""}"#);
  let s=parse(r#"{"native_shape_type":6,"selected_points_world":[[6,8,10]],"topology_data":{"faces":[],"edges_3d":[{"curve_present":false,"degenerate":true,"native_class":null,"geometry_data":null}],"vertices":[{"native_topology_index":"1","point_xyz":[1,2,3],"geometry_coordinates":"native_local","native_location":[[1,0,0,5],[0,1,0,6],[0,0,1,7],[0,0,0,1]]}]}}"#);
  assert!(validate(object(&r).unwrap(),object(&s).unwrap(),false).is_ok());
 }
}
