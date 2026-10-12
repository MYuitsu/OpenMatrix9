// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned 3D edge/vertex observations; native parameter flags are not certificates.
use crate::support_snapshot::{Json,SerializedLimit,numbers};
use crate::geometry_frame::{Frame,IDENTITY};
use std::fmt::Write;
#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Edge {pub edge_id:u64,pub start_vertex:u64,pub end_vertex:u64,pub domain:[f64;2],pub tolerance:f64,pub curve_present:u32,pub degenerate:u32,pub same_parameter:u32,pub same_range:u32,pub location:[f64;16]}
#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Vertex {pub vertex_id:u64,pub point:[f64;3],pub tolerance:f64,pub location:[f64;16]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Budget,Identity,Flags,Coordinate,Tolerance,Domain,Location,MissingCurve}
#[derive(Debug)]
pub struct Geometry {edges:Vec<Edge>,vertices:Vec<Vertex>}
pub fn counts_allowed(edges:usize,vertices:usize)->bool {
 edges.checked_add(vertices).is_some_and(|n|n<=16384)&&
 edges.checked_mul(std::mem::size_of::<Edge>()).and_then(|e|vertices.checked_mul(std::mem::size_of::<Vertex>()).and_then(|v|e.checked_add(v))).and_then(|n|n.checked_add(std::mem::size_of::<Geometry>())).is_some_and(|n|n<=8*1024*1024)
}
pub fn tolerance_valid(value:f64)->bool {value.is_finite()&&(0.0..=1e150).contains(&value)}
fn valid_location(value:[f64;16])->bool {Frame::compose(IDENTITY,IDENTITY,IDENTITY,value).is_ok()}
fn matrix(j:&mut Json,value:&[f64;16])->std::fmt::Result{
 j.write_char('[')?;for r in 0..4{if r!=0{j.write_char(',')?;}numbers(j,&value[r*4..r*4+4])?;}j.write_char(']')
}
impl Geometry {
 pub fn capture(edges:&[Edge],vertices:&[Vertex])->Result<Self,Error>{
  if !counts_allowed(edges.len(),vertices.len()){return Err(Error::Budget);}
  for (i,v) in vertices.iter().enumerate(){
   if v.vertex_id!=(i+1) as u64{return Err(Error::Identity);}
   if v.point.iter().any(|p|!p.is_finite()||p.abs()>1e150){return Err(Error::Coordinate);}
   if !tolerance_valid(v.tolerance){return Err(Error::Tolerance);}
   if !valid_location(v.location){return Err(Error::Location);}
  }
  for (i,e) in edges.iter().enumerate(){
   if e.edge_id!=(i+1) as u64||e.start_vertex==0||e.end_vertex==0||e.start_vertex>vertices.len() as u64||e.end_vertex>vertices.len() as u64{return Err(Error::Identity);}
   if [e.curve_present,e.degenerate,e.same_parameter,e.same_range].iter().any(|v|*v>1){return Err(Error::Flags);}
   if e.degenerate==1&&e.start_vertex!=e.end_vertex{return Err(Error::Identity);}
   if !tolerance_valid(e.tolerance){return Err(Error::Tolerance);}
   if e.curve_present==0{if e.degenerate==0{return Err(Error::MissingCurve);}if e.domain!=[0.,0.]{return Err(Error::Domain);}}
   else if !crate::topology_snapshot::domain_valid(e.domain[0],e.domain[1]){return Err(Error::Domain);}
   if !valid_location(e.location){return Err(Error::Location);}
  }
  Ok(Self{edges:edges.to_vec(),vertices:vertices.to_vec()})
 }
 pub fn edges(&self)->&[Edge]{&self.edges}
 pub fn vertices(&self)->&[Vertex]{&self.vertices}
 pub fn json(&self)->Result<String,SerializedLimit>{
  let mut j=Json(String::new());let result=(||{
   j.write_str("{\"schema_version\":1,\"edges_3d\":[")?;
   for (i,e) in self.edges.iter().enumerate(){if i!=0{j.write_char(',')?;}
    write!(j,"{{\"edge_id\":\"{}\",\"start_vertex_id\":\"{}\",\"end_vertex_id\":\"{}\",\"domain\":",e.edge_id,e.start_vertex,e.end_vertex)?;
    if e.curve_present==0{j.write_str("null")?;}else{numbers(&mut j,&e.domain)?;}
    write!(j,",\"tolerance\":{},\"curve_present\":{},\"degenerate\":{},\"native_same_parameter\":{},\"native_same_range\":{},\"parameter_correspondence_certificate\":null,\"geometry_coordinates\":\"native_local\",\"native_location\":",e.tolerance,e.curve_present==1,e.degenerate==1,e.same_parameter==1,e.same_range==1)?;
    matrix(&mut j,&e.location)?;j.write_char('}')?;
   }
   j.write_str("],\"vertices\":[")?;
   for (i,v) in self.vertices.iter().enumerate(){if i!=0{j.write_char(',')?;}
    write!(j,"{{\"vertex_id\":\"{}\",\"point_xyz\":",v.vertex_id)?;numbers(&mut j,&v.point)?;
    write!(j,",\"tolerance\":{},\"geometry_coordinates\":\"native_local\",\"native_location\":",v.tolerance)?;
    matrix(&mut j,&v.location)?;j.write_char('}')?;
   }
   j.write_str("]}")
  })();result.map(|_|j.0).map_err(|_|SerializedLimit)
 }
}
